use super::condition::replaceConditionExpression;
use super::search::searchStructure;
use crate::parser::bytes::Bytes;
use crate::parser::structure::ffi::bridge;
use crate::parser::structure::ffi::scopeStack;
use crate::parser::structure::primitives::{ArgsPrimitives, MethodsPrimitives};
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::tokenizer::tokenizer::readTokensSimple;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use crate::{_argc, _argv, _exit};
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
// =================================================================================================

// Обрабатывает последовательности из `Token`:
// 
// - Читает `Token` ⟶ вложения или одиночные;
//
// - Читает последовательности `Token` ⟶ `Line`;
//
// - Создаёт `Structure`.
//
// Всё это приводит к работе/вычислению этих записей как кода.
//
//
// `expression` - это набор `Token`, которой позволяет получить их результат.

// =================================================================================================

/// Эта функция обрабатывает линию с `return` и определяет куда пишется результат:
/// 
/// - `:=`: В ближайшую именованную [`Structure`] вверх по области видимости;
/// 
/// - `=`: В текущую [`Structure`].
/// 
fn searchReturnLine(
  line: &RwLockReadGuard<Line>,
  structureLink: Arc<RwLock<Structure>>
) -> bool
{
  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

  let mut lineTokens: Vec<Token> =
  { // Читаемая линия, на которой мы сейчас находимся.
    match line.tokens {
      Some(ref tokens) => tokens.clone(), // Токены линии, на которой мы сейчас находимся.
      None => return false // Если в линии нет токенов, то мы её не читаем.
    }
  };
  if lineTokens.is_empty() {
    return false // Если в линии нет токенов, то мы её не читаем.
  }

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  // Это `return` (`:=`) или результат блока (`=`)?
  let firstType: TokenType = *lineTokens[0].getDataType();
  if !matches!(firstType, TokenType::Equals | TokenType::Creation) {
    return false
  }

  // Удаляем первый токен (`=` или `:=`), чтобы он не мешался.
  lineTokens.remove(0);

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

  // Редактируемый родитель, поскольку мы собираемся присвоить значение его `result`.
  let newResultData: Token =
  {
    // Блок-выражение `= ? True { = 1 }`: `?` + условие заменяются результатом;
    //
    // Блоки выполняются без удержания блокировки на структуре.
    if let Some(replaced) = 
      replaceConditionExpression(&lineTokens, &line.lines, &structureLink) {
        lineTokens = replaced;
      }
    
    // Используем `expression`, чтобы получить результат выражения.
    let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
    structure.expression(&mut lineTokens)
  };

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

  drop(lineTokens); // Больше не нужно.
  
  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

  // Ищем куда писать `result`:
  //
  // - `:=` => Возврат результата только именной `Structure`.
  //           Поднимаемся через `.parent` до первого `name.is_some()`.
  //
  // - `=` => Пишем в текущую `structureLink` - т.е. текущую `Structure`.
  //
  let targetLink: Arc<RwLock<Structure>> = if firstType == TokenType::Creation
  {
    let mut currentLink: Arc<RwLock<Structure>> = structureLink.clone();
    loop
    {
      let current: RwLockReadGuard<Structure> = currentLink.read().unwrap();
      
      // Проверка на наличие имени у `Structure`.
      let hasName: bool = current.name.is_some();
      if hasName {
        break
      }
      
      // Берём нового родителя.
      let parent: Option< Arc<RwLock<Structure>> > = current.parent.clone();
      drop(current);
      
      match parent
      { // Поднимаемся выше, текущая `Structure` тоже была без имени.
        Some(parent) => currentLink = parent,
        // Дошли до самого верха, ничего выше `main` нет — `:=` игнорируется.
        None => return false
      };
    }
    currentLink
  } else {
    structureLink.clone()
  };

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

  let mut structure: RwLockWriteGuard<Structure> = targetLink.write().unwrap();

  // Структура ожидает какой-то тип в результате,
  // либо это может быть `TokenType::None`. Но мы просто будем менять `data`.

  if structure.result.is_some()
  { // Вариант, в котором результат ожидает возвращение определённого типа данных.
    if let Some(structureResult) = &mut structure.result
    { // Присваиваем новую data результату.
      structureResult.setData( newResultData.getData() );
    }
  } else
  { // Вариант, в котором тип результата был не указан;
    //
    // Используем `expression`, чтобы получить результат выражения;
    //
    // Присваиваем новый результат.
    structure.result = Some(newResultData);
  }

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

  // Всё успешно, это был результат.
  true

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
}

// =================================================================================================

/// Создает основную структуру программы.
pub fn createMainStructure(tokenizerLinesLinks: Vec< Arc<RwLock<Line>> >) -> Arc<RwLock<Structure>>
{
  // -----------------------------------------------------------------------------------------------

  // Основная структура; В неё вкладываются остальные;
  //
  // В эту структуру будут переданы стартовые параметры;
  //
  // Неизменяемая; Действует во время всей жизни программы.
  let mainStructureLink: Arc<RwLock<Structure>> =
    Arc::new(RwLock::new(Structure::new(
      None,
      StructureMut::Constant,
      StructureType::Primitive(PrimitiveType::Method),
      None,
      None
    )));
  let mut mainStructure: RwLockWriteGuard<Structure> = mainStructureLink.write().unwrap();

  // Присваиваем линии от Tokenizer; Это то что будет запущено как основная часть.
  mainStructure.lines = Some(tokenizerLinesLinks);

  // -----------------------------------------------------------------------------------------------
  // Параметры запуска (`argc`, `argv`).
  //
  // Значения приходят от запуска, поэтому у них, в отличие от остальных примитивов, есть линии.

  for (name, dataType) in ArgsPrimitives
  {
    let lines: Vec< Arc<RwLock<Line>> > = match *name
    {
      // Одна линия с одним токеном - сколько аргументов.
      "argc" => vec![
        Arc::new(RwLock::new(Line
        {
          tokens: Some(vec![
            Token::new(
              TokenType::UInt,
              Bytes::new(_argc.get().copied().unwrap_or(0).to_string())
            )
          ]),
          lines: None
        }))
      ],

      // По линии на каждый аргумент.
      "argv" => _argv.get().map(|values| values.as_slice()).unwrap_or(&[])
        .iter()
        .map(|arg: &String| Arc::new(RwLock::new(Line
        {
          tokens: Some(vec![
            Token::new(
              TokenType::String,
              Bytes::new( String::from(arg) )
            )
          ]),
          lines: None
        })))
        .collect(),

      _ => Vec::new()
    };

    mainStructure.pushStructure(
      Arc::new(RwLock::new(Structure::new(
        Some(String::from(*name)),
        StructureMut::Constant, // Неизменяемая.
        StructureType::Primitive(dataType.clone()),
        Some(lines),
        Some( mainStructureLink.clone() ) // Ссылаемся на родителя.
      )))
    );
  }

  // -----------------------------------------------------------------------------------------------
  // Примитивы (базовые структуры языка).

  for (name, dataType) in PrimitiveType::iter()
  {
    mainStructure.pushStructure(
      Arc::new(RwLock::new(Structure::new(
        Some(String::from(name)),
        StructureMut::Constant, // Неизменяемая.
        StructureType::Primitive(dataType.clone()),
        None, // Без значения — просто маркер.
        Some( mainStructureLink.clone() ) // Ссылаемся на родителя.
      )))
    );
  }

  for name in MethodsPrimitives
  {
    mainStructure.pushStructure(
      Arc::new(RwLock::new(Structure::new(
        Some(String::from(*name)),
        StructureMut::Constant, // Неизменяемая.
        StructureType::Primitive(PrimitiveType::Method),
        None, // Без значения — просто маркер.
        Some( mainStructureLink.clone() ) // Ссылаемся на родителя.
      )))
    );
  }
  
  // -----------------------------------------------------------------------------------------------
  drop(mainStructure);
  mainStructureLink
}

// =================================================================================================

/// Это основная функция для парсинга строк;
/// 
/// Она разделена на подготовительную часть, и часть запуска readLine().
fn parseLines(tokenizerLinesLinks: Vec< Arc<RwLock<Line>> >) -> Arc<RwLock<Structure>>
{ 
  // Подготовка к запуску.
  let mainStructureLink: Arc<RwLock<Structure>> = createMainStructure(tokenizerLinesLinks);

  // Подготовка закончена, читаем линии;
  //
  // Передаём ссылку на структуру и запускаем.
  readLines(mainStructureLink.clone());
  
  //
  mainStructureLink
}

// =================================================================================================

/// Эта функция занимается чтением блоков по ссылке на них.
//
// todo: исправить переполнение стека.
pub fn readLines(structureLink: Arc<RwLock<Structure>>) -> ()
{ // Получаем сколько линий вложено в структуру,
  // а также индекс чтения строк (у каждой структуры он свой, чтобы не путать чтение)

  // FFI-обёртка: если эта структура создана из `[ffi] { ... }` блока,
  // пушим слот в FFI scope-стек на всё время чтения её тела. Это
  // покрывает ВСЕ пути, по которым блок может быть выполнен:
  //   - анонимный `[ffi] { ... }` (выполняется прямо в searchStructure,
  //     но и оттуда можно звать readLines);
  //   - метод `[ffi] test() { ... }` (вызывается как функция в procedure.rs
  //     ⟶ readLines(calledStructureLink));
  //   - именованный блок `[ffi] name { ... }` (тоже вызывается).
  // Scope сам создаётся лениво в scopeStack::ensureFfiScope() при первом
  // FFI-вызове внутри блока — точно как просит issue #79/#84.
  let isFfi: bool = structureLink.read().unwrap().isFfiBlock;
  if isFfi {
    scopeStack::enterFfiBlock();
  }

  let linesLength: usize =
  {
    let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
    match &structure.lines
    {
      Some(lines) => lines.len(), // Если линии есть, значит получаем сколько их необходимо прочитать.
      None => return // Если нет линий, то не читаем.
    }
  };

  // Выполнение программы происходит до тех пор,
  // пока не будет всё прочитано, либо
  // пока не будет вызван _exitCode на true.
  let mut lineLink: Arc< RwLock<Line> >;

  while !_exit.load(Ordering::Relaxed)
  { // Если мы читаем строки, то создаём сразу ссылку на текущую линию.
    
    let lineIndex: usize = structureLink.read().unwrap().lineIndex.get();
    if lineIndex >= linesLength {
      break;
    }

    lineLink =
    { // Получаем её через чтение текущей структуры;
      // Берём линию по индексу линии (она точно будет, поскольку выше мы это проверили).
      let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
      let lines: &Vec< Arc<RwLock<Line>> > = structure.lines.as_ref().unwrap();
      lines[lineIndex].clone() // Клонируем нужную линию по индексу.
    };
    let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
    
    // После чего проверяем, если линия пустая на токены, то не читаем и идём дальше.
    if line.tokens.is_none()
    {
      structureLink.read().unwrap().lineIndex.set(lineIndex + 1);
      continue;
    }
    
    // Если всё хорошо, то начинаем читать через специальные функции.
    //
    // Ищем структуры.
    if !searchStructure(&line, structureLink.clone())
    { // Читаем return.

      if !searchReturnLine(&line, structureLink.clone())
      { // Ищем линейные выражения.

        let tokens: &mut Vec<Token> =
          &mut line
            .tokens.clone() // Клонируем токены, для сохранения возможности повторного запуска.
            .unwrap_or_default(); // todo плохо.
        drop(line);
        // Линия-оператор: результат никому не нужен (`lib.print(x)`).
        structureLink.read().unwrap()
          .expressionWith(tokens, &bridge::FfiExpect::Discard);
      }
    }
    else
    { // `searchStructure` обработал структуру (например `?`-блок).
      //
      // Если после этого у метода появился `result` — внутри блока был `:=`,
      // который поднялся до этого метода. Прерываем чтение.
      if structureLink.read().unwrap().result.is_some() {
        break
      }
    }

    { // Идём дальше
      let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
      structure.lineIndex.set(structure.lineIndex.get() + 1);
    }
  }

  // Сбрасываем указатель линий для текущей структуры на `0`
  // для того чтобы можно было запускать повторно.
  structureLink.read().unwrap().lineIndex.set(0);

  // FFI-обёртка: снимаем свой слот со стека.
  //
  // Внутри scope дропается (если был создан) вместе с библиотеками
  // и `AllocatedMemory`, привязанными к нему. Делаем это после сброса
  // lineIndex, чтобы можно было перезапустить блок — повторный запуск
  // снова пушнет свой слот.
  if isFfi {
    scopeStack::exitFfiBlock();
  }
}

// =================================================================================================

// todo Я вынес `runCode!` за место метода, который был тут;
//  И вот не понятно должны ли эти методы тоже быть в `testing`, хотя тот же `runCode!`
//  должен ли быт там? Логично что это часть основной инфраструктуры `parser`, поэтому оно
//  пока что так разделено. Это надо решить, возможно оно так и должно быть.

/// Запускает код и возвращает основную структуру.
pub fn runCodeBytes(buffer: &mut Vec<u8>) -> Arc<RwLock<Structure>>
{
  parseLines( readTokensSimple(buffer) )
}

/// Запускает `.rt` файл и возвращает основную структуру.
///
/// В отличие от обёртки `runCode!(r#"m := import("…")"#)` не опирается на
/// `import` — файл читается и выполняется напрямую, как entry в CLI.
pub fn runCodeFile(path: &str) -> Arc<RwLock<Structure>>
{
  assert!(
    path.ends_with(".rt"),
    "runCodeFile: expected a `.rt` file, got `{path}`"
  );
  let mut buffer: Vec<u8> = std::fs::read(path)
    .unwrap_or_else(|error| panic!("runCodeFile: read `{path}`: {error}"));
  let absolutePath: String = std::fs::canonicalize(path)
    .unwrap_or_else(|error| panic!("runCodeFile: canonicalize `{path}`: {error}"))
    .to_string_lossy().replace('\\', "/");
  let previousPath: Option<Arc<String>> =
    crate::_sourcePath.with(|sourcePath| sourcePath.borrow().clone());
  crate::_sourcePath.with(|sourcePath|
    *sourcePath.borrow_mut() = Some(Arc::new(absolutePath))
  );
  let result: Arc<RwLock<Structure>> = runCodeBytes(&mut buffer);
  crate::_sourcePath.with(|sourcePath|
    *sourcePath.borrow_mut() = previousPath
  );
  result
}

// =================================================================================================
