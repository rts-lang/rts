use crate::parser::testing::checkStructure;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Синтаксис float-литералов (issue #31): 0.0 = .0 = . = 0

// =================================================================================================

/// Проверка нескольких разных написаний нулей.
#[test]
fn floatDotZeros() -> ()
{
  checkStructure!(
    "a = .",
    a,
    type TokenType::UFloat,
    value "0"
  );
  checkStructure!(
    "a = .0",
    a,
    type TokenType::UFloat,
    value "0"
  );
  checkStructure!(
    "a = 0.",
    a,
    type TokenType::UFloat,
    value "0"
  );
  checkStructure!(
    "a = 0.0",
    a,
    type TokenType::UFloat,
    value "0"
  );
}

/// Без целой части. Поэтому десятичная часть будет 0.
#[test]
fn floatDotLeading() -> ()
{
  checkStructure!(
    "a = .1",
    a,
    type TokenType::UFloat,
    value "0.1"
  );
  checkStructure!(
    "a = .5",
    a,
    type TokenType::UFloat,
    value "0.5"
  );
  checkStructure!(
    "a = .25",
    a,
    type TokenType::UFloat,
    value "0.25"
  );
}

/// Без дробной части. Поэтому дробная часть будет 0.
#[test]
fn floatDotTrailing() -> ()
{
  checkStructure!(
    "a = 3.",
    a,
    type TokenType::UFloat,
    value "3"
  );
  checkStructure!(
    "a = 201.",
    a,
    type TokenType::UFloat,
    value "201"
  );
}

/// Выражения с float-литералами (+ - /).
#[test]
fn floatDotExpressions() -> ()
{
  checkStructure!(
    "a = .5 + .5",
    a,
    type TokenType::UFloat,
    value "1"
  );
  checkStructure!(
    "a = 1. + .5",
    a,
    type TokenType::UFloat,
    value "1.5"
  );
  checkStructure!(
    "a = 3. + 2.",
    a,
    type TokenType::UFloat,
    value "5"
  );
  checkStructure!(
    "a = .5 - .25",
    a,
    type TokenType::UFloat,
    value "0.25"
  );
  checkStructure!(
    "a = 10.5 - 10.0",
    a,
    type TokenType::UFloat,
    value "0.5"
  );
  checkStructure!(
    "a = . + .5",
    a,
    type TokenType::UFloat,
    value "0.5"
  );
  checkStructure!(
    "a = 1 / .5",
    a,
    type TokenType::UFloat,
    value "2"
  );
}

// =================================================================================================
