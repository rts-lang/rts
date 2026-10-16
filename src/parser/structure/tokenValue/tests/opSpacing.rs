use crate::parser::testing::checkStructure;
use crate::tokenizer::tokenizer::readTokensSimple;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock};
// =================================================================================================

// Тесты на пробелы между знаками операторов.
//
// Пробелы в RTS стираются везде, кроме одного места: между двумя знаками оператора.
// Два знака склеиваются в один оператор только вплотную: `!=` - один оператор,
// `! =` - два. Новый токен для пробела не нужен: две раздельные операции и есть информация о нём.
//
// Это нужно, чтобы выражение `a - -b` (бинарный `-` и унарный `-`) не слипалось
// в составной оператор, который имеет другой смысл.

// =================================================================================================

/// Вспомогательная функция:
/// Читает код токенайзером и сверяет типы токенов первого уровня по порядку.
fn checkTypes(code: &str, expected: &[TokenType]) -> ()
{
  let mut buffer: Vec<u8> = code.as_bytes().to_vec();
  let lines: Vec< Arc<RwLock<Line>> > = readTokensSimple(&mut buffer);

  let mut actual: Vec<String> = Vec::new();
  for lineLink in lines
  {
    let line = lineLink.read().unwrap();
    if let Some(tokens) = &line.tokens
    {
      for token in tokens
      {
        let token: &Token = token;
        actual.push(token.getDataType().to_string());
      }
    }
  }

  let expected: Vec<String> = expected.iter().map(|tokenType: &TokenType| tokenType.to_string()).collect();
  assert!(
    actual == expected,
    "Код `{}`: ожидались токены [{}], а получили [{}]",
    code.replace('\n', "\\n"), expected.join(", "), actual.join(", ")
  );
}

// =================================================================================================

/// Знаки через пробел остаются раздельными операторами.
#[test]
fn spacedSignsStaySeparate() -> ()
{
  checkTypes("a - -b", &[TokenType::Word, TokenType::Minus, TokenType::Minus, TokenType::Word]);
  checkTypes("a- -b", &[TokenType::Word, TokenType::Minus, TokenType::Minus, TokenType::Word]);
  checkTypes("a + +b", &[TokenType::Word, TokenType::Plus, TokenType::Plus, TokenType::Word]);
  checkTypes("a ! = b", &[TokenType::Word, TokenType::Not, TokenType::Equals, TokenType::Word]);
  checkTypes("a : = b", &[TokenType::Word, TokenType::Colon, TokenType::Equals, TokenType::Word]);
  checkTypes("a < = b", &[TokenType::Word, TokenType::LessThan, TokenType::Equals, TokenType::Word]);
  checkTypes("a > = b", &[TokenType::Word, TokenType::GreaterThan, TokenType::Equals, TokenType::Word]);
}

/// Знаки вплотную по-прежнему склеиваются в один оператор.
#[test]
fn adjacentSignsMerge() -> ()
{
  checkTypes("a != b", &[TokenType::Word, TokenType::NotEquals, TokenType::Word]);
  checkTypes("a:=b", &[TokenType::Word, TokenType::Creation, TokenType::Word]);
  checkTypes("a <= b", &[TokenType::Word, TokenType::LessThanOrEquals, TokenType::Word]);
  checkTypes("a >= b", &[TokenType::Word, TokenType::GreaterThanOrEquals, TokenType::Word]);
  checkTypes("a -> b", &[TokenType::Word, TokenType::Pointer, TokenType::Word]);
}

/// Пробелы вокруг знаков по-прежнему стираются: смысл тот же.
#[test]
fn spacesAroundSignsAreErased() -> ()
{
  checkTypes("a-b", &[TokenType::Word, TokenType::Minus, TokenType::Word]);
  checkTypes("a -b", &[TokenType::Word, TokenType::Minus, TokenType::Word]);
  checkTypes("a - b", &[TokenType::Word, TokenType::Minus, TokenType::Word]);
  checkTypes("a  !=  b", &[TokenType::Word, TokenType::NotEquals, TokenType::Word]);
}

/// Перенос строки между знаками не склеивает их: оператор в конце строки
/// просто продолжает выражение (#85), а следующий знак читается заново.
#[test]
fn newlineBetweenSignsDoesNotMerge() -> ()
{
  checkTypes("a -\n-b", &[TokenType::Word, TokenType::Minus, TokenType::Minus, TokenType::Word]);
  checkTypes("a !\n= b", &[TokenType::Word, TokenType::Not, TokenType::Equals, TokenType::Word]);
  checkTypes("10-\n-20", &[TokenType::UInt, TokenType::Minus, TokenType::Int]);
}

// =================================================================================================

/// Составных `**`, `//`, `%%`, `^^` нет: каждый знак читается отдельным оператором.
/// Из двойных остаются только `++`, `--` и операторы с `=`.
/// Благодаря этому `*` можно использовать как разыменование: `**p` и `a**b` не слипаются.
#[test]
fn doubledSignsAreRemoved() -> ()
{
  checkTypes("**a", &[TokenType::Multiply, TokenType::Multiply, TokenType::Word]);
  checkTypes("a**b", &[TokenType::Word, TokenType::Multiply, TokenType::Multiply, TokenType::Word]);
  checkTypes("a//b", &[TokenType::Word, TokenType::Divide, TokenType::Divide, TokenType::Word]);
  checkTypes("a%%b", &[TokenType::Word, TokenType::Modulo, TokenType::Modulo, TokenType::Word]);
  checkTypes("a^^b", &[TokenType::Word, TokenType::Exponent, TokenType::Exponent, TokenType::Word]);
}

/// `++` и `--` вплотную остаются одним токеном и заканчивают выражение (#85).
#[test]
fn incrementEndsLine() -> ()
{
  checkTypes("a++", &[TokenType::Word, TokenType::UnaryPlus]);
  checkTypes("a--", &[TokenType::Word, TokenType::UnaryMinus]);
  checkTypes("a + +b", &[TokenType::Word, TokenType::Plus, TokenType::Plus, TokenType::Word]);
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
    "a := 5 - -5",
    "a",
    type TokenType::UInt,
    value "10"
  );
  checkStructure!(
    "a := 5- -5",
    "a",
    type TokenType::UInt,
    value "10"
  );
  checkStructure!(
    "a := -5 - -5",
    "a",
    type TokenType::Int,
    value "0"
  );
}

/// `1 - - 5`: пробел между минусом и числом тоже стирается, как и раньше.
#[test]
fn minusSpacedNegativeNumber() -> ()
{
  checkStructure!(
    "a := 1 - - 5",
    "a",
    type TokenType::UInt,
    value "6"
  );
}

/// То же самое для константы слева.
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
