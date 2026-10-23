mod lines;
mod names;
mod native;
// =================================================================================================
use crate::parser::parser::runCode;
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

// Общие помощники для тестов `linkExpression`.
//
// Тесты проверяют сам метод: код выполняется, а затем из него читается ссылка.
// Так видно, что именно делает разбор ссылки, а не то, как её вызывает парсер.

// =================================================================================================

/// Выполняет код и читает из корня ссылку `link`.
///
/// `link` - путь ссылки: `"a.b"` -> `["a", "b"]`.
/// `parameters` - `None` - это чтение значения, `Some` - это вызов метода.
pub(super) fn linkOf(code: &str, link: &str, parameters: Option< Vec<Token> >) -> Token
{
  let main: Arc<RwLock<Structure>> = runCode(code);
  let mut segments: Vec<String> = Structure::parseLink(link);
  let result: Token = main.read().unwrap().linkExpression(None, &mut segments, parameters);
  result
}

/// Выполняет код и читает ссылку `link` из структуры `name`.
///
/// Отличается от [`linkOf`] тем, что ссылка читается не из корня,
/// а из указанной структуры: её собственные имена видны, но не имена уровнем выше.
pub(super) fn linkOfIn(code: &str, name: &str, link: &str, parameters: Option< Vec<Token> >) -> Token
{
  let main: Arc<RwLock<Structure>> = runCode(code);
  let structure: Arc<RwLock<Structure>> =
    main.read().unwrap().getStructureByScope(name)
      .unwrap_or_else(|| panic!("Structure '{}' was not created", name));
  let structureForCall: Arc<RwLock<Structure>> = structure.clone();
  let mut segments: Vec<String> = Structure::parseLink(link);
  let structureGuard: RwLockReadGuard<Structure> = structure.read().unwrap();
  let result: Token = structureGuard.linkExpression(Some(structureForCall), &mut segments, parameters);
  result
}

/// Выполняет код и читает линию `line` структуры `name`.
///
/// Числовая ссылка - это номер линии в конкретной структуре,
/// поэтому структура-источник тут обязательна.
pub(super) fn linkOfLine(code: &str, name: &str, line: usize, parameters: Option< Vec<Token> >) -> Token
{
  let main: Arc<RwLock<Structure>> = runCode(code);
  let structure: Arc<RwLock<Structure>> =
    main.read().unwrap().getStructureByScope(name)
      .unwrap_or_else(|| panic!("Structure '{}' was not created", name));
  let structureForCall: Arc<RwLock<Structure>> = structure.clone();
  let mut segments: Vec<String> = vec![line.to_string()];
  let structureGuard: RwLockReadGuard<Structure> = structure.read().unwrap();
  let result: Token = structureGuard.linkExpression(Some(structureForCall), &mut segments, parameters);
  result
}

/// Выполняет код и читает линию `line` корневой структуры.
///
/// Корневая структура - это `main`: в ней лежат и примитивы, и структуры файла.
pub(super) fn linkOfRootLine(code: &str, line: usize, parameters: Option< Vec<Token> >) -> Token
{
  let main: Arc<RwLock<Structure>> = runCode(code);
  let mainForCall: Arc<RwLock<Structure>> = main.clone();
  let mut segments: Vec<String> = vec![line.to_string()];
  let mainGuard: RwLockReadGuard<Structure> = main.read().unwrap();
  let result: Token = mainGuard.linkExpression(Some(mainForCall), &mut segments, parameters);
  result
}

/// Проверяет тип токена-результата и его значение.
pub(super) fn checkToken(result: &Token, expectedType: &TokenType, expectedData: &str) -> ()
{
  let tokenType: TokenType = *result.getDataType();
  let data: String = result.getData().toString().unwrap_or_default();
  assert!(
    tokenType == *expectedType,
    "unexpected token type: got '{}', expected '{}'",
    tokenType.to_string(), expectedType.to_string()
  );
  assert!(
    data == expectedData,
    "unexpected token data: got '{}', expected '{}'",
    data, expectedData
  );
}

/// Проверяет, что токен-результат пустой.
pub(super) fn checkEmpty(result: &Token) -> ()
{
  checkToken(result, &TokenType::None, "");
}

/// Структура-указатель на динамическую библиотеку: путь лежит в первой линии.
///
/// `importNative` в тестах не работает - ему нужен запущенный файл,
/// поэтому такую структуру собираем руками.
pub(super) fn pointerStructure(name: &str, path: &str, parent: &Arc<RwLock<Structure>>) -> Arc<RwLock<Structure>>
{
  Arc::new(RwLock::new(Structure::new(
    Some(String::from(name)),
    StructureMut::Constant,
    StructureType::Primitive(PrimitiveType::Pointer),
    Some(vec![Arc::new(RwLock::new(Line {
      tokens: Some(vec![Token::new(TokenType::String, path)]),
      lines: None
    }))]),
    Some(parent.clone())
  )))
}

/// Линия из токенов - для ручной сборки структур.
pub(super) fn lineOf(tokens: Vec<Token>) -> Arc<RwLock<Line>>
{
  Arc::new(RwLock::new(Line
  {
    tokens: Some(tokens),
    lines: None
  }))
}

/// Создаёт структуру и кладёт её ребёнком в `parent`.
///
/// Для случаев, которые из кода не собрать: вложенные объявления
/// становятся структурами только после исполнения тела блока,
/// поэтому цепочки вида `a.b.c` тестируются на собранном дереве.
pub(super) fn childStructure(
  name: &str,
  dataType: StructureType,
  lines: Vec< Arc<RwLock<Line>> >,
  parent: &Arc<RwLock<Structure>>
) -> Arc<RwLock<Structure>>
{
  let structure: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
    Some(String::from(name)),
    StructureMut::Constant,
    dataType,
    Some(lines),
    Some(parent.clone())
  )));
  parent.read().unwrap().pushStructure(structure.clone());
  structure
}

/// Корневая структура - без родителя и без линий.
pub(super) fn emptyMain() -> Arc<RwLock<Structure>>
{
  Arc::new(RwLock::new(Structure::new(
    Some(String::from("main")),
    StructureMut::Constant,
    StructureType::Primitive(PrimitiveType::None),
    None,
    None
  )))
}
