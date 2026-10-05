use crate::parser::testing::check;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Тесты на деление.

// =================================================================================================

/// Целые: усечение к нулю, знак результата.
#[test]
fn divideInteger() -> ()
{
  check!(
    "a = 10 / 4",
    a,
    type TokenType::UInt,
    value "2"
  );
  check!(
    "a = 10 / 5",
    a,
    type TokenType::UInt,
    value "2"
  );
  check!(
    "a = -7 / 2",
    a,
    type TokenType::Int,
    value "-3"
  );
  check!(
    "a = 7 / -2",
    a,
    type TokenType::Int,
    value "-3"
  );
  check!(
    "a = -7 / -2",
    a,
    type TokenType::Int,
    value "3"
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
    value "2.5"
  );
  check!(
    "a = 10 / 0.5",
    a,
    type TokenType::UFloat,
    value "20"
  );
  check!(
    "a = 1 / 0.1",
    a,
    type TokenType::UFloat,
    value "10"
  );
  check!(
    "a = -7.5 / 2",
    a,
    type TokenType::Float,
    value "-3.75"
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
    value "5"
  );
  check!(
    "a = 10 - 6 / 2",
    a,
    type TokenType::UInt,
    value "7"
  );
  check!(
    "a = 10 + 6 / 2",
    a,
    type TokenType::UInt,
    value "13"
  );
  check!(
    "a = 20 - 8 / 4 - 1",
    a,
    type TokenType::UInt,
    value "17"
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
    value "7"
  );
}

// =================================================================================================
