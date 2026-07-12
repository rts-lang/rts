use std::any::Any;
use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Once;
use libc::c_int;
use libffi::middle::{Arg, Cif, CodePtr, Type};
use libloading::Library;
use crate::parser::structure::structureType::StructureType;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Это реализация изоляции для FFI - чтобы мы могли безопасно обрабатывать такие пограничные места.
//
// Изоляция FFI без дочерних процессов и без MMU per-process: mmap-арена + guard-страница +
// sigsetjmp/signal. FFI живут в одном адресном пространстве Runtime.
//
// На каждый вызов:
//  1. ByteVector-аргументы копируются в свежую mmap-арену (не в обычный heap Vec).
//  2. Сразу за арену ставится guard-страница (PROT_NONE) - любой выход FFI за пределы своих
//     данных (даже вычисленным внутри FFI адресом) -> аппаратный page fault, а не тихая порча.
//  3. Сам вызов идёт под sigsetjmp; SIGSEGV/SIGBUS/SIGILL перехватывается глобальным обработчиком,
//     который делает siglongjmp обратно - runtime не падает, результатом вызова становится None.
//  4. По выходу из вызова арена дропается (munmap) целиком - что бы FFI в ней ни испортил,
//     это не всплывает наружу.
//
// Что НЕ защищается - это забота ОС, не рантайма: память, которую FFI-код успел
// испортить внутри собственной арены до трапа; сторонние ресурсы (файлы, сеть, чужой malloc-heap).
//
// Единственная дыра - дисциплинарная, не техническая: если в FFI передать сырой указатель на
// runtime-объект вместо копии в арене - защиты нет. Правило: только copy-in, никаких
// прямых ссылок на runtime.
//
// Сейчас арена - per-call, а recovery-стек - thread_local, разделяемого состояния между вызовами нет.
// Однопоточность рантайма не требуется явно - но recovery-стек thread_local, а не общий static, 
// специально: синхронные сигналы POSIX доставляет потоку-виновнику fault'а,
// так что per-thread стек остаётся корректным, даже если рантайм когда-нибудь станет
// многопоточным (в отличие от общего static, где это была бы гонка).
//
// todo (feature)
//  Region table (RegionId -> FfiRegion) - не реализована: сейчас Pointer всегда
//  возвращается как None (см. ниже), кросс-вызывной адресации внутри #ffi{}-блока пока нет.
//  Понадобится, когда несколько FFI-вызовов должны будут ссылаться друг на друга по адресам
//  внутри одного блока - тогда таблица регионов ложится поверх уже готовой арены/guard-страницы.
//
// todo (feature)
//  Если сама FFI-библиотека написана на Rust и паникует - unwind через границу extern "C"
//  (не "C-unwind") это UB, и начиная с современных версий Rust компилятор сам вызывает abort()
//  при попытке такого unwind'а. Это SIGABRT, не SIGSEGV/SIGBUS/SIGILL - наш обработчик его
//  не перехватывает.

// =================================================================================================

// todo desc
#[derive(Clone)]
#[derive(Debug)] // todo remove
pub enum FFIValue
{
  None, // Просто пустое значение
  //
  U8(u8),
  U16(u16),
  U32(u32),
  U64(u64),
  Usize(usize), // todo Должно быть тут?
  //
  I8(i8),
  I16(i16),
  I32(i32),
  I64(i64),
  Isize(isize), // todo Должно быть тут?
  //
  F32(f32),
  F64(f64),
  //
  Bool(bool),
  // Универсальный контейнер для произвольных байтовых данных
  ByteVector(Vec<u8>) // todo Не знаю насколько правильно это иметь тут, но
                      //  это самый простой вариант передачи без нарушения адресного пространства.
                      //  Но опять же кодировки и другие штуки как будут тут себя вести?
                      //  Мб легче проброс данных по адресам или что-то?
}

// todo desc
//
// todo Работает так же как getStructureType() - и они могут быть вынесены в абстракцию?
//
// todo Вообще надо сделать min/max/default для поведения примитивов.
//
// todo По факту не нарушает типизацию, но тип может если был Usize(19) то для вызова быть U8(19),
//  что по факту ошибка, так как будет сменен тип данных - важно его сохранять?
impl TryFrom<&mut Token> for FFIValue
{
  type Error = String;

  // todo desc
  fn try_from(token: &mut Token) -> Result<Self, Self::Error>
  {
    let dataType: &TokenType = token.getDataType();

    let data: String = match token.getData().toString() {
      Some(s) => s,
      None => return Err("Token data is empty".to_owned()),
    };

    match dataType
    {
      TokenType::UInt =>
      {
        if let Ok(value) = data.parse::<u128>()
        {
          if value <= u8::MAX as u128         { Ok(FFIValue::U8(value as u8))         }
          else if value <= u16::MAX as u128   { Ok(FFIValue::U16(value as u16))       }
          else if value <= u32::MAX as u128   { Ok(FFIValue::U32(value as u32))       }
          else if value <= u64::MAX as u128   { Ok(FFIValue::U64(value as u64))       }
          else if value <= usize::MAX as u128 { Ok(FFIValue::Usize(value as usize))   }
          else { Err(format!("UInt out of range: {}", value)) }
        } else {
          Err(format!("Failed to parse UInt: {}", data))
        }
      }
      TokenType::Int =>
      {
        if let Ok(value) = data.parse::<i128>()
        {
          if value >= i8::MIN as i128 && value <= i8::MAX as i128            { Ok(FFIValue::I8(value as i8))       }
          else if value >= i16::MIN as i128 && value <= i16::MAX as i128     { Ok(FFIValue::I16(value as i16))     }
          else if value >= i32::MIN as i128 && value <= i32::MAX as i128     { Ok(FFIValue::I32(value as i32))     }
          else if value >= i64::MIN as i128 && value <= i64::MAX as i128     { Ok(FFIValue::I64(value as i64))     }
          else if value >= isize::MIN as i128 && value <= isize::MAX as i128 { Ok(FFIValue::Isize(value as isize)) }
          else { Err(format!("Int out of range: {}", value)) }
        } else {
          Err(format!("Failed to parse Int: {}", data))
        }
      }
      TokenType::UFloat | TokenType::Float =>
      {
        if let Ok(value) = data.parse::<f64>()
        {
          if value >= f32::MIN as f64 && value <= f32::MAX as f64 { Ok(FFIValue::F32(value as f32)) }
          else if value >= f64::MIN && value <= f64::MAX          { Ok(FFIValue::F64(value))        }
          else { Err(format!("Float out of range: {}", value)) }
        } else {
          Err(format!("Failed to parse Float: {}", data))
        }
      }
      TokenType::String =>
      {
        Ok(FFIValue::ByteVector(data.into_bytes()))
      }
      _ => Err("Unsupported TokenType".to_owned()),
    }
  }
}

// =================================================================================================

// todo desc
#[derive(Clone, Copy)]
pub enum FFIType
{
  None, // Просто пустое значение
  //
  U8,
  U16,
  U32,
  U64,
  Usize, // todo Должно быть тут?
  //
  I8,
  I16,
  I32,
  I64,
  Isize, // todo Должно быть тут?
  //
  F32,
  F64,
  //
  Bool,
  Pointer // Сырой указатель
}

impl TryFrom<StructureType> for FFIType
{
  type Error = String;

  // todo desc
  fn try_from(ty: StructureType) -> Result<Self, Self::Error>
  {
    match ty
    {
      StructureType::None => Ok(FFIType::None),

      StructureType::U8 => Ok(FFIType::U8),
      StructureType::U16 => Ok(FFIType::U16),
      StructureType::U32 => Ok(FFIType::U32),
      StructureType::U64 => Ok(FFIType::U64),
      StructureType::Usize => Ok(FFIType::Usize),

      StructureType::I8 => Ok(FFIType::I8),
      StructureType::I16 => Ok(FFIType::I16),
      StructureType::I32 => Ok(FFIType::I32),
      StructureType::I64 => Ok(FFIType::I64),
      StructureType::Isize => Ok(FFIType::Isize),

      StructureType::F32 => Ok(FFIType::F32),
      StructureType::F64 => Ok(FFIType::F64),

      StructureType::Bool => Ok(FFIType::Bool),

      _ => Err(format!("Unsupported FFI type: {}", ty.to_string())),
    }
  }
}

// =================================================================================================
// Recovery-механизм: sigsetjmp/siglongjmp + сигналы
// =================================================================================================

/// Максимальная глубина вложенных FFI-вызовов (реентерабельность: FFI зовёт колбэк в runtime,
/// тот делает ещё один FFI-вызов - у каждого уровня своя точка восстановления).
const MaxFfiDepth: usize = 64; // todo Будет удалено при решение issue #79

/// Непрозрачный буфер под sigjmp_buf. На glibc/x86_64 реальный размер - 200 байт;
/// берём с запасом на 512, чтобы не зависеть от точного layout на других платформах.
#[repr(C, align(16))]
struct SigJmpBuf([u8; 512]);

extern "C"
{
  /// sigsetjmp в C - макрос, реальный экспортируемый символ называется __sigsetjmp
  /// (сохраняет маску сигналов - без этого после восстановления SIGSEGV мог бы остаться
  /// заблокированным и следующий трап убил бы процесс).
  #[link_name = "__sigsetjmp"]
  fn ffiSigSetJmp(env: *mut SigJmpBuf, savesigs: c_int) -> c_int;
  /// todo desc
  fn siglongjmp(env: *mut SigJmpBuf, val: c_int) -> !;
}

struct RecoveryStack
{
  bufs: [MaybeUninit<SigJmpBuf>; MaxFfiDepth],
  depth: usize,
}

thread_local!
{
  /// Стек точек восстановления - по одной на каждый активный (в т.ч. вложенный) FFI-вызов
  /// этого потока. thread_local, а не общий static: синхронные сигналы POSIX доставляет
  /// именно потоку-виновнику fault'а, поэтому per-thread стек остаётся корректным даже при
  /// гипотетической многопоточности (общий static тут был бы гонкой на depth/bufs).
  static Recovery: UnsafeCell<RecoveryStack> = UnsafeCell::new(RecoveryStack {
    bufs: unsafe { MaybeUninit::uninit().assume_init() }, // массив MaybeUninit - всегда валиден
    depth: 0,
  });

  /// Альтернативный стек сигнала на поток. Нужен на случай, если FFI-функция исчерпает не
  /// арену, а свой реальный call-стек (глубокая рекурсия) - тогда на основном стеке может не
  /// остаться места под кадр обработчика, и без sigaltstack процесс упадёt даже с sigaction.
  static AltStack: UnsafeCell<Option<Vec<u8>>> = UnsafeCell::new(None);
}

/// todo desc
extern "C" fn onFfiTrap(signum: c_int)
{
  Recovery.with(|cell| unsafe {
    let stack: &mut RecoveryStack = &mut *cell.get();
    if stack.depth == 0
    {
      // Сигнал пришёл не во время защищённого FFI-вызова - баг в самом рантайме,
      // а не в FFI. Маскировать чужую ошибку опаснее, чем честно уронить процесс:
      // восстанавливаем поведение по умолчанию и роняем по-настоящему.
      libc::signal(signum, libc::SIG_DFL);
      libc::raise(signum);
      return;
    }
    let top: usize = stack.depth - 1;
    siglongjmp(stack.bufs[top].as_mut_ptr(), 1);
  });
}

/// todo desc
fn ensureAltStackInstalled()
{
  AltStack.with(|cell| unsafe {
    let slot: &mut Option<Vec<u8>> = &mut *cell.get();
    if slot.is_some() { return; }

    let size: usize = libc::SIGSTKSZ.max(64 * 1024);
    let mut buf: Vec<u8> = vec![0u8; size];
    let stack: libc::stack_t = libc::stack_t {
      ss_sp: buf.as_mut_ptr() as *mut c_void,
      ss_flags: 0,
      ss_size: size,
    };
    libc::sigaltstack(&stack, std::ptr::null_mut());
    *slot = Some(buf); // держим буфер живым весь срок жизни потока
  });
}

/// todo desc
fn installSignalHandlersOnce()
{
  static InstallOnce: Once = Once::new();
  InstallOnce.call_once(|| unsafe {
    let mut action: libc::sigaction = std::mem::zeroed();
    action.sa_sigaction = onFfiTrap as usize;
    action.sa_flags = libc::SA_NODEFER | libc::SA_ONSTACK;
    libc::sigemptyset(&mut action.sa_mask);
    for &signal in &[libc::SIGSEGV, libc::SIGBUS, libc::SIGILL]
    {
      libc::sigaction(signal, &action, std::ptr::null_mut());
    }
  });
}

/// Выполняет `f` под защитой setjmp/signal.
/// 
/// `Ok(None)` - поймали SIGSEGV/SIGBUS/SIGILL: `f` не доработала, но рантайм цел.
/// 
/// `Ok(Some)` - вызов отработал штатно.
/// 
/// `Err` - превышена глубина вложенности (логический лимит, не крах).
fn protectedFfiCall<T>(f: impl FnOnce() -> T) -> Result<Option<T>, String>
{
  installSignalHandlersOnce();
  ensureAltStackInstalled();

  Recovery.with(|cell| {
    let stack: &mut RecoveryStack = unsafe { &mut *cell.get() };
    if stack.depth >= MaxFfiDepth
    {
      return Err("FFI recursion depth exceeded".to_string());
    }
    let slot: usize = stack.depth;
    stack.depth += 1;

    let jumped: c_int = unsafe { ffiSigSetJmp(stack.bufs[slot].as_mut_ptr(), 1) };
    let outcome: Option<T> = if jumped == 0
    {
      Some(f())
    } else {
      None // сюда возвращаемся через siglongjmp из onFfiTrap
    };

    let stack: &mut RecoveryStack = unsafe { &mut *cell.get() };
    stack.depth -= 1;
    Ok(outcome)
  })
}

// =================================================================================================
// Арена: mmap + guard-страница
// =================================================================================================

/// Изолированная арена под один FFI-вызов: отдельный mmap-регион + guard-страница (PROT_NONE)
/// сразу за данными. Любая запись FFI за пределы `dataSize` (даже вычисленным внутри FFI
/// адресом) -> аппаратный SIGSEGV вместо тихой порчи памяти рантайма.
struct FfiArena
{
  /// todo desc
  base: *mut u8,
  /// todo desc
  dataSize: usize,
  /// todo desc
  mapSize: usize,
}

impl FfiArena
{
  /// todo desc
  fn new(requestedSize: usize) -> Result<Self, String>
  {
    unsafe {
      let pageSize: usize = libc::sysconf(libc::_SC_PAGESIZE) as usize;
      let dataSize: usize = (requestedSize.max(1) + pageSize - 1) & !(pageSize - 1);
      let mapSize: usize = dataSize + pageSize; // + guard-страница

      let base: *mut c_void = libc::mmap(
        std::ptr::null_mut(),
        mapSize,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
        -1,
        0,
      );
      if base == libc::MAP_FAILED
      {
        return Err(format!("mmap failed: {}", std::io::Error::last_os_error()));
      }

      let guard: *mut c_void = base.add(dataSize);
      if libc::mprotect(guard, pageSize, libc::PROT_NONE) != 0
      {
        let err = std::io::Error::last_os_error();
        libc::munmap(base, mapSize);
        return Err(format!("mprotect failed: {}", err));
      }

      Ok(Self { base: base as *mut u8, dataSize, mapSize })
    }
  }

  /// todo desc
  fn basePtr(&self) -> *mut u8 { self.base }

  /// Safety: offset + bytes.len() <= dataSize (гарантируется раскладкой в performCall)
  unsafe fn writeAt(&mut self, offset: usize, bytes: &[u8])
  {
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.base.add(offset), bytes.len());
  }
}

impl Drop for FfiArena
{
  /// Освобождает mmap целиком - что бы FFI внутри арены ни испортил, оно уходит вместе с ней
  fn drop(&mut self)
  {
    unsafe { libc::munmap(self.base as *mut c_void, self.mapSize); }
  }
}

// =================================================================================================
// FFI вызов
// =================================================================================================

/// Загружает библиотеку, копирует ByteVector-аргументы в изолированную арену 
/// и выполняет FFI-вызов под защитой setjmp/signal.
fn performCall(libraryPath: &str, methodName: &str, args: &[FFIValue], resultType: FFIType) -> Result<FFIValue, String>
{
  // 1. Библиотека
  let library: Library = unsafe {
    Library::new(libraryPath)
      .map_err(|e| format!("Failed to load library: {}", e))?
  };

  // 2. Указатель на функцию
  let functionPointer: *mut c_void = unsafe {
    *library
      .get::<*mut c_void>(methodName.as_bytes())
      .map_err(|e| format!("Failed to find function: {}", e))?
  };

  // 3. Типы аргументов
  let argTypes: Vec<Type> = args
    .iter()
    .map(|arg| match arg
    {
      FFIValue::U8(_) => Ok(Type::u8()),
      FFIValue::U16(_) => Ok(Type::u16()),
      FFIValue::U32(_) => Ok(Type::u32()),
      FFIValue::U64(_) => Ok(Type::u64()),
      FFIValue::Usize(_) => Ok(Type::usize()),
      FFIValue::I8(_) => Ok(Type::i8()),
      FFIValue::I16(_) => Ok(Type::i16()),
      FFIValue::I32(_) => Ok(Type::i32()),
      FFIValue::I64(_) => Ok(Type::i64()),
      FFIValue::Isize(_) => Ok(Type::isize()),
      FFIValue::F32(_) => Ok(Type::f32()),
      FFIValue::F64(_) => Ok(Type::f64()),
      FFIValue::Bool(_) => Ok(Type::u8()), // bool as u8
      FFIValue::ByteVector(_) => Ok(Type::pointer()),
      FFIValue::None => Err("Cannot pass None as argument".to_string())
    })
    .collect::<Result<Vec<_>, _>>()?;

  // 4. Тип результата
  let returnType: Type = match resultType
  {
    FFIType::None => Type::void(),
    FFIType::U8 => Type::u8(),
    FFIType::U16 => Type::u16(),
    FFIType::U32 => Type::u32(),
    FFIType::U64 => Type::u64(),
    FFIType::Usize => Type::usize(),
    FFIType::I8 => Type::i8(),
    FFIType::I16 => Type::i16(),
    FFIType::I32 => Type::i32(),
    FFIType::I64 => Type::i64(),
    FFIType::Isize => Type::isize(),
    FFIType::F32 => Type::f32(),
    FFIType::F64 => Type::f64(),
    FFIType::Bool => Type::u8(),
    FFIType::Pointer => Type::pointer()
  };

  // 5. CIF
  let cif: Cif = Cif::new(argTypes.into_iter(), returnType);

  // 6. Разметка арены под все ByteVector-аргументы этого вызова (одна арена на вызов -
  //    "зона под каждый ffi", как и требовалось; не одна арена на каждый отдельный аргумент).
  let mut layout: Vec<(usize, usize)> = Vec::with_capacity(args.len());
  let mut arenaNeeded: usize = 0;
  for arg in args
  {
    if let FFIValue::ByteVector(v) = arg
    {
      let offset: usize = (arenaNeeded + 7) & !7; // выравнивание по 8 байт
      layout.push((offset, v.len()));
      arenaNeeded = offset + v.len();
    } else {
      layout.push((0, 0));
    }
  }

  let mut arena: Option<FfiArena> = if arenaNeeded > 0
  {
    Some(FfiArena::new(arenaNeeded)?)
  } else {
    None // чисто скалярный вызов - арена не нужна, но setjmp/signal защита всё равно активна
  };

  if let Some(arenaRef) = arena.as_mut()
  {
    for (arg, &(offset, len)) in args.iter().zip(layout.iter())
    {
      if let FFIValue::ByteVector(v) = arg
      {
        unsafe { arenaRef.writeAt(offset, &v[..len]); }
      }
    }
  }

  // 7. Store all boxed values first (no references yet); ByteVector - указатель внутрь арены
  let mut storage: Vec<Box<dyn Any>> = Vec::with_capacity(args.len());
  for (i, arg) in args.iter().enumerate()
  {
    match arg
    {
      FFIValue::U8(v) => storage.push(Box::new(*v)),
      FFIValue::U16(v) => storage.push(Box::new(*v)),
      FFIValue::U32(v) => storage.push(Box::new(*v)),
      FFIValue::U64(v) => storage.push(Box::new(*v)),
      FFIValue::Usize(v) => storage.push(Box::new(*v)),
      FFIValue::I8(v) => storage.push(Box::new(*v)),
      FFIValue::I16(v) => storage.push(Box::new(*v)),
      FFIValue::I32(v) => storage.push(Box::new(*v)),
      FFIValue::I64(v) => storage.push(Box::new(*v)),
      FFIValue::Isize(v) => storage.push(Box::new(*v)),
      FFIValue::F32(v) => storage.push(Box::new(*v)),
      FFIValue::F64(v) => storage.push(Box::new(*v)),
      FFIValue::Bool(b) => storage.push(Box::new(if *b { 1u8 } else { 0u8 })),
      FFIValue::ByteVector(_) => {
        let (offset, _len): (usize, usize) = layout[i];
        let rawPointer: *mut c_void = unsafe {
          arena.as_mut().unwrap().basePtr().add(offset) as *mut c_void
        };
        storage.push(Box::new(rawPointer));
      }
      FFIValue::None => return Err("Cannot pass None".to_string()),
    }
  }

  // 8. Build arguments using references to the stored boxes (no further mutations)
  let mut ffiArgs: Vec<Arg> = Vec::with_capacity(args.len());
  for (i, arg) in args.iter().enumerate()
  {
    match arg
    {
      FFIValue::U8(_) => ffiArgs.push(Arg::new(storage[i].downcast_ref::<u8>().unwrap())),
      FFIValue::U16(_) => ffiArgs.push(Arg::new(storage[i].downcast_ref::<u16>().unwrap())),
      FFIValue::U32(_) => ffiArgs.push(Arg::new(storage[i].downcast_ref::<u32>().unwrap())),
      FFIValue::U64(_) => ffiArgs.push(Arg::new(storage[i].downcast_ref::<u64>().unwrap())),
      FFIValue::Usize(_) => ffiArgs.push(Arg::new(storage[i].downcast_ref::<usize>().unwrap())),
      FFIValue::I8(_) => ffiArgs.push(Arg::new(storage[i].downcast_ref::<i8>().unwrap())),
      FFIValue::I16(_) => ffiArgs.push(Arg::new(storage[i].downcast_ref::<i16>().unwrap())),
      FFIValue::I32(_) => ffiArgs.push(Arg::new(storage[i].downcast_ref::<i32>().unwrap())),
      FFIValue::I64(_) => ffiArgs.push(Arg::new(storage[i].downcast_ref::<i64>().unwrap())),
      FFIValue::Isize(_) => ffiArgs.push(Arg::new(storage[i].downcast_ref::<isize>().unwrap())),
      FFIValue::F32(_) => ffiArgs.push(Arg::new(storage[i].downcast_ref::<f32>().unwrap())),
      FFIValue::F64(_) => ffiArgs.push(Arg::new(storage[i].downcast_ref::<f64>().unwrap())),
      FFIValue::Bool(_) => ffiArgs.push(Arg::new(storage[i].downcast_ref::<u8>().unwrap())),
      FFIValue::ByteVector(_) => {
        let dataPointer: &*mut c_void = storage[i].downcast_ref::<*mut c_void>().unwrap();
        ffiArgs.push(Arg::new(dataPointer));
      }
      FFIValue::None => return Err("Cannot pass None".to_string())
    }
  }

  // 9. Сам вызов - под защитой setjmp/signal. Всё unsafe здесь предполагает чужой код.
  let codePointer: CodePtr = CodePtr(functionPointer);
  let trapped: Option<FFIValue> = protectedFfiCall(|| unsafe {
    match resultType
    {
      FFIType::None => { cif.call::<()>(codePointer, &ffiArgs); FFIValue::None }
      FFIType::U8 => FFIValue::U8(cif.call::<u8>(codePointer, &ffiArgs)),
      FFIType::U16 => FFIValue::U16(cif.call::<u16>(codePointer, &ffiArgs)),
      FFIType::U32 => FFIValue::U32(cif.call::<u32>(codePointer, &ffiArgs)),
      FFIType::U64 => FFIValue::U64(cif.call::<u64>(codePointer, &ffiArgs)),
      FFIType::Usize => FFIValue::Usize(cif.call::<usize>(codePointer, &ffiArgs)),
      FFIType::I8 => FFIValue::I8(cif.call::<i8>(codePointer, &ffiArgs)),
      FFIType::I16 => FFIValue::I16(cif.call::<i16>(codePointer, &ffiArgs)),
      FFIType::I32 => FFIValue::I32(cif.call::<i32>(codePointer, &ffiArgs)),
      FFIType::I64 => FFIValue::I64(cif.call::<i64>(codePointer, &ffiArgs)),
      FFIType::Isize => FFIValue::Isize(cif.call::<isize>(codePointer, &ffiArgs)),
      FFIType::F32 => FFIValue::F32(cif.call::<f32>(codePointer, &ffiArgs)),
      FFIType::F64 => FFIValue::F64(cif.call::<f64>(codePointer, &ffiArgs)),
      FFIType::Bool => FFIValue::Bool(cif.call::<u8>(codePointer, &ffiArgs) != 0),
      FFIType::Pointer => {
        FFIValue::None // Пространства теперь общие, но кросс-вызывная адресация (table/region)
                       // ещё не реализована - см. todo вверху файла. Пока осознанно None.
      }
    }
  })?;
  // `arena` дропается тут обычным Rust scope-exit: она была создана ДО protectedFfiCall,
  // поэтому siglongjmp этот Drop не пропускает - munmap отработает в любом исходе вызова.

  Ok(trapped.unwrap_or(FFIValue::None)) // None здесь = поймали трап; runtime цел, вызов - не удался
}

// =================================================================================================

/// Безопасный вызов внешней FFI-функции;
/// 
/// mmap-арена + guard-страница + sigsetjmp/signal на каждый вызов;
/// весь поток синхронный, без своей многопоточности/асинхронности.
pub fn callExternal(libraryPath: &str, methodName: &str, parametersTokens: &mut [Token], resultType: StructureType) -> Result<FFIValue, String>
{
  // Обработка параметров
  let parameters: Vec<FFIValue> = parametersTokens
    .iter_mut()
    .map(FFIValue::try_from)  // Автоматически использует реализацию TryFrom<&Token>
    .collect::<Result<Vec<_>, _>>()?; // При первой ошибке возвращаем её

  // catch_unwind - отдельный рубеж от setjmp/signal: ловит Rust-панику в коде разметки/маршалинга
  // (не в самом чужом вызове - тот уже под protectedFfiCall).
  let outcome: Result<Result<FFIValue, String>, Box<dyn Any + Send>> =
    catch_unwind(AssertUnwindSafe(|| {
      performCall(libraryPath, methodName, &parameters, FFIType::try_from(resultType)?)
    }));

  match outcome
  {
    Ok(result) => result,
    Err(_) => Err("FFI function panicked".to_string()),
  }
}

// =================================================================================================
