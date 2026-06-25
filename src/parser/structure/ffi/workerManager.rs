use std::io::{Read, Stdin, Stdout, Write};
use std::sync::{Mutex, MutexGuard, OnceLock};
use serde::{Deserialize, Serialize};
use libloading::Library;
use std::ffi::{CStr};
use std::os::raw::c_char;
use std::process::{Command, Stdio, Child, ChildStdin, ChildStdout};
use std::env;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use libffi::middle::Type;
use crate::parser::structure::ffi::dynamicCobsAccumulator::{DynamicCobsAccumulator, DynamicFeedResult};
use crate::parser::structure::ffi::stdoutRedirect::StdoutRedirect;
use crate::parser::structure::structureType::StructureType;
use crate::tokenizer::types::token::Token;
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
pub enum FFIValue 
{
  None, // Просто пустое значение
  //
  U8(u8),
  U16(u16),
  U32(u32),
  U64(u64),
  //
  I8(i8),
  I16(i16),
  I32(i32),
  I64(i64),
  //
  F32(f32),
  F64(f64),
  //
  Bool(bool),
  String(String), // Будет передана как C-строка (null-terminated)
                  // todo Удалить т.к будет передаваться по-другому
  Pointer(usize), // Сырой указатель
}

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
  //
  I8,
  I16,
  I32,
  I64,
  //
  F32,
  F64,
  //
  Bool,
  Pointer, // Сырой указатель
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

      StructureType::I8 => Ok(FFIType::I8),
      StructureType::I16 => Ok(FFIType::I16),
      StructureType::I32 => Ok(FFIType::I32),
      StructureType::I64 => Ok(FFIType::I64),

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

            let catchResult: Result<Result<String, String>, Box<dyn std::any::Any + Send>> =
              catch_unwind(AssertUnwindSafe(|| processRequest(&data)));
            // Если паника перехвачена внутри worker, он остается жив и возвращает ошибку родителю;
            // Родитель получает штатный Err и не делает дорогостоящий restart.

            match catchResult 
            {
              // Успешное выполнение запроса
              Ok(Ok(res)) => WorkerResponse { result: Some(FFIValue::String(res)), error: None }, // todo Заменить string на abi-ffi 9м)
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

/// Загружает библиотеку, вызывает FFI-функцию и возвращает результат
fn processRequest(request: &WorkerRequest) -> Result<String, String> 
{
  // Загрузка библиотеки
  let library: Library = unsafe {
    Library::new(&request.libraryPath)
      .map_err(|e| format!("Failed to load library: {}", e))?
  };

  // Загрузка метода
  type FunctionSignature = extern "C" fn(*const u8, usize) -> *mut u8;
  let functionPointer: FunctionSignature = unsafe {
    *library.get::<FunctionSignature>(request.methodName.as_bytes())
      .map_err(|e| format!("Failed to find function: {}", e))?
  };

  // Пустые параметры
  if request.args.is_empty() {
    return Err("No arguments provided".to_string());
  }

  // Берём первый аргумент строки
  let argument: &FFIValue = &request.args[0]; // todo Пока только первый используется
  // Извлекаем строку из FFIValue
  let (pointer, length) = match argument
  { // todo Заменить string на abi-ffi
    FFIValue::String(s) => (s.as_ptr(), s.len()),
    _ => return Err("First argument must be a string".to_string()),
  };

  // Вызов библиотечной функции – весь вывод в stdout пойдёт в stderr,
  // потому что мы перенаправили stdout перед вызовом processRequest.
  let resultPointer: *mut u8 = functionPointer(pointer, length); // todo не уверен в его типе

  // C указатель -> безопасная обёртка CStr (нул-терминированная строка)
  Ok(String::new()) // todo хз что тут
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
pub fn callExternal(libraryPath: &str, methodName: &str, parametersTokens: &[Token], resultType: StructureType) -> Result<FFIValue, String>
{
  // Получаем worker
  let workerResult: &Result<Mutex<WorkerManager>, String> = FFIWorker.get_or_init(|| {
    WorkerManager::init()
      .map(Mutex::new)
      .map_err(|e| format!("Worker init error: {}", e))
  });
  
  // Обработка параметров
  // Преобразуем токены в строки (все должны быть строковыми)
  let parametersStrings: Vec<FFIValue> = parametersTokens
    .iter()
    .filter_map(|t| {
      t.getData()
        .toString()
        .map(FFIValue::String) // todo Заменить string на abi-ffi
    })
    .collect();
  
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
        &parametersStrings, 
        FFIType::try_from(resultType)?
      )
    }
    Err(e) => Err(e.clone()),
  }
}

// =================================================================================================