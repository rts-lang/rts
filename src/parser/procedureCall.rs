use std::io;
use std::io::Write;
use std::process::Command;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::thread::sleep;
use std::time::Duration;
use crate::{_exit, _exitCode};
use crate::logger::formatPrint;
use crate::parser::{readLines, searchStructure};
use crate::parser::structure::parameters::Parameters;
use crate::parser::structure::Structure;
use crate::tokenizer::line::Line;

// Procedure =======================================================================================
/// Это набор базовых процедур
struct Procedure {}
impl Procedure
{
  // ===============================================================================================
  /// Выводит несколько значений
  /// Выводит несколько значений и \n в конце
  fn print(structure: &Structure, parameters: &Parameters, newline: bool)
  {
    match parameters.getAllExpressions(structure)
    { None => {} Some(parameters) =>
    {
      for p in parameters.iter()
      {
        formatPrint( p.getData().toString().unwrap_or_default().as_str() );
        match newline
        {
          false => {},
          true => formatPrint("\n")
        }
        io::stdout().flush().unwrap();
      }
    }}
  }
  // ===============================================================================================
  /// Отчищаем вывод
  ///
  /// todo Можно выдавать результат boolean при ошибке
  fn clear()
  {
    let _ = Command::new("clear")
      .status(); // Игнорируем ошибки
  }
  // ===============================================================================================
  /// Запускаем линию выше заново
  ///
  /// todo Должна принимать количество на которое поднимает наверх
  fn go(structure: &Structure)
  {
    match &structure.parent
    { None => {} Some(parentLink) =>
    { // Получаем ссылку на линию
      let parent: RwLockReadGuard<Structure> = parentLink.read().unwrap();
      let lineIndexBuffer: usize = parent.lineIndex;

      match &parent.lines
      { None => {} Some(lines) =>
      {
        let (mut lineIndex, lineLink): (usize, Arc<RwLock<Line>>) =
          (lineIndexBuffer, lines[lineIndexBuffer].clone());

        // Используем линию parent а также сам parent для нового запуска
        let _ = drop(parent);
        searchStructure(
          &lineLink.read().unwrap(),
          parentLink.clone(),
          &mut lineIndex,
        );
        //
      }}
      //
    }}
    //
  }
  // ===============================================================================================
  /* todo
  "ex" =>
  { // exit block up
    println!("ex");
  }
  */
  // ===============================================================================================
  /// Ожидает определённое количество ms
  fn sleep(structure: &Structure, parameters: &Parameters)
  {
    match parameters.getExpression(structure, 0)
    { None => {} Some(p0) => 
    {
      let valueNumber: u64 =
        p0
          .getData().toString().unwrap_or_default()
          .parse::<u64>().unwrap_or_default(); // todo: depends on Value.rs ?
      match valueNumber > 0
      {
        false => {}
        true => { sleep(Duration::from_millis(valueNumber)); }
      }
      //
    } }
    //
  }
  // ===============================================================================================
  /// Завершает чтение всех структур с определённым кодом или кодом ошибки
  fn exit(structure: &Structure, parameters: &Parameters)
  {
    match parameters.getExpression(structure,0)
    { None => {} Some(p0) => unsafe
    {
      _exit = true;
      _exitCode =
        p0
          .getData().toString().unwrap_or_default()
          .parse::<i32>().unwrap_or(1);
    }}
  }
}

// Structure =======================================================================================
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
    if structureName.starts_with(|c: char| c.is_lowercase()) // todo if -> match
    { // Если название в нижнем регистре - то это точно процедура
      match structureName
      { // Проверяем на сходство стандартных функций
        "println" => Procedure::print(self, &parameters, true),
        "print" => Procedure::print(self, &parameters, false),
        "clear" => Procedure::clear(),
        "go" => Procedure::go(self),
        "sleep" => Procedure::sleep(self, &parameters),
        "exit" => Procedure::exit(self, &parameters),
        // =========================================================================================
        _ =>
        { // Если не было найдено совпадений среди стандартных процедур,
          // значит это нестандартный метод.
          match self.getStructureByName(&structureName)
          { None => {} Some(calledStructureLink) =>
          { // Запускаем структуру
            // todo обработка параметров для процедуры
            //      https://github.com/rts-lang/rts/blob/a173ba17e660bf19e9a46c4445f05003f717dd8a/src/parser/structure.rs
            readLines(calledStructureLink);
          }}
        }
        // =========================================================================================
      }
      // Всё успешно, это была стандартная процедура
    } // Если название структуры не в нижнем регистре
  }
}