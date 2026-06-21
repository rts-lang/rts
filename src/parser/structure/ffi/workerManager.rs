use std::io::{Read, Stdin, Stdout, Write};
use std::sync::Mutex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use libloading::Library;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::process::{Command, Stdio, Child, ChildStdin, ChildStdout};
use std::env;
use std::path::PathBuf;
use postcard::accumulator::{CobsAccumulator, FeedResult};
// =================================================================================================

// Это реализация изоляции для FFI - чтобы мы могли безопасно обрабатывать такие пограничные места.

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

  pub struct StdoutRedirect {
    saved_fd: i32,
  }

  impl StdoutRedirect {
    pub fn new() -> Self {
      let stdoutFd: RawFd = std::io::stdout().as_raw_fd();
      let stderrFd: RawFd = std::io::stderr().as_raw_fd();
      let saved = unsafe { libc::dup(stdoutFd) };
      unsafe { libc::dup2(stderrFd, stdoutFd) };
      StdoutRedirect { saved_fd: saved }
    }
  }

  // ===============================================================================================
  
  extern "C" {
    static stdout: *mut libc::FILE;
  }

  impl Drop for StdoutRedirect 
  {
    fn drop(&mut self) 
    {
      unsafe {
        // Сброс буфера Си-рантайма перед возвратом дескриптора
        libc::fflush(stdout);

        let stdoutFd: RawFd = std::io::stdout().as_raw_fd();
        libc::dup2(self.saved_fd, stdoutFd);
        libc::close(self.saved_fd);
      }
    }
  }
  
  // ===============================================================================================
}

use stdoutRedirect::StdoutRedirect;

// =================================================================================================

#[derive(Serialize, Deserialize)]
struct WorkerRequest 
{
  libraryPath: String,
  methodName: String,
  args: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct WorkerResponse 
{
  result: Option<String>,
  error: Option<String>,
}

pub fn workerMain() 
{
  let mut inputStream: Stdin = std::io::stdin();
  let mut outputStream: Stdout = std::io::stdout();

  let mut rawBuffer: [u8; 1024] = [0u8; 1024];
  let mut cobsBuffer: CobsAccumulator<4096> = CobsAccumulator::new();

  loop {
    let bytesRead: usize = match inputStream.read(&mut rawBuffer) {
      Ok(n) => n,
      Err(_) => break,
    };

    if bytesRead == 0 {
      break;
    }

    let mut window: &[u8] = &rawBuffer[..bytesRead];
    while !window.is_empty() {
      window = match cobsBuffer.feed::<WorkerRequest>(window) 
      {
        FeedResult::Consumed => break,
        FeedResult::OverFull(remaining) => remaining,
        FeedResult::DeserError(_) => break,
        FeedResult::Success { data, remaining } => {
          // Перенаправляем stdout на время обработки запроса
          let response: WorkerResponse = {
            let redirect: StdoutRedirect = StdoutRedirect::new();
            match processRequest(&data) {
              Ok(res) => WorkerResponse { result: Some(res), error: None },
              Err(err) => WorkerResponse { result: None, error: Some(err) },
            }
          }; // здесь redirect уничтожается, stdout восстанавливается

          if let Ok(bytes) = postcard::to_allocvec_cobs(&response) {
            let _ = outputStream.write_all(&bytes);
            let _ = outputStream.flush();
          }
          remaining
        }
      };
    }
    //
  }
}

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

  // Вызов библиотечной функции – весь вывод в stdout теперь пойдёт в stderr,
  // потому что мы перенаправили stdout перед вызовом processRequest.
  let resultPointer: *mut u8 = functionPointer(pointer, length);

  if resultPointer.is_null() {
    Ok(String::new())
  } else {
    let cString: &CStr = unsafe { CStr::from_ptr(resultPointer as *const c_char) };
    let resultString: String = cString.to_str()
      .map_err(|e| format!("UTF-8 conversion error: {}", e))?
      .to_string();
    Ok(resultString)
  }
}

// =================================================================================================

struct WorkerManager 
{
  childProcess: Child,
  stdinHandle: ChildStdin,
  stdoutHandle: ChildStdout,
  cobsBuffer: CobsAccumulator<4096>,
}

impl WorkerManager 
{
  fn init() -> Result<Self, String> 
  {
    let executablePath: PathBuf = env::current_exe()
      .map_err(|e| format!("Failed to get exe path: {}", e))?;

    let mut childProcess: Child = Command::new(executablePath)
      .arg("worker")
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit()) // stderr будет содержать вывод библиотечных функций
      .spawn()
      .map_err(|e| format!("Failed to spawn worker: {}", e))?;

    let stdinHandle: ChildStdin = childProcess.stdin.take().ok_or("Failed to open stdin")?;
    let stdoutHandle: ChildStdout = childProcess.stdout.take().ok_or("Failed to open stdout")?;

    Ok(Self {
      childProcess,
      stdinHandle,
      stdoutHandle,
      cobsBuffer: CobsAccumulator::new(),
    })
  }

  fn restart(&mut self) -> Result<(), String> 
  {
    let _ = self.childProcess.kill();
    let _ = self.childProcess.wait();

    let executablePath: PathBuf = env::current_exe()
      .map_err(|e| format!("Failed to get exe path: {}", e))?;

    let newChild: Child = Command::new(executablePath)
      .arg("worker")
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .spawn()
      .map_err(|e| format!("Failed to spawn worker: {}", e))?;

    self.childProcess = newChild;
    self.stdinHandle = self.childProcess.stdin.take().ok_or("Failed to open stdin")?;
    self.stdoutHandle = self.childProcess.stdout.take().ok_or("Failed to open stdout")?;
    self.cobsBuffer = CobsAccumulator::new();

    Ok(())
  }

  pub fn callExternal(&mut self, libraryPath: &str, methodName: &str, args: &[String]) -> Result<String, String> 
  {
    let request: WorkerRequest = WorkerRequest {
      libraryPath: libraryPath.to_string(),
      methodName: methodName.to_string(),
      args: args.to_vec(),
    };

    let bytes: Vec<u8> = postcard::to_allocvec_cobs(&request)
      .map_err(|e| format!("Serialization error: {}", e))?;

    if let Err(e) = self.stdinHandle.write_all(&bytes).and_then(|_| self.stdinHandle.flush()) {
      let _ = self.restart();
      return Err(format!("Write error, worker restarted: {}", e));
    }

    let mut rawBuffer: [u8; 1024] = [0u8; 1024];
    loop {
      let bytesRead: usize = match self.stdoutHandle.read(&mut rawBuffer) {
        Ok(0) => {
          let _ = self.restart();
          return Err("Worker terminated unexpectedly, restarted".to_string());
        }
        Ok(n) => n,
        Err(e) => {
          let _ = self.restart();
          return Err(format!("Read error, worker restarted: {}", e));
        }
      };

      let mut window: &[u8] = &rawBuffer[..bytesRead];
      while !window.is_empty() {
        window = match self.cobsBuffer.feed::<WorkerResponse>(window) 
        {
          FeedResult::Consumed => break,
          FeedResult::OverFull(remaining) => remaining,
          FeedResult::DeserError(_) => {
            let _ = self.restart();
            return Err("Deserialization error, worker restarted".to_string());
          }
          FeedResult::Success { data, remaining } => {
            if let Some(err) = data.error {
              return Err(err);
            }
            return data.result.ok_or("Empty result".to_string());
          }
        };
      }
      //
    }
  }
}

impl Drop for WorkerManager {
  fn drop(&mut self) {
    let _ = self.childProcess.kill();
    let _ = self.childProcess.wait();
  }
}

// =================================================================================================

lazy_static! {
  static ref FFIWorker: Mutex<WorkerManager> = Mutex::new(WorkerManager::init().expect("Failed to initialize FFI worker"));
}

pub fn callExternal(libraryPath: &str, methodName: &str, args: &[String]) -> Result<String, String> 
{
  let mut worker = FFIWorker.lock().map_err(|e| format!("Lock error: {}", e))?;
  worker.callExternal(libraryPath, methodName, args)
}

// =================================================================================================