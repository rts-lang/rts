#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

// Сборка с флагом `analyzer` не может собирать бинарный файл.
// Она может только `wasm` и `lib` виды сборки.
#[cfg(feature = "analyzer")]
compile_error!("This binary cannot be compiled with the 'analyzer' feature enabled. Please build the library crate instead.");

include!("prelude.rs");

// =================================================================================================

use std::{
  time::{Instant,Duration},
  env,
  io::{self, Read},
  fs::File
};
use crate::logger::logger::{log, logExit, logSeparator};
use crate::parser::parser::parseLines;
use crate::tokenizer::tokenizer::readTokensSimple;

// todo удалить mods
mod tokenizer;
mod parser;
mod logger;
mod packages;

// =================================================================================================

// help
fn help() -> ()
{
  // todo: description
  log("ok","version");
  log("ok","<empty>");
  log("ok","help");
  log("ok","drun");
  log("ok","drun <filename>");
  log("ok","drun \"<script>\"");
  log("ok","run");
  log("ok","run <filename>");
  log("ok","run \"<script>\"");
  log("ok","package <empty>");
  log("ok","package help");
  log("ok","package local");
  log("ok","package local-delete");
  logExit(0);
}

/// Основной метод для бинарника RTS;
/// Позволяет работать с параметрами;
/// Обладает режимом чтения файла или скрипта из строки;
fn main() -> io::Result<()> 
{
  let startTime: Instant = Instant::now();

  // args to key-values
  let mut args: (String, Vec<String>) = (String::new(), Vec::new());
  let input: Vec<String> = env::args().collect();
  
  if input.len() > 1 
  {
    // first argument is treated as key, others as values
    let command: String = input[1].clone();
    let values:  Vec<String> = input.iter().skip(2).cloned().collect();
    
    // store key and values in args vector
    args = (command, values);
  } else { help() }
  
  // read key
  let mut runFile: bool = false;
  let mut buffer: Vec<u8> = Vec::new();

  let valuesLength: usize = (args.1).len();

  if !args.0.is_empty() 
  {
    let key: &str = args.0.as_str();
    match key
    {
      "version" => 
      { // get version
        log("ok", &format!("RTS v{}", _version));
        logExit(0);
      }
      "help" => help(),
      "package" =>
      { // package
        // packageApi(&args.1,valuesLength).await; todo
        logExit(0);
      },
      _ if (key == "run" || key == "drun") && valuesLength >= 1 =>
      { // run

        if key == "drun" 
        { // debug mode ?
          unsafe{_debugMode = true;}
        }

        unsafe{
          _argc = valuesLength-1;
          _argv = args.1[1..].to_vec();
          _filePath = args.1[0].clone();
        }

        if unsafe{_debugMode} {
          log("ok",&format!("Run [{}]",unsafe{&*_filePath}));
        }

        unsafe{
          // Проверяем, что мы запускаем файл или скрипт;
          // todo: В данном случае это является временным решением,
          //       чтобы сохранить run и drun, а также разделить скрипт и файлы;
          let filePathEnd: String =
            _filePath
              .chars().rev().take(3)
              .collect::<Vec<_>>().iter().rev().collect();
          runFile = filePathEnd == ".rt";
        }

        // run package
        // todo: run package
      }
      _ => {
        log("err","Use [rts help] to get help");
        logExit(1)
      }
    }
  }

  if unsafe{_debugMode}
  {
    logSeparator("Arguments");
    log("ok","Debug mode");
  }

  // run file
  if runFile 
  { // Обработка файла
    
    if unsafe{_debugMode} {
      logSeparator(&format!("Running the file [{}] in debug mode",unsafe{&*_filePath}));
    }
    
    // open file
    let mut file: File = match File::open(unsafe{&*_filePath}) 
    {
      Ok(file) => 
      {
        if unsafe{_debugMode} 
        {
          log("ok","Opening was successful");
        }
        file
      },
      Err(_) => 
      {
        log("err","Unable to opening file");
        logExit(1)
      }
    };
    
    // read file into buffer
    match file.read_to_end(&mut buffer) 
    {
      Ok(_) => 
      {
        if unsafe{_debugMode} 
        {
          log("ok","Reading was successful");
        }
      }
      Err(_) => 
      {
        log("err","Unable to read file");
        logExit(1)
      }
    }
  } else
  { // Обработка скрипта
    // run script
    if unsafe{_debugMode} { 
      logSeparator("Running the script in debug mode"); 
    }

    unsafe{ buffer = _filePath.clone().into_bytes(); }
  }


  // Начинаем чтение кода
  parseLines( readTokensSimple(&mut buffer, unsafe{_debugMode}) );

  if unsafe{_debugMode} 
  { // Замеры всего прошедшего времени работы
    let endTime:  Instant  = Instant::now();
    let duration: Duration = endTime-startTime;
    log("ok",&format!("All duration [{:?}]",duration));
  }
  // ** Для дополнительных тестов можно использовать hyperfine/perf

  // Возвращаем код завершения
  logExit(unsafe{_exitCode});
}
