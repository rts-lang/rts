use crate::parser::structure::structureType::StructureType;
use crate::parser::testing::check;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================
// Границы: type токена бесконечен (UInt/Int/UFloat), а stype и значение зажимает структура (#71).

/// Большой литерал в делении: на 1 и на 0 (issue #30).
#[test]
fn limitsDivideBig() -> ()
{
  // UInt.
  check!(
    "a = 18446744073709551615 / 1",
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    val &u64::MAX.to_string()
  );
  check!(
    "a = 99999999999999999999999 / 0",
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    val &u64::MAX.to_string()
  );
  check!(
    "a = 99999999999999999999999 / 1",
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    val &u64::MAX.to_string()
  );

  // Int.
  check!(
    "a = -99999999999999999999999 / 0",
    a,
    type TokenType::Int,
    stype StructureType::I64,
    val &i64::MIN.to_string()
  );
  check!(
    "a = -99999999999999999999999 / 1",
    a,
    type TokenType::Int,
    stype StructureType::I64,
    val &i64::MIN.to_string()
  );
}

/// Переполнение самой операции зажимается в границу, а не падает.
#[test]
fn limitsOverflow() -> ()
{
  check!(
    "a = 18446744073709551615 + 1",
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    val &u64::MAX.to_string()
  );
  check!(
    "a = 18446744073709551615 * 2",
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    val &u64::MAX.to_string()
  );
  check!(
    "a = -9223372036854775808 - 1",
    a,
    type TokenType::Int,
    stype StructureType::I64,
    val &i64::MIN.to_string()
  );
  // Результат больше i64::MAX.
  check!(
    "a = -9223372036854775808 / -1",
    a,
    type TokenType::Int,
    stype StructureType::I64,
    val &i64::MAX.to_string()
  );
}

/// Новое значение в `~~` переменной: stype выводится заново, а не остаётся U8 от `0`.
#[test]
fn limitsReassign() -> ()
{
  check!(
    r#"
      a~~ = 0
      a = 18446744073709551615
    "#,
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    val &u64::MAX.to_string()
  );
  check!(
    r#"
      a~~ = 0
      a = 18446744073709551616
    "#,
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    val &u64::MAX.to_string()
  );
  check!(
    r#"
      a~~ = 0
      a = 99999999999999999999999
    "#,
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    val &u64::MAX.to_string()
  );
  check!(
    r#"
      a~~ = 0
      a = -9223372036854775809
    "#,
    a,
    type TokenType::Int,
    stype StructureType::I64,
    val &i64::MIN.to_string()
  );
}

/// Float: type зависит только от знака, а не от `f64::parse()`; результат не `None`.
#[test]
fn limitsFloat() -> ()
{
  // UFloat - UFloat с отрицательным результатом.
  check!(
    "a = 1.0 - 2.0",
    a,
    type TokenType::Float,
    val "-1"
  );
  // inf (переполнение UFloat + UFloat).
  check!(
    "a = 1e308 + 1e308",
    a,
    type TokenType::UFloat
  );
  // NaN (inf - inf): не начинается с `-`, значит UFloat.
  check!(
    "a = (1e308 + 1e308) - (1e308 + 1e308)",
    a,
    type TokenType::UFloat
  );
  // Литерал больше f64::MAX: токенайзер хранит как есть.
  check!(
    "a = 1e309",
    a,
    type TokenType::UFloat
  );
}

// =================================================================================================
