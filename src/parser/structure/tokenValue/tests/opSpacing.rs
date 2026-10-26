use crate::parser::testing::checkStructure;
use crate::tokenizer::testing::checkTokensTypes;
use crate::tokenizer::tokenizer::readTokensSimple;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Тесты на пробелы между знаками операторов.
//
//
// Пробелы стираются везде, кроме одного места: между двумя знаками оператора.
//
// Два знака склеиваются в один оператор только вплотную: `!=` - один оператор,
//
// `! =` - два разных оператора.
//
//
// Это нужно, чтобы выражение `a - -b` (бинарный `-` и унарный `-`) не слипалось
// в составной оператор, который имеет другой смысл.
//
//
// todo Я еще не определился полностью должно ли это быть здесь: потому что тут половина работы
//  из токенайзера и половина работы из парсера. Хотя у них единая задача для тестирования,
//  но в целом это по сути можно оставить в парсере т.к. это ближе к нему?

// =================================================================================================

/// Знаки через пробел остаются раздельными операторами.
#[test]
fn spacedSignsStaySeparate() -> ()
{
  checkTokensTypes(
    "a - -b",
    &[TokenType::Word, TokenType::Minus, TokenType::Minus, TokenType::Word]
  );
  checkTokensTypes(
    "a- -b",
    &[TokenType::Word, TokenType::Minus, TokenType::Minus, TokenType::Word]
  );
  checkTokensTypes(
    "a + +b",
    &[TokenType::Word, TokenType::Plus, TokenType::Plus, TokenType::Word]
  );
  checkTokensTypes(
    "a ! = b",
    &[TokenType::Word, TokenType::Not, TokenType::Equals, TokenType::Word]
  );
  checkTokensTypes(
    "a : = b",
    &[TokenType::Word, TokenType::Colon, TokenType::Equals, TokenType::Word]
  );
  checkTokensTypes(
    "a < = b",
    &[TokenType::Word, TokenType::LessThan, TokenType::Equals, TokenType::Word]
  );
  checkTokensTypes(
    "a > = b",
    &[TokenType::Word, TokenType::GreaterThan, TokenType::Equals, TokenType::Word]
  );
}

/// Знаки вплотную по-прежнему склеиваются в один оператор.
#[test]
fn adjacentSignsMerge() -> ()
{
  checkTokensTypes(
    "a != b",
    &[TokenType::Word, TokenType::NotEquals, TokenType::Word]
  );
  checkTokensTypes(
    "a:=b",
    &[TokenType::Word, TokenType::Creation, TokenType::Word]
  );
  checkTokensTypes(
    "a <= b",
    &[TokenType::Word, TokenType::LessThanOrEquals, TokenType::Word]
  );
  checkTokensTypes(
    "a >= b",
    &[TokenType::Word, TokenType::GreaterThanOrEquals, TokenType::Word]
  );
  checkTokensTypes(
    "a -> b",
    &[TokenType::Word, TokenType::Pointer, TokenType::Word]
  );
}

/// Пробелы вокруг знаков по-прежнему стираются: смысл тот же.
#[test]
fn spacesAroundSignsAreErased() -> ()
{
  checkTokensTypes(
    "a-b",
    &[TokenType::Word, TokenType::Minus, TokenType::Word]
  );
  checkTokensTypes(
    "a -b",
    &[TokenType::Word, TokenType::Minus, TokenType::Word]
  );
  checkTokensTypes(
    "a - b",
    &[TokenType::Word, TokenType::Minus, TokenType::Word]
  );
  checkTokensTypes(
    "a  !=  b",
    &[TokenType::Word, TokenType::NotEquals, TokenType::Word]
  );
}

/// Перенос строки между знаками не склеивает их: оператор в конце строки
/// просто продолжает выражение, а следующий знак читается заново.
#[test]
fn newlineBetweenSignsDoesNotMerge() -> ()
{
  checkTokensTypes(
    "a -\n-b",
    &[TokenType::Word, TokenType::Minus, TokenType::Minus, TokenType::Word]
  );
  checkTokensTypes(
    "a !\n= b",
    &[TokenType::Word, TokenType::Not, TokenType::Equals, TokenType::Word]
  );
  checkTokensTypes(
    "10-\n-20",
    &[TokenType::UInt, TokenType::Minus, TokenType::Int]
  );
}

// =================================================================================================

/// `++` и `--` вплотную остаются одним токеном и заканчивают выражение.
#[test]
fn incrementEndsLine() -> ()
{
  checkTokensTypes(
    "a++",
    &[TokenType::Word, TokenType::UnaryPlus]
  );
  checkTokensTypes(
    "a--",
    &[TokenType::Word, TokenType::UnaryMinus]
  );
  checkTokensTypes(
    "a + +b",
    &[TokenType::Word, TokenType::Plus, TokenType::Plus, TokenType::Word]
  );
}

/// `a++` завершает выражение, а `a +` продолжает его на следующую строку.
#[test]
fn incrementDoesNotContinueLine() -> ()
{
  let mut buffer: Vec<u8> = "a++\nb".as_bytes().to_vec();
  assert!(readTokensSimple(&mut buffer).len() == 2, "`a++` и `b` должны быть двумя линиями");

  let mut buffer: Vec<u8> = "a +\nb".as_bytes().to_vec();
  assert!(readTokensSimple(&mut buffer).len() == 1, "`a +` и `b` должны быть одной линией");
}

// =================================================================================================

/// `5 - -5`: бинарный минус и отрицательное число, а не `--`.
#[test]
fn minusNegativeNumber() -> ()
{
  checkStructure!(
    "a := 5 - -5", "a",
    type TokenType::UInt,
    value "10"
  );
  checkStructure!(
    "a := 5- -5", "a",
    type TokenType::UInt,
    value "10"
  );
  checkStructure!(
    "a := -5 - -5", "a",
    type TokenType::Int,
    value "0"
  );
}

/// `1 - - 5`: пробел между минусом и числом тоже стирается.
#[test]
fn minusSpacedNegativeNumber() -> ()
{
  checkStructure!(
    "a := 1 - - 5", "a",
    type TokenType::UInt,
    value "6"
  );
}

/// todo desc
#[test]
fn minusNegativeNumberFromConstant() -> ()
{
  checkStructure!(
    r#"
    x := 6
    a := x - -5
    "#,
    "a",
    type TokenType::UInt,
    value "11"
  );
}

// =================================================================================================