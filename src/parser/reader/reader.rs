use super::result::searchReturnLine;
use super::search::searchStructure;
use crate::_exit;
use crate::parser::structure::ffi::bridge;
use crate::parser::structure::ffi::scopeStack;
use crate::parser::structure::structure::Structure;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

// Обрабатывает последовательности из `Token`:
// 
// - Читает `Token` ⟶ вложения или одиночные;
//
// - Читает последовательности из `Token` ⟶ `Line`;
//
// - Создаёт `Structure`.
//
// Всё это приводит к работе/вычислению этих записей как кода.
//
//
// `expression` - это набор `Token`, который позволяет получить их результат.
//
// `Поиск` в данном случае - нахождение последовательностей, которые парсер не просто читает,
// а запускает в процессе чтения. Т.е. они имеют семантическое значение.

// =================================================================================================

/// Эта функция занимается чтением [`Structure`] по ссылке на неё.
/// 
/// Это можно сравнить с чтением блока кода, потому что [`Structure`] -
/// это набор [`Line`] из [`Token`]. Поэтому он ищет те конструкции/выражения,
/// которые способен обработать и тем самым запускает последовательности токенов.
//
// todo Исправить переполнение стека.
pub fn readStructureLines(structureLink: Arc<RwLock<Structure>>) -> ()
{
  // -----------------------------------------------------------------------------------------------
  
  // FFI-обёртка: если эта структура создана из `[ffi] { ... }` блока,
  // пушим слот в FFI scope-стек на всё время чтения её тела.
  // 
  // Это покрывает все пути, по которым блок может быть выполнен:
  //  
  // - Анонимный `[ffi] { ... }`: Выполняется прямо в `searchStructure`,
  //   но и оттуда можно звать `readStructureLines`;
  //
  // - Метод `[ffi] test() { ... }`: Вызывается как функция в `procedure.rs`
  //   ⟶ `readStructureLines(calledStructureLink)`;
  //
  // - Именованный блок `[ffi] name { ... }`.
  //
  // `scope` сам создаётся лениво в `scopeStack::ensureFfiScope()`
  // при первом FFI-вызове внутри блока.
  //
  let isFfi: bool = structureLink.read().unwrap().isFfiBlock;
  if isFfi {
    scopeStack::enterFfiBlock();
  }

  // -----------------------------------------------------------------------------------------------

  // Получаем сколько линий вложено в структуру.
  let linesLength: usize =
  {
    let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
    match &structure.lines {
      // Если линии есть, значит получаем сколько их необходимо прочитать.
      Some(lines) => lines.len(),
      // Если нет линий, то не читаем.
      None => return
    }
  };

  // -----------------------------------------------------------------------------------------------

  // Выполнение программы происходит до тех пор:
  //
  // - Пока не будет прочитаны все линии;
  //
  // - Пока не будет вызван `_exitCode` на `true`.
  //
  let mut lineLink: Arc< RwLock<Line> >;
  while !_exit.load(Ordering::Relaxed)
  {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    
    let lineIndex: usize = structureLink.read().unwrap().lineIndex.get();
    // Если все линии будут прочитаны - мы выйдем;
    // Это также защита от чтения линий за списком.
    if lineIndex >= linesLength {
      break;
    }

    lineLink =
    { // Получаем ссылку на линию через чтение текущей структуры;
      // Берём линию по индексу линии (она точно будет, поскольку выше мы это проверили).
      let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
      let lines: &Vec< Arc<RwLock<Line>> > = structure.lines.as_ref().unwrap();
      lines[lineIndex].clone() // Клонируем нужную линию по индексу.
    };
    let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
    
    // Проверяем, если линия пустая на токены, то не читаем и идём дальше.
    if line.tokens.is_none()
    {
      structureLink.read().unwrap().lineIndex.set(lineIndex + 1);
      continue;
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    
    // Если всё хорошо, то начинаем читать линии через специальные функции.
    
    // Ищем структуры.
    if !searchStructure(&line, structureLink.clone())
    { 
      // Читаем return.
      if !searchReturnLine(&line, structureLink.clone())
      {
        // Ищем линейные выражения.
        let tokens: &mut Vec<Token> =
          &mut line
            .tokens.clone() // Клонируем токены, для сохранения возможности повторного запуска.
            // todo Кстати не понятно, нужно ли клонировать. Дело в том, что в нужных местах сами
            //  структуры уже клонированы. Например методы так делают. А обычному коду по идее
            //  это не к чему, ведь он запускается один раз. Хотя возможно если речь о ветвлениях
            //  или возвратах назад через `go`, то это необходимо.
            .unwrap_or_default(); // todo Плохо.
        drop(line);
        
        // Линия-оператор: результат никому не нужен (`lib.print(x)`).
        structureLink.read().unwrap()
          .expressionWith(tokens, &bridge::FfiExpect::Discard);
      }
    } else
    { // `searchStructure` обработал структуру (например `?`-блок).
      //
      // Если после этого у метода появился `result` — внутри блока был `:=`,
      // который поднялся до этого метода. Прерываем чтение.
      if structureLink.read().unwrap().result.is_some() {
        break
      }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    // Идём дальше
    let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
    structure.lineIndex.set(structure.lineIndex.get() + 1);

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  }

  // -----------------------------------------------------------------------------------------------

  // Сбрасываем указатель линий для текущей структуры на `0`
  // для того чтобы можно было запускать повторно.
  structureLink.read().unwrap().lineIndex.set(0);

  // FFI-обёртка: снимаем свой слот со стека.
  //
  // Внутри `scope` дропается (если был создан) вместе с библиотеками
  // и `AllocatedMemory`, привязанными к нему. Делаем это после сброса
  // `lineIndex`, чтобы можно было перезапустить блок — повторный запуск
  // снова пушнет свой слот.
  if isFfi {
    scopeStack::exitFfiBlock();
  }

  // -----------------------------------------------------------------------------------------------
}

// =================================================================================================
