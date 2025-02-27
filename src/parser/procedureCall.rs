use std::io;
use std::io::Write;
use std::process::Command;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread::sleep;
use std::time::Duration;
use crate::{_exit, _exitCode};
use crate::logger::formatPrint;
use crate::parser::{readLines, searchStructure};
use crate::parser::structure::parameters::Parameters;
use crate::parser::structure::Structure;
use crate::tokenizer::line::Line;
use crate::tokenizer::token::Token;

struct Procedure {}
impl Procedure
{
  // =========================================================================================
  /// Выводит несколько значений
  fn print(structure: &Structure, parameters: &Parameters)
  {
    match parameters.getAllExpressions(structure)
    {
      None => {}
      Some(parameters) =>
      {
        for p in parameters.iter()
        {
          formatPrint(&format!("{}", p.getData().unwrap_or_default()));
          io::stdout().flush().unwrap();
        }
        //
      }
    }
    //
  }
  // =========================================================================================
  /// Выводит несколько значений и \n в конце
  fn println(structure: &Structure, parameters: &Parameters)
  {
    Self::print(structure, parameters);
    formatPrint("\n");
    //
  }
}

impl Structure
{
  /// Запускает стандартные процедуры;
  /// Процедура - это такая структура, которая не возвращает результат.
  ///
  /// Но кроме того, запускает не стандартные методы;
  /// Из нестандартных методов, процедуры могут вернуть результат, в таком случае, их следует считать функциями.
  ///
  /// todo: вынести все стандартные варианты в отдельный модуль
  pub fn procedureCall(&self, structureName: &str, parameters: Parameters) -> ()
  {
    if structureName.starts_with(|c: char| c.is_lowercase())
    { // Если название в нижнем регистре - то это точно процедура
      match structureName
      { // Проверяем на сходство стандартных функций
        "println" =>
        { // println
          Procedure::println(self, &parameters);
        }
        // =========================================================================================
        "print" =>
        { // print
          Procedure::print(self, &parameters);
          //
        }
        // =========================================================================================
        "clear" =>
        { // clear
          let _ = Command::new("clear")
            .status(); // Игнорируем ошибки
          // todo: однако можно выдавать результат boolean при ошибке
        }
        // =========================================================================================
        "go" =>
        { // Запускаем линию выше заново
          match &self.parent
          {
            None => {}
            Some(parentLink) =>
            {
              // Получаем ссылку на линию
              let parent: RwLockReadGuard<'_, Structure> = parentLink.read().unwrap();
              let lineIndexBuffer: usize = parent.lineIndex;

              match &parent.lines
              {
                None => {}
                Some(lines) =>
                {
                  let (mut lineIndex, lineLink): (usize, Arc<RwLock<Line>>) =
                    (lineIndexBuffer, lines[lineIndexBuffer].clone());

                  let _ = drop(parent);

                  // Используем линию parent а также сам parent для нового запуска
                  searchStructure(
                    lineLink.clone(),
                    parentLink.clone(),
                    &mut lineIndex,
                  );
                  //
                }
              }
              //
            }
          }
          //
        }
        // =========================================================================================
        /*
        "ex" =>
        { // exit block up
          println!("ex");
        }
        */
        // =========================================================================================
        "sleep" =>
        { // sleep
          match parameters.getExpression(self, 0)
          {
            None => {}
            Some(p0) => unsafe
            {
              let valueNumber: u64 =
                p0
                  .getData().unwrap_or_default()
                  .parse::<u64>().unwrap_or_default(); // todo: depends on Value.rs
              match valueNumber > 0
              {
                true => { sleep(Duration::from_millis(valueNumber)); }
                false => {}
              }
              //
            }
          }
          //
        }
        // =========================================================================================
        "exit" =>
        { // Завершает программу с определённым кодом или кодом ошибки;
          match parameters.getExpression(self,0)
          {
            None => {}
            Some(p0) => unsafe
            {
              _exit = true;
              _exitCode =
                p0
                  .getData().unwrap_or_default()
                  .parse::<i32>().unwrap_or(1);
              //
            }
          }
          //
        }
        // =========================================================================================
        _ =>
        { // Если не было найдено совпадений среди стандартных процедур,
          // значит это нестандартный метод.
          match self.getStructureByName(&structureName)
          {
            None => {}
            Some(calledStructureLink) =>
            { // После получения такой нестандартной структуры по имени,
              // мы смотрим на её параметры
              {
                let calledStructure: RwLockReadGuard<'_, Structure> = calledStructureLink.read().unwrap();
                match parameters.getAllExpressions(self)
                {
                  None => {}
                  Some(parameters) =>
                  {
                    for (l, parameter) in parameters.iter().enumerate()
                    {
                      match &calledStructure.structures
                      {
                        None => {}
                        Some(calledStructureStructures) =>
                          {
                            let parameterResult: Token = self.expression(&mut vec![parameter.clone()]);
                            match calledStructureStructures.get(l)
                            {
                              None => {}
                              Some(parameterStructure) =>
                              {
                                let mut parameterStructure: RwLockWriteGuard<'_, Structure> = parameterStructure.write().unwrap();
                                // add new structure
                                parameterStructure.lines =
                                  Some(vec![
                                    Arc::new(
                                      RwLock::new(
                                        Line {
                                          tokens: vec![parameterResult],
                                          indent: 0,
                                          lines:  None,
                                          parent: None
                                        }
                                      ))
                                  ]);
                              }
                            }
                            //
                          }
                      }
                      //
                    }
                    //
                  }
                }
              }
              // Запускаем новую структуру
              readLines(calledStructureLink.clone());
            }
          }
        }
        // =========================================================================================
      }
      // Всё успешно, это была стандартная процедура
    } // Если название структуры не в нижнем регистре
  }
}