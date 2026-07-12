use std::any::Any;
use std::io::{Read, Stdin, Stdout, Write};
use std::sync::{Mutex, MutexGuard, OnceLock};
use serde::{Deserialize, Serialize};
use libloading::Library;
use std::process::{Command, Stdio, Child, ChildStdin, ChildStdout};
use std::env;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use crate::parser::structure::ffi::dynamicCobsAccumulator::{DynamicCobsAccumulator, DynamicFeedResult};
use crate::parser::structure::ffi::stdoutRedirect::StdoutRedirect;
use crate::parser::structure::structureType::StructureType;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use libffi::middle::{Arg, Cif, CodePtr, Type};
use std::ffi::c_void;
// =================================================================================================

// Это реализация изоляции для FFI - чтобы мы могли безопасно обрабатывать такие пограничные места.
// 
// Она также всегда перезапускается, чтобы мы могли её использовать постоянно;
// А также синхронная, чтобы не нарушать поток и вести себя как обычный запуск чего-то.
// Собственно, ей и не надо иметь многопоточность или асинхронность.
//
// Также worker не защищает от бесконечных циклов - это не обязанность языка;
// Это проблема программы, как обычно, просто переписывается код - это нормально.
// К тому же, это невозможно было бы сделать, так как нельзя определить - что задержка, а что нет.
//
// Сама изоляция накладывает небольшие расходы. Для примера - на легкой задаче это выглядит как
// простой. В самих тестах скорость и работа быстрая, но за кадром есть 
// нагрузка жизни самого дочернего процесса; На больших задачах она не заметна.
//
// todo (feature)
//  В целом, для скорости мы можем выдать флаг на изоляцию - тогда код мог бы упасть, но смысл 
//  как раз в том, чтобы ставить его в release. Но это может быть использовано неверно и 
//  требует четкого механизма безопасности - потому что понапишут потом кода с ошибками.
//
// todo (feature)
//  Мы также могли бы выдать флаги или какой-то механизм для регулирования WorkerManager;
//  Потому что возможно, это будет гибко при написании чего-то для своего FFI в самом коде.
//
// todo (feature)
//  Теоретически мы могли бы добавить многопоточность:
//  - Несколько WorkerManager, 2 должно хватать для само-замены в горячих местах; Но мы не можем 
//    знать горячие места чтобы понять, что надо 10 штук запустить, поэтому 2.
//  - Реальные несколько потоков, но это потребовало бы четкой системы движения по строкам кода и
//    решения зависимостей.

// =================================================================================================

// todo desc
#[derive(Clone, Serialize, Deserialize)]
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
    
    println!("try_from: {}:{}",data,dataType.to_string());

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
#[derive(Serialize, Deserialize)]
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

/// Запрос, отправляемый от родителя воркеру
#[derive(Serialize, Deserialize)]
struct WorkerRequest 
{
  /// Путь к динамической библиотеке
  libraryPath: String,
  /// Имя вызываемой функции (символ)
  methodName: String,
  /// Аргументы
  args: Vec<FFIValue>,
  /// Тип результата todo По идее должен быть сам результат
  resultType: FFIType
}

/// Ответ воркера родителю
#[derive(Serialize, Deserialize)]
struct WorkerResponse 
{
  /// Успешный результат
  result: Option<FFIValue>,
  /// Сообщение об ошибке
  error: Option<String>,
}

/// Главный цикл воркера: чтение запросов, выполнение и отправка ответов
pub fn workerMain() 
{
  // Поток входных данных
  let mut inputStream: Stdin = std::io::stdin();
  // Поток выходных данных
  let mut outputStream: Stdout = std::io::stdout();
  
  // Буфер чтения
  let mut rawBuffer: [u8; 8192] = [0u8; 8192];
  // Аккумулятор COBS-пакетов
  let mut cobsBuffer: DynamicCobsAccumulator = DynamicCobsAccumulator::new();

  loop 
  { // Цикл обработки запросов
    let bytesRead: usize = match inputStream.read(&mut rawBuffer) {
      Ok(n) => n,
      Err(_) => break,
    };

    // Проверка пустого чтения потока
    if bytesRead == 0 {
      break;
    }

    let mut window: &[u8] = &rawBuffer[..bytesRead];

    // Разбор входного буфера по COBS окнам
    while !window.is_empty() 
    {
      window = match cobsBuffer.feed::<WorkerRequest>(window) 
      {
        // Входные данные полностью потреблены, ожидание новых
        DynamicFeedResult::Consumed => break,
        // Ошибка десериализации пакета
        DynamicFeedResult::DeserError(e) => 
        {
          let response: WorkerResponse = WorkerResponse {
            result: None,
            error: Some(format!("Deserialization error: {:?}", e)),
          };
          if let Ok(bytes) = postcard::to_allocvec_cobs(&response) {
            let _ = outputStream.write_all(&bytes);
            let _ = outputStream.flush();
          }
          cobsBuffer.clear();
          break;
        }
        // Успешное извлечение запроса
        DynamicFeedResult::Success { data, remaining } => 
        {
          // Перенаправляем stdout на время обработки запроса
          let response: WorkerResponse = 
          {
            let _redirect: StdoutRedirect = StdoutRedirect::new();

            let catchResult: Result<Result<FFIValue, String>, Box<dyn Any + Send>> =
              catch_unwind(AssertUnwindSafe(|| processRequest(&data)));
            // Если паника перехвачена внутри worker, он остается жив и возвращает ошибку родителю;
            // Родитель получает штатный Err и не делает дорогостоящий restart.

            match catchResult 
            {
              // Успешное выполнение запроса
              Ok(Ok(res)) => 
                WorkerResponse { result: Some(FFIValue::Usize(0/*res*/)), error: None }, // todo Сделать result type чтобы был а не ручной
              // Логическая ошибка обработки запроса
              Ok(Err(err)) => WorkerResponse { result: None, error: Some(err) },
              // Паника внутри FFI
              Err(_) => WorkerResponse {
                result: None,
                error: Some("FFI function panicked".to_string()),
              },
            }
          }; // Здесь redirect уничтожается, stdout восстанавливается

          if let Ok(bytes) = postcard::to_allocvec_cobs(&response) {
            // Сериализация ответа в COBS-кадр
            let _ = outputStream.write_all(&bytes);
            // Немедленная отправка данных в stdout
            let _ = outputStream.flush();
          }
          // Остаток буфера после извлечения пакета
          remaining
        }
        //
      };
    }
    //
  }
}

/// Загружает библиотеку, вызывает FFI-функцию с произвольными аргументами и 
/// возвращает результат в виде FFIValue.
fn processRequest(request: &WorkerRequest) -> Result<FFIValue, String> 
{
  // 1. Load the library
  let library: Library = unsafe {
    Library::new(&request.libraryPath)
      .map_err(|e| format!("Failed to load library: {}", e))?
  };

  // 2. Get function pointer
  let functionPointer: *mut c_void = unsafe { // todo return value? или оно ниже уже есть?
    *library
      .get::<*mut c_void>(request.methodName.as_bytes())
      .map_err(|e| format!("Failed to find function: {}", e))?
  };

  // 3. Build argument types
  let argTypes: Vec<Type> = request
    .args
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

  // 4. Return type
  let returnType: Type = match request.resultType 
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

  // 5. Create CIF
  let cif: Cif = Cif::new(argTypes.into_iter(), returnType);

  // 6. Store all boxed values first (no references yet)
  let mut storage: Vec<Box<dyn Any>> = Vec::with_capacity(request.args.len());
  for arg in &request.args 
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
      FFIValue::ByteVector(v) => {
        let mut byteVector: Vec<u8> = v.clone();
        let rawPointer: *mut c_void = byteVector.as_mut_ptr() as *mut c_void;
        storage.push(Box::new((byteVector, rawPointer)));
      }
      FFIValue::None => return Err("Cannot pass None".to_string()),
    }
  }

  // 7. Build arguments using references to the stored boxes (no further mutations)
  let mut args: Vec<Arg> = Vec::with_capacity(request.args.len());
  for (i, arg) in request.args.iter().enumerate() 
  {
    match arg 
    {
      FFIValue::U8(_) => {
        let val: &u8 = storage[i].downcast_ref::<u8>().unwrap();
        args.push(Arg::new(val));
      }
      FFIValue::U16(_) => {
        let val: &u16 = storage[i].downcast_ref::<u16>().unwrap();
        args.push(Arg::new(val));
      }
      FFIValue::U32(_) => {
        let val: &u32 = storage[i].downcast_ref::<u32>().unwrap();
        args.push(Arg::new(val));
      }
      FFIValue::U64(_) => {
        let val: &u64 = storage[i].downcast_ref::<u64>().unwrap();
        args.push(Arg::new(val));
      }
      FFIValue::Usize(_) => {
        let val: &usize = storage[i].downcast_ref::<usize>().unwrap();
        args.push(Arg::new(val));
      }
      FFIValue::I8(_) => {
        let val: &i8 = storage[i].downcast_ref::<i8>().unwrap();
        args.push(Arg::new(val));
      }
      FFIValue::I16(_) => {
        let val: &i16 = storage[i].downcast_ref::<i16>().unwrap();
        args.push(Arg::new(val));
      }
      FFIValue::I32(_) => {
        let val: &i32 = storage[i].downcast_ref::<i32>().unwrap();
        args.push(Arg::new(val));
      }
      FFIValue::I64(_) => {
        let val: &i64 = storage[i].downcast_ref::<i64>().unwrap();
        args.push(Arg::new(val));
      }
      FFIValue::Isize(_) => {
        let val: &isize = storage[i].downcast_ref::<isize>().unwrap();
        args.push(Arg::new(val));
      }
      FFIValue::F32(_) => {
        let val: &f32 = storage[i].downcast_ref::<f32>().unwrap();
        args.push(Arg::new(val));
      }
      FFIValue::F64(_) => {
        let val: &f64 = storage[i].downcast_ref::<f64>().unwrap();
        args.push(Arg::new(val));
      }
      FFIValue::Bool(_) => {
        let val: &u8 = storage[i].downcast_ref::<u8>().unwrap();
        args.push(Arg::new(val));
      }
      FFIValue::ByteVector(_) => {
        let dataTuple: &(Vec<u8>, *mut c_void) = 
          storage[i].downcast_ref::<(Vec<u8>, *mut c_void)>().unwrap();
        args.push(Arg::new(&dataTuple.1));
      }
      FFIValue::None => return Err("Cannot pass None".to_string())
    }
  }

  // 8. Call the FFI function (all unsafe calls are wrapped)
  let codePointer: CodePtr = CodePtr(functionPointer);
  let result: FFIValue = match request.resultType 
  {
    FFIType::None => {
      unsafe { cif.call::<()>(codePointer, &args) };
      FFIValue::None
    }
    FFIType::U8 => {
      let val: u8 = unsafe { cif.call::<u8>(codePointer, &args) };
      FFIValue::U8(val)
    }
    FFIType::U16 => {
      let val: u16 = unsafe { cif.call::<u16>(codePointer, &args) };
      FFIValue::U16(val)
    }
    FFIType::U32 => {
      let val: u32 = unsafe { cif.call::<u32>(codePointer, &args) };
      FFIValue::U32(val)
    }
    FFIType::U64 => {
      let val: u64 = unsafe { cif.call::<u64>(codePointer, &args) };
      FFIValue::U64(val)
    }
    FFIType::Usize => {
      let val: usize = unsafe { cif.call::<usize>(codePointer, &args) };
      FFIValue::Usize(val)
    }
    FFIType::I8 => {
      let val: i8 = unsafe { cif.call::<i8>(codePointer, &args) };
      FFIValue::I8(val)
    }
    FFIType::I16 => {
      let val: i16 = unsafe { cif.call::<i16>(codePointer, &args) };
      FFIValue::I16(val)
    }
    FFIType::I32 => {
      let val: i32 = unsafe { cif.call::<i32>(codePointer, &args) };
      FFIValue::I32(val)
    }
    FFIType::I64 => {
      let val: i64 = unsafe { cif.call::<i64>(codePointer, &args) };
      FFIValue::I64(val)
    }
    FFIType::Isize => {
      let val: isize = unsafe { cif.call::<isize>(codePointer, &args) };
      FFIValue::Isize(val)
    }
    FFIType::F32 => {
      let val: f32 = unsafe { cif.call::<f32>(codePointer, &args) };
      FFIValue::F32(val)
    }
    FFIType::F64 => {
      let val: f64 = unsafe { cif.call::<f64>(codePointer, &args) };
      FFIValue::F64(val)
    }
    FFIType::Bool => {
      let val: u8 = unsafe { cif.call::<u8>(codePointer, &args) };
      FFIValue::Bool(val != 0)
    }
    FFIType::Pointer => {
      FFIValue::None // todo Не знаю, я пока что ограничил это, ведь пространства то разные.
    }
  };

  Ok(result)
}

// =================================================================================================

/// Управляет дочерним процессом-воркером
struct WorkerManager 
{
  /// Дочерний процесс-воркер
  childProcess: Child,
  /// Канал записи в STDIN воркера
  stdinHandle: ChildStdin,
  /// Канал чтения из STDOUT воркера
  stdoutHandle: ChildStdout,
  /// Аккумулятор для декодирования COBS-ответов
  cobsBuffer: DynamicCobsAccumulator,
}

impl WorkerManager 
{
  // ===============================================================================================
  
  /// Вспомогательный метод: Запуск дочернего процесса
  fn spawnWorker() -> Result<Child, String> 
  {
    // Получение пути к текущему исполняемому файлу процесса
    let executablePath: PathBuf = env::current_exe()
      .map_err(|e| format!("Failed to get exe path: {}", e))?;
    // Запускаем дубликат
    Command::new(executablePath)
      .arg("ffi")
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .spawn()
      .map_err(|e| format!("Failed to spawn worker: {}", e))
  }

  /// Вспомогательный метод: Остановка текущего дочернего процесса
  fn killChildProcess(&mut self) -> () 
  {
    let _ = self.childProcess.kill();
    let _ = self.childProcess.wait();
  }

  // ===============================================================================================

  /// Запускает новый экземпляр воркера (дочерний процесс)
  fn init() -> Result<Self, String> 
  {
    // Создание дочернего процесса воркера
    let mut childProcess: Child = Self::spawnWorker()?;

    // Забираем канал stdin у процесса (для отправки данных в воркер)
    let stdinHandle: ChildStdin = childProcess.stdin.take().ok_or("Failed to open stdin")?;
    // Забираем канал stdout у процесса (для чтения ответов воркера)
    let stdoutHandle: ChildStdout = childProcess.stdout.take().ok_or("Failed to open stdout")?;

    //
    Ok(Self {
      childProcess,
      stdinHandle,
      stdoutHandle,
      cobsBuffer: DynamicCobsAccumulator::new(),
    })
  }

  // ===============================================================================================

  /// Перезапускает воркера (убивает и создаёт заново)
  fn restart(&mut self) -> Result<(), String> 
  {
    self.killChildProcess();

    // Запуск нового процесса воркера
    let newChildProcess: Child = Self::spawnWorker()?;
    self.childProcess = newChildProcess;

    // Перепривязка stdin для нового процесса
    self.stdinHandle = self.childProcess.stdin.take().ok_or("Failed to open stdin")?;
    // Перепривязка stdout для нового процесса
    self.stdoutHandle = self.childProcess.stdout.take().ok_or("Failed to open stdout")?;
    // Сброс буфера декодирования (COBS)
    self.cobsBuffer = DynamicCobsAccumulator::new();

    Ok(())
  }

  // ===============================================================================================
  
  /// Отправляет запрос воркеру и ждёт ответ; всегда перезапускает воркер.
  pub fn callExternal(&mut self, libraryPath: &str, methodName: &str, args: &[FFIValue], resultType: FFIType) -> Result<FFIValue, String> 
  {
    let communicationResult: Result<FFIValue, String> = (|| 
    {
      // Запрос
      let request: WorkerRequest = WorkerRequest {
        libraryPath: libraryPath.to_string(),
        methodName: methodName.to_string(),
        args: args.to_vec(),
        resultType
      };

      // Сериализация запроса в COBS-байты
      let bytes: Vec<u8> = postcard::to_allocvec_cobs(&request)
        .map_err(|e| format!("Serialization error: {}", e))?;

      // Отправка запроса в воркер через stdin
      if let Err(e) = 
        self.stdinHandle
        .write_all(&bytes)
        .and_then(|_| self.stdinHandle
        .flush()) 
      {
        return Err(format!("Write error: {}", e));
      }

      // Срез байтов из буфера чтения stdout;
      // Содержит только реально прочитанные данные (без мусора хвоста массива).
      let mut rawBuffer: [u8; 8192] = [0u8; 8192];

      loop 
      { // Чтение данных из stdout воркера
        let bytesRead: usize = match self.stdoutHandle.read(&mut rawBuffer) {
          Ok(0) => return Err("Worker terminated unexpectedly".to_string()),
          Ok(n) => n,
          Err(e) => return Err(format!("Read error: {}", e)),
        };

        let window: &[u8] = &rawBuffer[..bytesRead];

        while !window.is_empty() 
        { // Пока в текущем куске stdout есть необработанные байты
          match self.cobsBuffer.feed::<WorkerResponse>(window) 
          {
            // Все данные уже обработаны буфером COBS
            DynamicFeedResult::Consumed => break,
            // Ошибка декодирования/десериализации COBS
            DynamicFeedResult::DeserError(e) => return Err(format!("Deserialization error: {:?}", e)),
            // Полный объект ответа восстановлен
            DynamicFeedResult::Success { data, remaining: _ } => 
            { // remaining не используется т.к. не нужны потоковые ответы.
              return if let Some(err) = data.error {
                Err(err)
              } else {
                data.result.ok_or("Empty result".to_string())
              };
            }
          };
          //
        }
      }
      //
    })();

    let _ = self.restart(); 
    // todo (feature)
    //  Перезапускаем всегда - чтобы не делать освобождение памяти;
    //  Иначе нужно использовать отчистку - которая разная будет и мы не сможем угадать её;
    //  А также мы не можем знать что вот он испортил мало памяти, а этот немного испортил -
    //  любой процесс должен быть изолирован.
    //  Но в целом, мы могли бы дать 2 worker и их хватало бы с головой на горячие участки.

    communicationResult
  }

  // ===============================================================================================
}

impl Drop for WorkerManager 
{
  /// Убивает воркера при уничтожении менеджера
  fn drop(&mut self) 
  {
    self.killChildProcess();
  }
}

// =================================================================================================

/// Глобальный синглтон WorkerManager, инициализируемый при первом обращении
/// и защищённый мьютексом для синхронной работы;
/// Используется OnceLock для защиты от падений при загрузке WorkerManager.
static FFIWorker: OnceLock<Result<Mutex<WorkerManager>, String>> = OnceLock::new();

/// Внешний интерфейс для вызова FFI-функции через ворке
pub fn callExternal(libraryPath: &str, methodName: &str, parametersTokens: &mut [Token], resultType: StructureType) -> Result<FFIValue, String>
{
  // Получаем worker
  let workerResult: &Result<Mutex<WorkerManager>, String> = FFIWorker.get_or_init(|| {
    WorkerManager::init()
      .map(Mutex::new)
      .map_err(|e| format!("Worker init error: {}", e))
  });
  
  // Обработка параметров
  println!("parametersTokens: {:?}",parametersTokens);
  let parameters: Vec<FFIValue> = parametersTokens
    .iter_mut()
    .map(FFIValue::try_from)  // автоматически использует реализацию TryFrom<&Token>
    .collect::<Result<Vec<_>, _>>()?; // при первой ошибке возвращаем её

  println!("FFI parameters (len={}):", parameters.len());
  for (i, val) in parameters.iter().enumerate() {
    println!("  [{}] = {:?}", i, val);
  }
  
  //
  match workerResult 
  {
    Ok(workerMutex) => 
    {
      let mut worker: MutexGuard<WorkerManager> = workerMutex.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
      // Делегирование вызова во внутренний механизм воркера
      worker.callExternal(
        libraryPath, 
        methodName, 
        &parameters, 
        FFIType::try_from(resultType)?
      )
    }
    Err(e) => Err(e.clone()),
  }
}

// =================================================================================================