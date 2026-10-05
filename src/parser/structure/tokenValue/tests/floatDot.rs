use crate::parser::testing::check;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Синтаксис float-литералов (issue #31): 0.0 = .0 = . = 0

// =================================================================================================

/// Проверка нескольких разных написаний нулей.
#[test]
fn floatDotZeros() -> ()
{
  check!(
    "a = .",
    a,
    type TokenType::UFloat,
    val "0"
  );
  check!(
    "a = .0",
    a,
    type TokenType::UFloat,
    val "0"
  );
  check!(
    "a = 0.",
    a,
    type TokenType::UFloat,
    val "0"
  );
  check!(
    "a = 0.0",
    a,
    type TokenType::UFloat,
    val "0"
  );
}

/// Без целой части. Поэтому десятичная часть будет 0.
#[test]
fn floatDotLeading() -> ()
{
  check!(
    "a = .1",
    a,
    type TokenType::UFloat,
    val "0.1"
  );
  check!(
    "a = .5",
    a,
    type TokenType::UFloat,
    val "0.5"
  );
  check!(
    "a = .25",
    a,
    type TokenType::UFloat,
    val "0.25"
  );
}

/// Без дробной части. Поэтому дробная часть будет 0.
#[test]
fn floatDotTrailing() -> ()
{
  check!(
    "a = 3.",
    a,
    type TokenType::UFloat,
    val "3"
  );
  check!(
    "a = 201.",
    a,
    type TokenType::UFloat,
    val "201"
  );
}

/// Выражения с float-литералами (+ - /).
#[test]
fn floatDotExpressions() -> ()
{
  check!(
    "a = .5 + .5",
    a,
    type TokenType::UFloat,
    val "1"
  );
  check!(
    "a = 1. + .5",
    a,
    type TokenType::UFloat,
    val "1.5"
  );
  check!(
    "a = 3. + 2.",
    a,
    type TokenType::UFloat,
    val "5"
  );
  check!(
    "a = .5 - .25",
    a,
    type TokenType::UFloat,
    val "0.25"
  );
  check!(
    "a = 10.5 - 10.0",
    a,
    type TokenType::UFloat,
    val "0.5"
  );
  check!(
    "a = . + .5",
    a,
    type TokenType::UFloat,
    val "0.5"
  );
  check!(
    "a = 1 / .5",
    a,
    type TokenType::UFloat,
    val "2"
  );
}

// =================================================================================================
