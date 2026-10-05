use crate::parser::testing::check;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================
// Тесты на деление. (не на 0).

/// Целые: усечение к нулю, знак результата.
#[test]
fn divideInteger() -> ()
{
  check!(
    "a = 10 / 4",
    a,
    type TokenType::UInt,
    val "2"
  );
  check!(
    "a = 10 / 5",
    a,
    type TokenType::UInt,
    val "2"
  );
  check!(
    "a = -7 / 2",
    a,
    type TokenType::Int,
    val "-3"
  );
  check!(
    "a = 7 / -2",
    a,
    type TokenType::Int,
    val "-3"
  );
  check!(
    "a = -7 / -2",
    a,
    type TokenType::Int,
    val "3"
  );
}

/// Дробные: результат UFloat/Float, точное значение.
#[test]
fn divideFloat() -> ()
{
  check!(
    "a = 10 / 4.0",
    a,
    type TokenType::UFloat,
    val "2.5"
  );
  check!(
    "a = 10 / 0.5",
    a,
    type TokenType::UFloat,
    val "20"
  );
  check!(
    "a = 1 / 0.1",
    a,
    type TokenType::UFloat,
    val "10"
  );
  check!(
    "a = -7.5 / 2",
    a,
    type TokenType::Float,
    val "-3.75"
  );
}

/// Цепочки и приоритет: `/` выше `+`/`-`, ассоциативность слева.
#[test]
fn divideChainsAndPrecedence() -> ()
{
  check!(
    "a = 100 / 10 / 2",
    a,
    type TokenType::UInt,
    val "5"
  );
  check!(
    "a = 10 - 6 / 2",
    a,
    type TokenType::UInt,
    val "7"
  );
  check!(
    "a = 10 + 6 / 2",
    a,
    type TokenType::UInt,
    val "13"
  );
  check!(
    "a = 20 - 8 / 4 - 1",
    a,
    type TokenType::UInt,
    val "17"
  );
}

/// Минус слитно с числом (`-6` — одно число): `10 -6 / 2` = `10 + (-6 / 2)`.
#[test]
fn divideAdjacentMinus() -> ()
{
  check!(
    "a = 10 -6 / 2",
    a,
    type TokenType::UInt,
    val "7"
  );
}

// =================================================================================================
// Синтаксис float-литералов (issue #31): 0.0 = .0 = . = 0

/// Голая точка и нули: `.`, `.0`, `0.`, `0.0` → 0 | UFloat.
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

/// Leading dot: `.1`, `.5`, `.25`.
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

/// Trailing dot: `3.`, `201.` — значение без хвостовой точки.
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
