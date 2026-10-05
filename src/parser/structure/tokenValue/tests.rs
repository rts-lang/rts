use crate::parser::testing::check;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

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
