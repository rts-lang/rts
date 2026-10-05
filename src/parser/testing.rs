use std::sync::{Arc, RwLock, RwLockReadGuard};
use crate::parser::parser::readLines;
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::StructureType;
use crate::tokenizer::tokenizer::readTokensSimple;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Общие помощники для tests: запуск кода RTS прямо из `cargo test`.
//
// Модуль подключён в `parser/mod.rs` под `#[cfg(test)]`, поэтому в обычную сборку не попадает.
//
// Проверка - макрос `check!`: одна строка на кейс, и сверяются ТОЛЬКО указанные грани.
// В `.rt` файлах проверки бывают разные: где-то `type` и `stype`, где-то только один
// из них, где-то ещё и значение. Макрос повторяет ровно тот набор, что проверяет файл.

/// Убирает общий отступ и крайние пустые строки: многострочный код в тесте можно
/// писать `r#"` с отступом, относительные отступы внутри кода сохраняются.
/// Однострочный код без отступа возвращается как есть.
pub fn dedent(code: &str) -> String
{
  let mut lines: Vec<&str> = code.split('\n').collect();
  if lines.first().is_some_and(|line| line.trim().is_empty()) { lines.remove(0); }
  if lines.last().is_some_and(|line| line.trim().is_empty()) { lines.pop(); }
  let minIndent: usize = lines.iter()
    .filter(|line| !line.trim().is_empty())
    .map(|line| line.len() - line.trim_start().len())
    .min()
    .unwrap_or(0);
  lines.iter()
    .map(|line| if line.trim().is_empty() { "" } else { &line[minIndent..] })
    .collect::<Vec<&str>>()
    .join("\n")
}

/// Выполняет код в отдельной структуре, как модуль в `include`, и возвращает её.
///
/// `RTS::run` не подходит: он пишет в общую `MainStructure`, и константа `a` из одного
/// теста остаётся в другом.
pub fn runCode(code: &str) -> Arc<RwLock<Structure>>
{
  let mut buffer: Vec<u8> = dedent(code).into_bytes();
  let main: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
    Some(String::from("main")),
    StructureMut::Constant,
    StructureType::Method,
    Some(readTokensSimple(&mut buffer)),
    None
  )));
  readLines(main.clone());
  main
}

/// Возвращает переменную `name` из выполненного кода; если её нет - паника.
pub fn getVariable(main: &Arc<RwLock<Structure>>, name: &str) -> Arc<RwLock<Structure>>
{
  main.read().unwrap().getStructureByName(name)
    .unwrap_or_else(|| panic!("Variable '{}' was not created", name))
}

/// Токен переменной: (тип токена, строковое значение).
fn tokenOf(main: &Arc<RwLock<Structure>>, name: &str) -> (TokenType, String)
{
  let variable: Arc<RwLock<Structure>> = getVariable(main, name);
  let structure: RwLockReadGuard<Structure> = variable.read().unwrap();
  let line: RwLockReadGuard<Line> = structure.lines.as_ref()
    .unwrap_or_else(|| panic!("Variable '{}' has no lines", name))[0].read().unwrap();
  let token: &Token = line.tokens.as_ref()
    .unwrap_or_else(|| panic!("Variable '{}' has no tokens", name))
    .first().unwrap();
  (*token.getDataType(), token.getData().toString().unwrap_or_default())
}

/// `type` - тип токена переменной, как `type(a)` в `.rt` (TokenType::to_string()).
pub fn checkType(main: &Arc<RwLock<Structure>>, name: &str, code: &str, expected: &TokenType) -> ()
{
  let (tokenType, _): (TokenType, String) = tokenOf(main, name);
  assert!(
    tokenType == *expected,
    "check type in `{}` for '{}': expected '{}', got '{}'",
    code, name, expected.to_string(), tokenType.to_string()
  );
}

/// `stype` - тип структуры переменной, как `stype(a)` в `.rt` (StructureType::to_string()).
pub fn checkStype(main: &Arc<RwLock<Structure>>, name: &str, code: &str, expected: &StructureType) -> ()
{
  let variable: Arc<RwLock<Structure>> = getVariable(main, name);
  let structure: RwLockReadGuard<Structure> = variable.read().unwrap();
  assert!(
    structure.dataType == *expected,
    "check stype in `{}` for '{}': expected '{}', got '{}'",
    code, name, expected.to_string(), structure.dataType.to_string()
  );
}

/// `val` - значение переменной, как `{a}` в `.rt`.
pub fn checkValue(main: &Arc<RwLock<Structure>>, name: &str, code: &str, expected: &str) -> ()
{
  let (_, data): (TokenType, String) = tokenOf(main, name);
  assert!(
    data == expected,
    "check value in `{}` for '{}': expected '{}', got '{}'",
    code, name, expected, data
  );
}

// =================================================================================================

/// Проверка переменной одной строкой; выполняет код и сверяет ТОЛЬКО указанные грани.
///
/// Грани юзаются в любом наборе - ровно те, что проверяет `.rt` файл: где-то сверяются
/// `type` и `stype`, где-то только один из них, где-то ещё и значение. Порядок граней
/// фиксированный - `type` → `stype` → `val`, как в `.rt` файлах (`type(a) | stype(a) = a`):
///
///     check!("a: U8 = 300", a, type TokenType::UInt, stype StructureType::U8, val "255");
///     check!("a = 0.0", a, type TokenType::UFloat, stype StructureType::F32); // Без значения
///     check!("a = 10", a, stype StructureType::U8); // Только stype
///
/// Грани: `type` - тип токена (`type(a)` в `.rt`), `stype` - тип структуры (`stype(a)`),
/// `val` - значение (`{a}`). Имя переменной - идентификатор (`a`) или путь строкой
/// (`"a.b"`); каждый вызов исполняет свой код в отдельной структуре.
macro_rules! check
{
  // Имя переменной - идентификатор: `check!("a = 10", a, val "10")`
  ($code:expr, $name:ident $(, type $t:expr)? $(, stype $s:expr)? $(, val $v:expr)? $(,)?) =>
  {{
    let main = $crate::parser::testing::runCode($code);
    $(
      $crate::parser::testing::checkType(&main, stringify!($name), $code, &$t);
    )?
    $(
      $crate::parser::testing::checkStype(&main, stringify!($name), $code, &$s);
    )?
    $(
      $crate::parser::testing::checkValue(&main, stringify!($name), $code, &$v);
    )?
  }};

  // Имя переменной - путь строкой: `check!("a = 10", "a.b", val "10")`
  ($code:expr, $name:expr $(, type $t:expr)? $(, stype $s:expr)? $(, val $v:expr)? $(,)?) =>
  {{
    let main = $crate::parser::testing::runCode($code);
    $(
      $crate::parser::testing::checkType(&main, $name, $code, &$t);
    )?
    $(
      $crate::parser::testing::checkStype(&main, $name, $code, &$s);
    )?
    $(
      $crate::parser::testing::checkValue(&main, $name, $code, &$v);
    )?
  }};
}

pub(crate) use check;

// =================================================================================================

/// Смоук-тесты самого `check!`: разные наборы граней компилируются и реально сверяются.
#[cfg(test)]
mod tests
{
  use crate::parser::structure::structureType::StructureType;
  use crate::tokenizer::types::tokenType::TokenType;
  // ===============================================================================================

  /// Только `type` - как в `types1.rt` (`println(type(10+10))`).
  #[test]
  fn typeOnly() -> ()
  {
    check!("a = 10", a, type TokenType::UInt);
  }

  /// Только `stype`.
  #[test]
  fn stypeOnly() -> ()
  {
    check!("a = 10", a, stype StructureType::U8);
  }

  /// Только `val`.
  #[test]
  fn valueOnly() -> ()
  {
    check!("a = 10", a, val "10");
  }

  /// `stype` и `val` без `type`.
  #[test]
  fn stypeAndValue() -> ()
  {
    check!("a = 10", a, stype StructureType::U8, val "10");
  }

  /// Все грани и висячая запятая.
  #[test]
  fn allFacetsTrailingComma() -> ()
  {
    check!("a = 10", a, type TokenType::UInt, stype StructureType::U8, val "10",);
  }

  /// Многострочный `r#"` с отступом: общий отступ снимается.
  #[test]
  fn multilineIndented() -> ()
  {
    check!(
      r#"
        x = 10
        n = -10
        z = 0
        a = x / z
      "#,
      a,
      type TokenType::UInt,
      val "10"
    );
  }

  /// `dedent` сохраняет относительные отступы и не трогает однострочный код.
  #[test]
  fn dedentKeepsRelativeIndent() -> ()
  {
    assert_eq!(super::dedent("\n    a\n      b\n\n    c\n  "), "a\n  b\n\nc");
    assert_eq!(super::dedent("a = 10"), "a = 10");
    assert_eq!(super::dedent("x = 1\na = x"), "x = 1\na = x");
  }

  /// Сверка ловит неверное значение, а не проходит молча.
  #[test]
  #[should_panic(expected = "check value in `a = 10` for 'a'")]
  fn catchesMismatch() -> ()
  {
    check!("a = 10", a, val "11");
  }

  // ===============================================================================================
}

// =================================================================================================