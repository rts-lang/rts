/* /main
  RTS init file

  Несколько моментов о коде:
  
  - match быстрее if; matches! быстрее простой проверки if
    при множествах значениях на одну проверку; match охватывает
    варианты ветвления более подробно
    
  - .len() == 0 быстрее is_empty()
  
  - Использование ссылок на данные быстрее клонирования,
    но не означает, что всегда нужно использовать ссылки;
    
  - Использование Arc+RwLock позволяет нескольким потокам 
    управлять чем-то без клонирования его самого;
    
  - На RwLock следует вовремя использовать drop(),
    не создавать переменные на них, а также использовать в
    замкнутых временных блоках.
    
  - Следует избегать флагов mut;
  
  - Следует указывать типы везде, где это возможно;
  
  - Объявление данных следует выносить за циклы.
  
  - Следует использовать временные блоки, чтобы оставались 
    только необходимые данные.
*/
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

#[macro_use]
extern crate lazy_static;

use std::{
  time::{Instant,Duration},
  env,
  io::{self, Read},
  fs::File
};

use crate::logger::*;

mod logger;
mod tokenizer;
mod parser;
mod packageApi;
// other globals
pub static mut _filePath: String = String::new(); // run file path
pub static mut _debugMode: bool = false;          // debug flag
// input & output
pub static mut _argc: usize       = 0;            // arhuments count
pub static mut _argv: Vec<String> = Vec::new();   // arguments vector

pub static mut _exitCode: i32 = 0;      // Значение которое вернёт программа при завершении;
pub static mut _exit:     bool = false; // Завершилась ли программа ?
// version
pub static _version: &str = "231206.0";
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
// main
#[tokio::main]
async fn main() -> io::Result<()> 
{
  let startTime: Instant = Instant::now();

  //
  use crate::tokenizer::*;
  use crate::parser::*;
  use crate::packageApi::packageApi;

  // args to key-values
  let mut args: (String, Vec<String>) = (String::new(), Vec::new());
  let input:    Vec<String> = env::args().collect();
  match input.len() > 1 
  {
    false => { help() }
    true => 
    {
      // first argument is treated as key, others as values
      let command: String      = input[1].clone();
      let values:  Vec<String> = input.iter().skip(2).cloned().collect();
      // store key and values in args vector
      args = (command.clone(), values.clone());
    }
  }
  
  // read key
  let mut runFile: bool = false;
  let mut buffer:  Vec<u8> = Vec::new();

  let valuesLength: usize = (args.1).len();

  match !args.0.is_empty() 
  {
    false => {}
    true => {
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
          packageApi(&args.1,valuesLength).await;
          logExit(0);
        },
        _ if (key == "run" || key == "drun") && valuesLength >= 1 =>
        { // run

          match key == "drun" 
          { // debug mode ?
            false => {}
            true  => unsafe{_debugMode = true;}
          }

          unsafe
          {
            _argc = valuesLength-1;
            _argv = (args.1)[1..].to_vec();
            _filePath = args.1[0].clone();
          }

          match unsafe{_debugMode} 
          {
            false => {}
            true  => { log("ok",&format!("Run [{}]",unsafe{&*_filePath})); }
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
  }

  match unsafe{_debugMode}
  {
    false => {}
    true => 
    {
      logSeparator("Arguments");
      log("ok","Debug mode");
    }
  }

  // run file
  match runFile 
  {
    true => 
    { // Обработка файла
      match unsafe{_debugMode} 
      {
        true  => { logSeparator(&format!("Running the file [{}] in debug mode",unsafe{&*_filePath})); }
        false => {}
      }
      // open file
      let mut file: File = match File::open(unsafe{&*_filePath}) 
      {
        Ok(file) => 
        {
          match unsafe{_debugMode} 
          {
            true  => { log("ok","Opening was successful"); }
            false => {}
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
          match unsafe{_debugMode} 
          {
            true  => { log("ok","Reading was successful"); }
            false => {}
          }
        }
        Err(_) => 
        {
          log("err","Unable to read file");
          logExit(1)
        }
      }
    }
    false =>
    { // Обработка скрипта
      // run script
      match unsafe{_debugMode}
      {
        false => {}
        true  => { logSeparator("Running the script in debug mode"); }
      }

      unsafe{ buffer = _filePath.clone().into_bytes(); }
    }
  }

  // проверяем что в конце был \n, если нет, то добавляем его
  match buffer.last() 
  {
    None => {}
    Some(&lastByte) => 
    {
      match lastByte != b'\n' 
      {
        false => {}
        true  => { buffer.push(b'\n'); }
      }
    }
  }

  // Начинаем чтение кода
  parseLines( readTokens(buffer, unsafe{_debugMode}) );
  
  match unsafe{_debugMode} 
  {
    // Замеры всего прошедшего времени работы
    false => {}
    true => 
    { 
      let endTime:  Instant  = Instant::now();
      let duration: Duration = endTime-startTime;
      log("ok",&format!("All duration [{:?}]",duration));
    }
  }
  // ** Для дополнительных тестов можно использовать hyperfine/perf

  // Возвращаем код завершения
  logExit(unsafe{_exitCode});
}
