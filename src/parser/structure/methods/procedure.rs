use std::process::Command;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread::sleep;
use std::time::Duration;
use crate::{_exit, _exitCode};
use crate::parser::parser::{readLines, searchStructure};
use crate::parser::structure::structure::Structure;
use crate::tokenizer::types::line::Line;
#[cfg(not(target_family = "wasm"))]
use std::io;
#[cfg(not(target_family = "wasm"))]
use std::io::Write;
use std::ops::DerefMut;
#[cfg(not(target_family = "wasm"))]
use crate::logger::logger::formatPrint;
use crate::parser::structure::methods::tokensParameters::{TokensParameters};
use crate::tokenizer::types::token::Token;
// =================================================================================================

/// Это набор базовых процедур
struct Procedure;
impl Procedure
{
  // ===============================================================================================
  
  /// Выводит несколько значений;
  /// Выводит несколько значений и \n в конце.
  fn print(structure: &Structure, parameters: &TokensParameters, newline: bool)
  {
    #[cfg(not(target_family = "wasm"))]
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
  fn sleep(structure: &Structure, parameters: &TokensParameters)
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
  fn exit(structure: &Structure, parameters: &TokensParameters)
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
  
  // ===============================================================================================
}

// =================================================================================================

impl Structure
{
  // ===============================================================================================
  
  /// Запускает стандартные процедуры;
  /// Процедура - это такая структура, которая не возвращает результат.
  ///
  /// Но кроме того, запускает не стандартные методы;
  /// Из нестандартных методов, процедуры могут вернуть результат, в таком случае, их следует считать функциями.
  ///
  /// todo Вынести все стандартные варианты в отдельный модуль (теперь когда #68, надо ли?)
  /// 
  /// todo Кстати было замечено что 2 и последующие параметры могут обрабатывать не верно, а 1 норм.
  ///   Пример был когда у 2 параметра None - то его не видно, а 1 был виден, при проверках type/stype.
  pub fn procedureCall(&self, structureName: &str, parameters: TokensParameters) -> ()
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
        // -----------------------------------------------------------------------------------------
        _ => 
        { // Если не найдено совпадений среди стандартных процедур,
          // значит это нестандартный метод.
          match self.getStructureByName(&structureName) 
          {
            None => {}
            Some(calledStructureLink) => 
            {
              // 1. Вычисляем значения переданных аргументов в контексте вызывающей стороны;
              // Они здесь точно есть, но в Some мы оборачиваем чтобы не делать clone ниже при take.
              let mut parametersValues: Vec<Option<Token>> = parameters
                .getAllExpressions(self)
                .unwrap_or_default()
                .into_iter()
                .map(Some)
                .collect();

              // 2. Присваиваем значения параметрам (дочерним структурам) вызываемой функции
              // todo Они же потом не удаляются? Вообще по логике должна быть копия структуры,
              //  если он используется как метод? и там создание этого?
              {
                let calledStructure: RwLockReadGuard<Structure> = calledStructureLink.read().unwrap();
                
                let mut calledStructureStructuresLink: RwLockWriteGuard<Option< Vec< Arc<RwLock<Structure>> > >> = 
                  calledStructure.structures.write().unwrap();
                
                if let Some(calledStructureStructures) = calledStructureStructuresLink.deref_mut()
                {
                  for (idx, calledStructureStructureLink) in calledStructureStructures.iter_mut().enumerate() 
                  {
                    if idx < parametersValues.len() 
                    { // Проходит по количеству параметров, потому что первые структуры - это параметры.
                      let mut calledStructureStructure: RwLockWriteGuard<Structure> = 
                        calledStructureStructureLink.write().unwrap();
                      println!("    procedure name: {:?} = {:?}",calledStructureStructure.name,parametersValues[idx].clone());

                      // Забираем токен один раз
                      let mut token: Token = parametersValues[idx].take().unwrap(); // Здесь токен еще точно есть
                      
                      // Нормализируем под тип параметра
                      // todo:
                      //  Кстати не должен ли getAllExpressions сам делать приведение?
                      //  Много таких мест в коде с params.
                      Structure::normalizeToken(
                        &mut token, 
                        calledStructureStructure.dataType.clone()
                      );
                      
                      // Устанавливаем lines параметра как линию с одним токеном – переданным значением
                      calledStructureStructure.lines = Some(vec![
                        Arc::new(RwLock::new(Line {
                          tokens: Some(vec![token]),
                          indent: None,
                          lines: None,
                          parent: None,
                        }))
                      ]);
                    }
                  }
                  //
                }
              }

              // 3. Запускаем исполнение тела функции
              readLines(calledStructureLink);
            }
          }
        }
        // -----------------------------------------------------------------------------------------
      }
      // Всё успешно, это была стандартная процедура
    } // Если название структуры не в нижнем регистре
  }

  // ===============================================================================================
}

// =================================================================================================