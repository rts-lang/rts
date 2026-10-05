use crate::parser::structure::structureType::StructureType;
use crate::parser::testing::check;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================
// Деление на 0 не ошибка: результат - левая часть выражения (issue #30).

// todo Не знаю нужны ли тут stype кому-то еще для большей проверки?
//  Оно частично используется тут, но вообще это же математика а `StructureType` check.

// =================================================================================================

/// Целые UInt/Int: литералы и переменные.
#[test]
fn divideByZeroInteger() -> ()
{
  // UInt / UInt.
  check!(
    "a = 10 / 0",
    a,
    type TokenType::UInt,
    value "10"
  );
  check!(
    "a = 0 / 0",
    a,
    type TokenType::UInt,
    value "0"
  );
  check!(
    "a = 255 / 0",
    a,
    type TokenType::UInt,
    value "255"
  );
  check!(
    "a = 18446744073709551615 / 0",
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    value "18446744073709551615"
  );

  // Int / UInt.
  check!(
    "a = -10 / 0",
    a,
    type TokenType::Int,
    value "-10"
  );
  check!(
    "a = -9223372036854775808 / 0",
    a,
    type TokenType::Int,
    stype StructureType::I64,
    value "-9223372036854775808"
  );

  // Нулевой делитель со знаком.
  check!(
    "a = 10 / -0",
    a,
    type TokenType::UInt,
    value "10"
  );
  check!(
    "a = -10 / -0",
    a,
    type TokenType::Int,
    value "-10"
  );

  // Константы.
  check!(
    r#"
      x = 10
      n = -10
      z = 0
      a = x / z
    "#,
    a,
    type TokenType::UInt,
    value "10"
  );
  check!(
    r#"
      x = 10
      z = 0
      a = x / 0
    "#,
    a,
    type TokenType::UInt,
    value "10"
  );
  check!(
    r#"
      x = 10
      z = 0
      a = 10 / z
    "#,
    a,
    type TokenType::UInt,
    value "10"
  );
  check!(
    r#"
      n = -10
      z = 0
      a = n / z
    "#,
    a,
    type TokenType::Int,
    value "-10"
  );
  check!(
    r#"
      z = 0
      a = z / z
    "#,
    a,
    type TokenType::UInt,
    value "0"
  );
}

/// Дробные UFloat/Float: литералы и переменные.
#[test]
fn divideByZeroFloat() -> ()
{
  check!(
    "a = 10.5 / 0.0",
    a,
    type TokenType::UFloat,
    value "10.5"
  );
  check!(
    "a = 0.0 / 0.0",
    a,
    type TokenType::UFloat,
    value "0"
  );
  check!(
    "a = -10.5 / 0.0",
    a,
    type TokenType::Float,
    value "-10.5"
  );
  
  // UFloat / Float (отрицательный ноль тоже ноль).
  check!(
    "a = 10.5 / -0.0",
    a,
    type TokenType::UFloat,
    value "10.5"
  );
  check!(
    "a = -10.5 / -0.0",
    a,
    type TokenType::Float,
    value "-10.5"
  );

  // Константы.
  check!(
    r#"
      x = 10.5
      z = 0.0
      a = x / z
    "#,
    a,
    type TokenType::UFloat,
    value "10.5"
  );
  check!(
    r#"
      x = 10.5
      a = x / 0.0
    "#,
    a,
    type TokenType::UFloat,
    value "10.5"
  );
  check!(
    r#"
      n = -10.5
      z = 0.0
      a = n / z
    "#,
    a,
    type TokenType::Float,
    value "-10.5"
  );
}

/// Смешанные целое/дробное: тип результата определяют типы операндов.
#[test]
fn divideByZeroMixed() -> ()
{
  // Целое / дробный ноль.
  check!(
    "a = 10 / 0.0",
    a,
    type TokenType::UFloat,
    value "10"
  );
  check!(
    "a = 0 / 0.0",
    a,
    type TokenType::UFloat,
    value "0"
  );
  check!(
    "a = -10 / 0.0",
    a,
    type TokenType::Float,
    value "-10"
  );
  check!(
    "a = 10 / -0.0",
    a,
    type TokenType::Float,
    value "10"
  );
  check!(
    "a = -10 / -0.0",
    a,
    type TokenType::Float,
    value "-10"
  );

  // Дробное / целый ноль.
  check!(
    "a = 10.5 / 0",
    a,
    type TokenType::UFloat,
    value "10.5"
  );
  check!(
    "a = -10.5 / 0",
    a,
    type TokenType::Float,
    value "-10.5"
  );
  check!(
    "a = 10.5 / -0",
    a,
    type TokenType::UFloat,
    value "10.5"
  );
  check!(
    "a = -10.5 / -0",
    a,
    type TokenType::Float,
    value "-10.5"
  );
}

/// ABI типы как делитель и как делимое.
#[test]
fn divideByZeroAbi() -> ()
{
  // U8
  check!(
    r#"
      zA: U8 = 0
      a = 10 / zA
    "#,
    a,
    type TokenType::UInt,
    value "10"
  );
  check!(
    r#"
      xA: U8 = 10
      a = xA / 0
    "#,
    a,
    type TokenType::UInt,
    value "10"
  );
  // U16
  check!(
    r#"
      zB: U16 = 0
      a = 10 / zB
    "#,
    a,
    type TokenType::UInt,
    value "10"
  );
  check!(
    r#"
      xB: U16 = 10
      a = xB / 0
    "#,
    a,
    type TokenType::UInt,
    value "10"
  );
  // U32
  check!(
    r#"
      zC: U32 = 0
      a = 10 / zC
    "#,
    a,
    type TokenType::UInt,
    value "10"
  );
  check!(
    r#"
      xC: U32 = 10
      a = xC / 0
    "#,
    a,
    type TokenType::UInt,
    value "10"
  );
  // U64
  check!(
    r#"
      zD: U64 = 0
      a = 10 / zD
    "#,
    a,
    type TokenType::UInt,
    value "10"
  );
  check!(
    r#"
      xD: U64 = 10
      a = xD / 0
    "#,
    a,
    type TokenType::UInt,
    value "10"
  );
  // Usize
  check!(
    r#"
      zE: Usize = 0
      a = 10 / zE
    "#,
    a,
    type TokenType::UInt,
    value "10"
  );
  check!(
    r#"
      xE: Usize = 10
      a = xE / 0
    "#,
    a,
    type TokenType::UInt,
    value "10"
  );

  // I8
  check!(
    r#"
      zF: I8 = 0
      a = -10 / zF
    "#,
    a,
    type TokenType::Int,
    value "-10"
  );
  check!(
    r#"
      xF: I8 = -10
      a = xF / 0
    "#,
    a,
    type TokenType::Int,
    value "-10"
  );
  // I16
  check!(
    r#"
      zG: I16 = 0
      a = -10 / zG
    "#,
    a,
    type TokenType::Int,
    value "-10"
  );
  check!(
    r#"
      xG: I16 = -10
      a = xG / 0
    "#,
    a,
    type TokenType::Int,
    value "-10"
  );
  // I32
  check!(
    r#"
      zH: I32 = 0
      a = -10 / zH
    "#,
    a,
    type TokenType::Int,
    value "-10"
  );
  check!(
    r#"
      xH: I32 = -10
      a = xH / 0
    "#,
    a,
    type TokenType::Int,
    value "-10"
  );
  // I64
  check!(
    r#"
      zI: I64 = 0
      a = -10 / zI
    "#,
    a,
    type TokenType::Int,
    value "-10"
  );
  check!(
    r#"
      xI: I64 = -10
      a = xI / 0
    "#,
    a,
    type TokenType::Int,
    value "-10"
  );
  // Isize
  check!(
    r#"
      zJ: Isize = 0
      a = -10 / zJ
    "#,
    a,
    type TokenType::Int,
    value "-10"
  );
  check!(
    r#"
      xJ: Isize = -10
      a = xJ / 0
    "#,
    a,
    type TokenType::Int,
    value "-10"
  );

  // F32
  check!(
    r#"
      zK: F32 = 0.0
      a = 10.5 / zK
    "#,
    a,
    type TokenType::UFloat,
    value "10.5"
  );
  check!(
    r#"
      xK: F32 = 10.5
      a = xK / 0.0
    "#,
    a,
    type TokenType::UFloat,
    value "10.5"
  );
  check!(
    r#"
      yK: F32 = 0.0
      a = -10.5 / yK
    "#,
    a,
    type TokenType::Float,
    value "-10.5"
  );

  // F64
  check!(
    r#"
      zL: F64 = 0.0
      a = 10.5 / zL
    "#,
    a,
    type TokenType::UFloat,
    value "10.5"
  );
  check!(
    r#"
      xL: F64 = 10.5
      a = xL / 0.0
    "#,
    a,
    type TokenType::UFloat,
    value "10.5"
  );
  check!(
    r#"
      yL: F64 = 0.0
      a = -10.5 / yL
    "#,
    a,
    type TokenType::Float,
    value "-10.5"
  );
}

/// Char, String, True/False; цепочки и приоритет.
#[test]
fn divideByZeroOther() -> ()
{
  // Char и String.
  check!(
    "a = 'a' / 0",
    a,
    type TokenType::Char
  );
  check!(
    "a = \"abc\" / 0",
    a,
    type TokenType::String
  );
  check!(
    "a = 10 / 'a'",
    a,
    type TokenType::UInt,
    value "10"
  );
  check!(
    "a = True / False",
    a,
    type TokenType::UInt,
    value "1"
  );

  // Цепочки: каждый шаг слева направо.
  check!(
    "a = 100 / 0 / 2",
    a,
    type TokenType::UInt,
    value "50"
  );
  check!(
    "a = 100 / 5 / 0",
    a,
    type TokenType::UInt,
    value "20"
  );
  check!(
    "a = 100 / 0 / 0",
    a,
    type TokenType::UInt,
    value "100"
  );

  // Приоритет: деление раньше + и -.
  check!(
    "a = 10 - 6 / 0",
    a,
    type TokenType::UInt,
    value "4"
  );
  check!(
    "a = 10 + 6 / 0",
    a,
    type TokenType::UInt,
    value "16"
  );
  check!(
    "a = 1 / 0 + 2 / 0",
    a,
    type TokenType::UInt,
    value "3"
  );
  check!(
    "a = -1 - 1 / 0",
    a,
    type TokenType::Int,
    value "-2"
  );
}

// =================================================================================================
