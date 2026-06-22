use std::io::{Read, Stdin, Stdout, Write};
use std::sync::{LazyLock, Mutex};
use serde::{Deserialize, Serialize};
use libloading::Library;
use std::ffi::{CStr};
use std::os::raw::c_char;
use std::process::{Command, Stdio, Child, ChildStdin, ChildStdout};
use std::env;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use serde::de::DeserializeOwned;
use crate::parser::structure::ffi::workerManager::stdoutRedirect::StdoutRedirect;
// =================================================================================================

// Это реализация изоляции для FFI - чтобы мы могли безопасно обрабатывать такие пограничные места.
// 
// Она также всегда перезапускается, чтобы мы могли её использовать постоянно;
// А также синхронная, чтобы не нарушать поток и вести себя как обычный запуск чего-то.
// Собственно, ей и не надо иметь многопоточность или асинхронность.

// =================================================================================================

// todo
// Кросс-платформенное перенаправление stdout -> stderr; (Сейчас только linux)
//
// Проблема: Внутри worker-процесса библиотечная функция (например, printc) выводит данные в stdout, 
// а этот же канал используется для передачи COBS-закодированных ответов родительскому процессу. 
// В результате stdout содержит смесь произвольного текста и бинарных сообщений, что нарушает протокол 
// и вызывает ошибку десериализации (invalid type, expected value) при попытке распарсить ответ.
// 
// Решение: Временно перенаправлять stdout в stderr на время выполнения библиотечной функции, чтобы 
// изолировать её вывод от служебного канала связи. Это позволяет отправлять родителю 
// только чистые COBS-сообщения.

#[cfg(unix)]
mod stdoutRedirect 
{
  use std::os::fd::RawFd;
  use std::os::unix::io::AsRawFd;
  use libc;
  // ===============================================================================================

  pub struct StdoutRedirect 
  {
    /// Сохраняет оригинальный дескриптор stdout для последующего восстановления
    savedFd: i32,
  }

  impl StdoutRedirect 
  {
    /// Сохраняет текущий stdout и перенаправляет его в stderr
    pub fn new() -> Self 
    {
      let stdoutFd: RawFd = std::io::stdout().as_raw_fd();
      let stderrFd: RawFd = std::io::stderr().as_raw_fd();
      let saved = unsafe { libc::dup(stdoutFd) };
      unsafe { libc::dup2(stderrFd, stdoutFd) };
      StdoutRedirect { savedFd: saved }
    }
  }

  // ===============================================================================================
  
  impl Drop for StdoutRedirect 
  {
    /// Восстанавливает оригинальный stdout и закрывает сохранённый дескриптор
    fn drop(&mut self) 
    {
      unsafe {
        // Сброс буфера Си-рантайма перед возвратом дескриптора
        libc::fflush(std::ptr::null_mut()); // Сбрасывает все открытые C-потоки

        let stdoutFd: RawFd = std::io::stdout().as_raw_fd();
        libc::dup2(self.savedFd, stdoutFd);
        libc::close(self.savedFd);
      }
    }
  }
  
  // ===============================================================================================
}

// =================================================================================================

/// Результат обработки очередной порции данных аккумулятором COBS
pub enum DynamicFeedResult<'a, T> 
{
  /// Данные поглощены, ждём завершения сообщения
  Consumed,
  /// Ошибка десериализации COBS или postcard
  DeserError(String),
  /// Успешно декодировано сообщение с остатком данных
  Success { data: T, remaining: &'a [u8] }
}

/// Аккумулятор для накопления и декодирования COBS-сообщений из потока;
/// Он динамического размера, чтобы мы могли не нарушать работу FFI потоков извне;
/// Поскольку они бы не смогли изменить runtime и не писали бы это - 
/// мы должны дать им это из коробки.
pub struct DynamicCobsAccumulator 
{
  /// Буфер сырых байт COBS
  rawBuffer: Vec<u8>,
  /// Буфер раскодированных данных
  decodedBuffer: Vec<u8>
}

impl DynamicCobsAccumulator 
{
  // ===============================================================================================

  /// Создаёт аккумулятор с начальной ёмкостью буферов (по умолчанию 4096 байт).
  pub fn new() -> Self {
    Self::withCapacity(4096)
  }

  /// Создаёт аккумулятор с заданной начальной ёмкостью.
  pub fn withCapacity(capacity: usize) -> Self 
  {
    Self {
      rawBuffer: Vec::with_capacity(capacity),
      decodedBuffer: Vec::with_capacity(capacity),
    }
  }

  // ===============================================================================================

  /// Очищает оба буфера для подготовки к новому сообщению
  pub fn clear(&mut self) 
  {
    self.rawBuffer.clear();
    self.decodedBuffer.clear();
  }

  // ===============================================================================================

  /// Подаёт порцию данных, пытается извлечь законченное COBS-сообщение
  pub fn feed<'a, T: DeserializeOwned>(&mut self, data: &'a [u8]) -> DynamicFeedResult<'a, T> 
  {
    for (i, &byte) in data.iter().enumerate() 
    {
      if byte == 0x00 
      {
        if let Err(e) = Self::decodeCobs(&self.rawBuffer, &mut self.decodedBuffer) {
          self.clear();
          return DynamicFeedResult::DeserError(e);
        }
        self.rawBuffer.clear();

        match postcard::from_bytes::<T>(&self.decodedBuffer) 
        {
          Ok(val) => {
            return DynamicFeedResult::Success {
              data: val,
              remaining: &data[i + 1..],
            };
          }
          Err(e) => {
            self.clear();
            return DynamicFeedResult::DeserError(format!("{:?}", e));
          }
        }
      } else {
        // Буфер автоматически расширится при необходимости
        self.rawBuffer.push(byte);
      }
    }
    DynamicFeedResult::Consumed
  }

  // ===============================================================================================

  /// Декодирует COBS-последовательность в обычные байты
  fn decodeCobs(src: &[u8], dst: &mut Vec<u8>) -> Result<(), String> 
  {
    dst.clear();
    let mut srcIndex: usize = 0;

    while srcIndex < src.len() {
      let code: usize = src[srcIndex] as usize;
      if code == 0 {
        return Err("Invalid COBS: zero byte in data".into());
      }
      srcIndex += 1;
      let end: usize = srcIndex + code - 1;

      if end > src.len() {
        return Err("Invalid COBS: unexpected end".into());
      }

      dst.extend_from_slice(&src[srcIndex..end]);
      srcIndex = end;

      if code < 0xFF && srcIndex < src.len() {
        dst.push(0);
      }
    }

    Ok(())
  }
  
  // ===============================================================================================
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
  /// Аргументы todo Пока только первый используется
  args: Vec<String>,
}

/// Ответ воркера родителю
#[derive(Serialize, Deserialize)]
struct WorkerResponse 
{
  /// Успешный результат (строка)
  result: Option<String>,
  /// Сообщение об ошибке
  error: Option<String>,
}

/// Главный цикл воркера: чтение запросов, выполнение и отправка ответов
pub fn workerMain() 
{
  let mut inputStream: Stdin = std::io::stdin();
  let mut outputStream: Stdout = std::io::stdout();
  let mut rawBuffer: [u8; 8192] = [0u8; 8192];
  let mut cobsBuffer: DynamicCobsAccumulator = DynamicCobsAccumulator::new();

  loop {
    let bytesRead: usize = match inputStream.read(&mut rawBuffer) {
      Ok(n) => n,
      Err(_) => break,
    };

    if bytesRead == 0 {
      break;
    }

    let mut window: &[u8] = &rawBuffer[..bytesRead];

    while !window.is_empty() 
    {
      window = match cobsBuffer.feed::<WorkerRequest>(window) 
      {
        DynamicFeedResult::Consumed => break,
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
        DynamicFeedResult::Success { data, remaining } => 
        {
          // Перенаправляем stdout на время обработки запроса
          let response: WorkerResponse = {
            #[cfg(unix)]
            let _redirect: StdoutRedirect = StdoutRedirect::new();

            let catchResult: Result<Result<String, String>, Box<dyn std::any::Any + Send>> =
              catch_unwind(AssertUnwindSafe(|| processRequest(&data)));
            // Если паника перехвачена внутри worker, он остается жив и возвращает ошибку родителю;
            // Родитель получает штатный Err и не делает дорогостоящий restart.

            match catchResult {
              Ok(Ok(res)) => WorkerResponse { result: Some(res), error: None },
              Ok(Err(err)) => WorkerResponse { result: None, error: Some(err) },
              Err(_) => WorkerResponse {
                result: None,
                error: Some("FFI function panicked".to_string()),
              },
            }
          }; // здесь redirect уничтожается, stdout восстанавливается

          if let Ok(bytes) = postcard::to_allocvec_cobs(&response) {
            let _ = outputStream.write_all(&bytes);
            let _ = outputStream.flush();
          }
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
  let library: Library = unsafe {
    Library::new(&request.libraryPath)
      .map_err(|e| format!("Failed to load library: {}", e))?
  };

  type FunctionSignature = extern "C" fn(*const u8, usize) -> *mut u8;
  let functionPointer: FunctionSignature = unsafe {
    *library.get::<FunctionSignature>(request.methodName.as_bytes())
      .map_err(|e| format!("Failed to find function: {}", e))?
  };

  if request.args.is_empty() {
    return Err("No arguments provided".to_string());
  }

  let argument: &String = &request.args[0];
  let argumentBytes: &[u8] = argument.as_bytes();
  let pointer: *const u8 = argumentBytes.as_ptr();
  let length: usize = argumentBytes.len();

  // Вызов библиотечной функции – весь вывод в stdout пойдёт в stderr,
  // потому что мы перенаправили stdout перед вызовом processRequest.
  let resultPointer: *mut u8 = functionPointer(pointer, length);

  if resultPointer.is_null() {
    return Ok(String::new());
  }

  let cString: &CStr = unsafe { CStr::from_ptr(resultPointer as *const c_char) };
  let resultString: String = cString.to_str()
    .map_err(|e| format!("UTF-8 conversion error: {}", e))?
    .to_string();

  // НЕ освобождаем resultPointer — мы не знаем аллокатор библиотеки;
  // Мы также не можем вмешиваться в незнакомые библиотеки и делать внешние методы;
  // Это утечка памяти, но worker процесс перезапускается всегда.

  Ok(resultString)
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

  /// Запускает новый экземпляр воркера (дочерний процесс)
  fn init() -> Result<Self, String> 
  {
    let executablePath: PathBuf = env::current_exe()
      .map_err(|e| format!("Failed to get exe path: {}", e))?;

    let mut childProcess: Child = Command::new(executablePath)
      .arg("ffi")
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .spawn()
      .map_err(|e| format!("Failed to spawn worker: {}", e))?;

    let stdinHandle: ChildStdin = childProcess.stdin.take().ok_or("Failed to open stdin")?;
    let stdoutHandle: ChildStdout = childProcess.stdout.take().ok_or("Failed to open stdout")?;

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
    let _ = self.childProcess.kill();
    let _ = self.childProcess.wait();

    let executablePath: PathBuf = env::current_exe()
      .map_err(|e| format!("Failed to get exe path: {}", e))?;

    let newChild: Child = Command::new(executablePath)
      .arg("ffi")
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .spawn()
      .map_err(|e| format!("Failed to spawn worker: {}", e))?;

    self.childProcess = newChild;
    self.stdinHandle = self.childProcess.stdin.take().ok_or("Failed to open stdin")?;
    self.stdoutHandle = self.childProcess.stdout.take().ok_or("Failed to open stdout")?;
    self.cobsBuffer = DynamicCobsAccumulator::new();

    Ok(())
  }

  // ===============================================================================================
  
  /// Отправляет запрос воркеру и ждёт ответ; всегда перезапускает воркер.
  pub fn callExternal(&mut self, libraryPath: &str, methodName: &str, args: &[String]) -> Result<String, String> 
  {
    let communicationResult: Result<String, String> = (|| {
      let request = WorkerRequest {
        libraryPath: libraryPath.to_string(),
        methodName: methodName.to_string(),
        args: args.to_vec(),
      };

      let bytes: Vec<u8> = postcard::to_allocvec_cobs(&request)
        .map_err(|e| format!("Serialization error: {}", e))?;

      if let Err(e) = self.stdinHandle.write_all(&bytes).and_then(|_| self.stdinHandle.flush()) {
        return Err(format!("Write error: {}", e));
      }

      let mut rawBuffer: [u8; 8192] = [0u8; 8192];

      loop {
        let bytesRead: usize = match self.stdoutHandle.read(&mut rawBuffer) {
          Ok(0) => return Err("Worker terminated unexpectedly".to_string()),
          Ok(n) => n,
          Err(e) => return Err(format!("Read error: {}", e)),
        };

        let mut window: &[u8] = &rawBuffer[..bytesRead];

        while !window.is_empty() {
          window = match self.cobsBuffer.feed::<WorkerResponse>(window) {
            DynamicFeedResult::Consumed => break,
            DynamicFeedResult::DeserError(e) => return Err(format!("Deserialization error: {:?}", e)),
            DynamicFeedResult::Success { data, remaining: _ } => {
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

    let _ = self.restart(); // Перезапускаем всегда - чтобы не делать освобождение памяти

    communicationResult
  }

  // ===============================================================================================
}

impl Drop for WorkerManager 
{
  /// Убивает воркера при уничтожении менеджера
  fn drop(&mut self) 
  {
    let _ = self.childProcess.kill();
    let _ = self.childProcess.wait();
  }
}

// =================================================================================================

/// Глобальный синглтон WorkerManager, инициализируемый при первом обращении 
/// и защищённый мьютексом для синхронной работы.
static FFIWorker: LazyLock< Mutex<WorkerManager> > =
  LazyLock::new(|| {
    Mutex::new(
      WorkerManager::init().expect("Failed to initialize FFI worker")
    )
  });

/// Внешний интерфейс для вызова FFI-функции через ворке
pub fn callExternal(libraryPath: &str, methodName: &str, args: &[String]) -> Result<String, String> 
{
  let mut worker = FFIWorker.lock().map_err(|e| format!("Lock error: {}", e))?;
  worker.callExternal(libraryPath, methodName, args)
}

// =================================================================================================