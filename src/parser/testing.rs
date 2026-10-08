use crate::parser::parser::readLines;
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::StructureType;
use crate::tokenizer::tokenizer::readTokensSimple;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

// Общие помощники для tests: запуск кода RTS прямо из `cargo test`.
//
// Модуль подключён в `parser/mod.rs` под `#[cfg(test)]`, поэтому в обычную сборку не попадает.
//
// Проверка - макрос `checkStructure!`: одна строка на кейс, и сверяются ТОЛЬКО указанные грани.
// В `.rt` файлах проверки бывают разные: где-то `type` и `stype`, где-то только один
// из них, где-то ещё и значение. Макрос повторяет ровно тот набор, что проверяет файл.
//
// todo rewrite desc

// =================================================================================================

/// Выполняет код в отдельной структуре, как модуль в `include`, и возвращает её.
///
/// `RTS::run` не подходит: он пишет в общую `MainStructure`, и константа `a` из одного
/// теста остаётся в другом.
/// 
/// todo rewrite desc
pub(super) fn runCode(code: &str) -> Arc<RwLock<Structure>>
{
  let mut buffer: Vec<u8> = code.as_bytes().to_vec();
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

/// Считает выражение из токенов и возвращает токен-результат.
///
/// Для тестов Token/TokenType: структуры и StructureType тут не участвуют, поэтому нет `a = ...`.
pub(super) fn expressionOf(code: &str) -> Token
{
  let mut buffer: Vec<u8> = code.as_bytes().to_vec();
  let lines: Vec<Arc<RwLock<Line>>> = readTokensSimple(&mut buffer);
  
  let mut tokens: Vec<Token> = lines.first()
    .unwrap_or_else(|| panic!("Expression `{}` has no lines", code))
    .read().unwrap().tokens.clone()
    .unwrap_or_else(|| panic!("Expression `{}` has no tokens", code));
  
  let main: Structure = Structure::new(
    Some(String::from("main")),
    StructureMut::Constant,
    StructureType::Method,
    None,
    None
  );
  
  main.expression(&mut tokens)
}

/// Возвращает структуру по `name` из выполненного кода; если её нет - паника.
pub(super) fn getStructure(main: &Arc<RwLock<Structure>>, name: &str) -> Arc<RwLock<Structure>>
{
  main.read().unwrap().getStructureByName(name)
    .unwrap_or_else(|| panic!("Structure '{}' was not created", name))
}

/// Токен структуры: тип токена, строковое значение.
fn tokenOf(main: &Arc<RwLock<Structure>>, name: &str) -> (TokenType, String)
{
  let structure: Arc<RwLock<Structure>> = getStructure(main, name);
  let structureLock: RwLockReadGuard<Structure> = structure.read().unwrap();
  
  let line: RwLockReadGuard<Line> = structureLock.lines.as_ref()
    .unwrap_or_else(|| panic!("Structure '{}' has no lines", name))[0].read().unwrap();
  let token: &Token = line.tokens.as_ref()
    .unwrap_or_else(|| panic!("Structure '{}' has no tokens", name))
    .first().unwrap();
  
  (*token.getDataType(), token.getData().toString().unwrap_or_default())
}

// Грани возвращают `Err(текст)`, а не паникуют: `checkStructure!` паникует сам, а `tryCheckStructure!` отдаёт
// `Result`, поэтому ошибку сверки можно проверить без паники и без вывода в `--nocapture`.

/// `type` - тип токена.
pub(super) fn checkType(main: &Arc<RwLock<Structure>>, name: &str, code: &str, expected: &TokenType) -> Result<(), String>
{
  let (tokenType, _): (TokenType, String) = tokenOf(main, name);
  if tokenType == *expected { return Ok(()); }
  Err(format!(
    "check type in `{}` for '{}': expected '{}', got '{}'",
    code, name, expected.to_string(), tokenType.to_string()
  ))
}

/// `stype` - тип структуры.
pub(super) fn checkStype(main: &Arc<RwLock<Structure>>, name: &str, code: &str, expected: &StructureType) -> Result<(), String>
{
  let structure: Arc<RwLock<Structure>> = getStructure(main, name);
  let structureLock: RwLockReadGuard<Structure> = structure.read().unwrap();
  if structureLock.dataType == *expected { return Ok(()); }
  Err(format!(
    "check stype in `{}` for '{}': expected '{}', got '{}'",
    code, name, expected.to_string(), structureLock.dataType.to_string()
  ))
}

/// ``value` - значение структуры.
pub(super) fn checkValue(main: &Arc<RwLock<Structure>>, name: &str, code: &str, expected: &str) -> Result<(), String>
{
  let (_, data): (TokenType, String) = tokenOf(main, name);
  if data == expected { return Ok(()); }
  Err(format!(
    "check value in `{}` for '{}': expected '{}', got '{}'",
    code, name, expected, data
  ))
}

/// Печатает все грани структуры: type, stype, value.
///
/// Для отладки кейсов без assert.
/// 
/// В `cargo test` вывод виден только с `-- --nocapture`
/// (или `RUST_TEST_NOCAPTURE=1`).
pub(super) fn printStructure(main: &Arc<RwLock<Structure>>, name: &str, code: &str) -> ()
{
  let (tokenType, data): (TokenType, String) = tokenOf(main, name);
  let structure: Arc<RwLock<Structure>> = getStructure(main, name);
  let structureLock: RwLockReadGuard<Structure> = structure.read().unwrap();
  let stype: String = structureLock.dataType.to_string();
  println!(
    "show `{}` for '{}' | type={}, stype={}, value={}",
    code, name, tokenType.to_string(), stype, data
  );
}

// =================================================================================================

/// Проверка структуры одной строкой; выполняет код и сверяет ТОЛЬКО указанные грани.
///
/// Грани юзаются в любом наборе - ровно те, что проверяет `.rt` файл: где-то сверяются
/// `type` и `stype`, где-то только один из них, где-то ещё и значение. Порядок граней
/// фиксированный - `type` → `stype` → `value`, как в `.rt` файлах (`type(a) | stype(a) = a`):
///
///     checkStructure!("a: U8 = 300", a, type TokenType::UInt, stype StructureType::U8, value "255");
///     checkStructure!("a = 0.0", a, type TokenType::UFloat, stype StructureType::F32); // Без значения
///     checkStructure!("a = 10", a, stype StructureType::U8); // Только stype
///
/// Грани: `type` - тип токена (`type(a)` в `.rt`), `stype` - тип структуры (`stype(a)`),
/// `value` - значение (`{a}`). Имя структуры - идентификатор (`a`) или путь строкой
/// (`"a.b"`); каждый вызов исполняет свой код в отдельной структуре.
///
/// Паникует на первой неверной грани; `tryCheckStructure!` вместо паники возвращает `Err`.
macro_rules! checkStructure
{
  ($($arguments:tt)*) =>
  {
    if let Err(message) = $crate::parser::testing::tryCheckStructure!($($arguments)*)
    {
      panic!("{}", message);
    }
  };
}

pub(super) use checkStructure;

/// То же, что `tryCheckStructure!`, но возвращает `Result<(), String>`: первая неверная грань - `Err`.
macro_rules! tryCheckStructure
{
  // Имя структуры - идентификатор: `tryCheckStructure!("a = 10", a, value "10")`
  ($code:expr, $name:ident $(, type $t:expr)? $(, stype $s:expr)? $(, value $v:expr)? $(,)?) =>
  {{
    let main = $crate::parser::testing::runCode($code);
    (|| -> Result<(), String>
    {
      $(
        $crate::parser::testing::checkType(&main, stringify!($name), $code, &$t)?;
      )?
      $(
        $crate::parser::testing::checkStype(&main, stringify!($name), $code, &$s)?;
      )?
      $(
        $crate::parser::testing::checkValue(&main, stringify!($name), $code, &$v)?;
      )?
      Ok(())
    })()
  }};

  // Имя структуры - путь строкой: `tryChecktryCheckStructure!("a = 10", "a.b", value "10")`
  ($code:expr, $name:expr $(, type $t:expr)? $(, stype $s:expr)? $(, value $v:expr)? $(,)?) =>
  {{
    let main = $crate::parser::testing::runCode($code);
    (|| -> Result<(), String>
    {
      $(
        $crate::parser::testing::checkType(&main, $name, $code, &$t)?;
      )?
      $(
        $crate::parser::testing::checkStype(&main, $name, $code, &$s)?;
      )?
      $(
        $crate::parser::testing::checkValue(&main, $name, $code, &$v)?;
      )?
      Ok(())
    })()
  }};
}

#[allow(unused_imports)]
pub(super) use tryCheckStructure;

// =================================================================================================

/// Отладка без assert: выполняет код и печатает type, stype, value структуры.
///
/// Принимает те же грани, что и `checkStructure!`: чтобы переключиться между показом и
/// проверкой, достаточно поменять имя макроса. Печатает ВСЕ грани, а для указанных
/// ещё и расхождение с ожиданием (`≠`). Не паникует.
///
/// Вывод в консоль:
///
///     cargo test -- --nocapture --test-threads=1
///     cargo test testName -- --nocapture
///
///     showStructure!("a = 1e308 + 1e308", a, type TokenType::UFloat);
///     // show `a = 1e308 + 1e308` for 'a' | type=UFloat, stype=F64, value=1.7976931348623157e308
macro_rules! showStructure
{
  // Имя структуры - идентификатор: `showStructure!("a = 10", a, value "11")`
  ($code:expr, $name:ident $(, type $t:expr)? $(, stype $s:expr)? $(, value $v:expr)? $(,)?) =>
  {{
    let main = $crate::parser::testing::runCode($code);
    $crate::parser::testing::printStructure(&main, stringify!($name), $code);
    $(
      if let Err(message) = $crate::parser::testing::checkType(&main, stringify!($name), $code, &$t)
      { println!("  ≠ {}", message); }
    )?
    $(
      if let Err(message) = $crate::parser::testing::checkStype(&main, stringify!($name), $code, &$s)
      { println!("  ≠ {}", message); }
    )?
    $(
      if let Err(message) = $crate::parser::testing::checkValue(&main, stringify!($name), $code, &$v)
      { println!("  ≠ {}", message); }
    )?
  }};

  // Имя структуры - путь строкой: `showStructure!("a = 10", "a.b", value "11")`
  ($code:expr, $name:expr $(, type $t:expr)? $(, stype $s:expr)? $(, value $v:expr)? $(,)?) =>
  {{
    let main = $crate::parser::testing::runCode($code);
    $crate::parser::testing::printStructure(&main, $name, $code);
    $(
      if let Err(message) = $crate::parser::testing::checkType(&main, $name, $code, &$t)
      { println!("  ≠ {}", message); }
    )?
    $(
      if let Err(message) = $crate::parser::testing::checkStype(&main, $name, $code, &$s)
      { println!("  ≠ {}", message); }
    )?
    $(
      if let Err(message) = $crate::parser::testing::checkValue(&main, $name, $code, &$v)
      { println!("  ≠ {}", message); }
    )?
  }};
}

#[allow(unused_imports)]
pub(super) use showStructure;

// =================================================================================================

/// Смоук-тесты самого `tryCheckStructure!`: разные наборы граней компилируются и реально сверяются.
/// Проверка выражения из токенов (без `a = ...`): сверяются ТОЛЬКО указанные грани.
///
/// Грани: `type` - тип токена результата, `value` - его значение; порядок `type` → `value`.
///
///     checkExpression!("(1e308 + 1e308) - (1e308 + 1e308)", type TokenType::UFloat, value "0");
///     checkExpression!("1e309", value &f64::MAX.to_string()); // Только значение
///
/// Паникует на первой неверной грани.
macro_rules! checkExpression
{
  ($code:expr $(, type $t:expr)? $(, value $v:expr)? $(,)?) =>
  {{
    let code: &str = $code;
    let token: $crate::tokenizer::types::token::Token = $crate::parser::testing::expressionOf(code);
    $(
      let expectedType: $crate::tokenizer::types::tokenType::TokenType = $t;
      assert!(
        *token.getDataType() == expectedType,
        "check expression type in `{}`: expected '{}', got '{}'",
        code, expectedType.to_string(), token.getDataType().to_string()
      );
    )?
    $(
      let expectedValue: &str = &$v;
      let data: String = token.getData().toString().unwrap_or_default();
      assert!(
        data == expectedValue,
        "check expression value in `{}`: expected '{}', got '{}'",
        code, expectedValue, data
      );
    )?
  }};
}

pub(super) use checkExpression;

// =================================================================================================

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
    checkStructure!("a = 10", a, type TokenType::UInt);
  }

  /// Только `stype`.
  #[test]
  fn stypeOnly() -> ()
  {
    checkStructure!("a = 10", a, stype StructureType::U8);
  }

  /// Только `value`.
  #[test]
  fn valueOnly() -> ()
  {
    checkStructure!("a = 10", a, value "10");
  }

  /// `stype` и `value` без `type`.
  #[test]
  fn stypeAndValue() -> ()
  {
    checkStructure!("a = 10", a, stype StructureType::U8, value "10");
  }

  /// Все грани и висячая запятая.
  #[test]
  fn allFacetsTrailingComma() -> ()
  {
    checkStructure!("a = 10", a, type TokenType::UInt, stype StructureType::U8, value "10",);
  }

  /// Сверка ловит неверную грань, а не проходит молча. Через `tryCheckStructure!`: паники нет.
  #[test]
  fn catchesMismatch() -> ()
  {
    assert_eq!(
      tryCheckStructure!("a = 10", a, type TokenType::Int).unwrap_err(),
      "check type in `a = 10` for 'a': expected 'Int', got 'UInt'"
    );
    assert_eq!(
      tryCheckStructure!("a = 10", a, stype StructureType::U16).unwrap_err(),
      "check stype in `a = 10` for 'a': expected 'U16', got 'U8'"
    );
    assert_eq!(
      tryCheckStructure!("a = 10", a, value "11").unwrap_err(),
      "check value in `a = 10` for 'a': expected '11', got '10'"
    );
    // Верные грани - `Ok`.
    assert!(tryCheckStructure!("a = 10", a, type TokenType::UInt, stype StructureType::U8, value "10").is_ok());
  }

  /// `showStructure!` не паникует и печатает грани (смотреть с `-- --nocapture`).
  #[test]
  fn showStructure() -> ()
  {
    showStructure!("a = 10", a);
  }

  // ===============================================================================================
}

// =================================================================================================