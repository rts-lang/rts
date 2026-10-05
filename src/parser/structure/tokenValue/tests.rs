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
// Деление на 0 не ошибка: результат - левая часть выражения (issue #30).

/// Целые UInt/Int: литералы и переменные.
#[test]
fn divideByZeroInteger() -> ()
{
  // UInt / UInt.
  check!(
    "a = 10 / 0",
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    "a = 0 / 0",
    a,
    type TokenType::UInt,
    val "0"
  );
  check!(
    "a = 255 / 0",
    a,
    type TokenType::UInt,
    val "255"
  );
  check!(
    "a = 18446744073709551615 / 0",
    a,
    type TokenType::UInt,
    val "18446744073709551615"
  );

  // Int / UInt.
  check!(
    "a = -10 / 0",
    a,
    type TokenType::Int,
    val "-10"
  );
  check!(
    "a = -9223372036854775808 / 0",
    a,
    type TokenType::Int,
    val "-9223372036854775808"
  );

  // Нулевой делитель со знаком.
  check!(
    "a = 10 / -0",
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    "a = -10 / -0",
    a,
    type TokenType::Int,
    val "-10"
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
    val "10"
  );
  check!(
    r#"
      x = 10
      z = 0
      a = x / 0
    "#,
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    r#"
      x = 10
      z = 0
      a = 10 / z
    "#,
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    r#"
      n = -10
      z = 0
      a = n / z
    "#,
    a,
    type TokenType::Int,
    val "-10"
  );
  check!(
    r#"
      z = 0
      a = z / z
    "#,
    a,
    type TokenType::UInt,
    val "0"
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
    val "10.5"
  );
  check!(
    "a = 0.0 / 0.0",
    a,
    type TokenType::UFloat,
    val "0"
  );
  check!(
    "a = -10.5 / 0.0",
    a,
    type TokenType::Float,
    val "-10.5"
  );
  
  // UFloat / Float (отрицательный ноль тоже ноль).
  check!(
    "a = 10.5 / -0.0",
    a,
    type TokenType::UFloat,
    val "10.5"
  );
  check!(
    "a = -10.5 / -0.0",
    a,
    type TokenType::Float,
    val "-10.5"
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
    val "10.5"
  );
  check!(
    r#"
      x = 10.5
      a = x / 0.0
    "#,
    a,
    type TokenType::UFloat,
    val "10.5"
  );
  check!(
    r#"
      n = -10.5
      z = 0.0
      a = n / z
    "#,
    a,
    type TokenType::Float,
    val "-10.5"
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
    val "10"
  );
  check!(
    "a = 0 / 0.0",
    a,
    type TokenType::UFloat,
    val "0"
  );
  check!(
    "a = -10 / 0.0",
    a,
    type TokenType::Float,
    val "-10"
  );
  check!(
    "a = 10 / -0.0",
    a,
    type TokenType::Float,
    val "10"
  );
  check!(
    "a = -10 / -0.0",
    a,
    type TokenType::Float,
    val "-10"
  );

  // Дробное / целый ноль.
  check!(
    "a = 10.5 / 0",
    a,
    type TokenType::UFloat,
    val "10.5"
  );
  check!(
    "a = -10.5 / 0",
    a,
    type TokenType::Float,
    val "-10.5"
  );
  check!(
    "a = 10.5 / -0",
    a,
    type TokenType::UFloat,
    val "10.5"
  );
  check!(
    "a = -10.5 / -0",
    a,
    type TokenType::Float,
    val "-10.5"
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
    val "10"
  );
  check!(
    r#"
      xA: U8 = 10
      a = xA / 0
    "#,
    a,
    type TokenType::UInt,
    val "10"
  );
  // U16
  check!(
    r#"
      zB: U16 = 0
      a = 10 / zB
    "#,
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    r#"
      xB: U16 = 10
      a = xB / 0
    "#,
    a,
    type TokenType::UInt,
    val "10"
  );
  // U32
  check!(
    r#"
      zC: U32 = 0
      a = 10 / zC
    "#,
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    r#"
      xC: U32 = 10
      a = xC / 0
    "#,
    a,
    type TokenType::UInt,
    val "10"
  );
  // U64
  check!(
    r#"
      zD: U64 = 0
      a = 10 / zD
    "#,
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    r#"
      xD: U64 = 10
      a = xD / 0
    "#,
    a,
    type TokenType::UInt,
    val "10"
  );
  // Usize
  check!(
    r#"
      zE: Usize = 0
      a = 10 / zE
    "#,
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    r#"
      xE: Usize = 10
      a = xE / 0
    "#,
    a,
    type TokenType::UInt,
    val "10"
  );

  // I8
  check!(
    r#"
      zF: I8 = 0
      a = -10 / zF
    "#,
    a,
    type TokenType::Int,
    val "-10"
  );
  check!(
    r#"
      xF: I8 = -10
      a = xF / 0
    "#,
    a,
    type TokenType::Int,
    val "-10"
  );
  // I16
  check!(
    r#"
      zG: I16 = 0
      a = -10 / zG
    "#,
    a,
    type TokenType::Int,
    val "-10"
  );
  check!(
    r#"
      xG: I16 = -10
      a = xG / 0
    "#,
    a,
    type TokenType::Int,
    val "-10"
  );
  // I32
  check!(
    r#"
      zH: I32 = 0
      a = -10 / zH
    "#,
    a,
    type TokenType::Int,
    val "-10"
  );
  check!(
    r#"
      xH: I32 = -10
      a = xH / 0
    "#,
    a,
    type TokenType::Int,
    val "-10"
  );
  // I64
  check!(
    r#"
      zI: I64 = 0
      a = -10 / zI
    "#,
    a,
    type TokenType::Int,
    val "-10"
  );
  check!(
    r#"
      xI: I64 = -10
      a = xI / 0
    "#,
    a,
    type TokenType::Int,
    val "-10"
  );
  // Isize
  check!(
    r#"
      zJ: Isize = 0
      a = -10 / zJ
    "#,
    a,
    type TokenType::Int,
    val "-10"
  );
  check!(
    r#"
      xJ: Isize = -10
      a = xJ / 0
    "#,
    a,
    type TokenType::Int,
    val "-10"
  );

  // F32
  check!(
    r#"
      zK: F32 = 0.0
      a = 10.5 / zK
    "#,
    a,
    type TokenType::UFloat,
    val "10.5"
  );
  check!(
    r#"
      xK: F32 = 10.5
      a = xK / 0.0
    "#,
    a,
    type TokenType::UFloat,
    val "10.5"
  );
  check!(
    r#"
      yK: F32 = 0.0
      a = -10.5 / yK
    "#,
    a,
    type TokenType::Float,
    val "-10.5"
  );

  // F64
  check!(
    r#"
      zL: F64 = 0.0
      a = 10.5 / zL
    "#,
    a,
    type TokenType::UFloat,
    val "10.5"
  );
  check!(
    r#"
      xL: F64 = 10.5
      a = xL / 0.0
    "#,
    a,
    type TokenType::UFloat,
    val "10.5"
  );
  check!(
    r#"
      yL: F64 = 0.0
      a = -10.5 / yL
    "#,
    a,
    type TokenType::Float,
    val "-10.5"
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
    val "10"
  );
  check!(
    "a = True / False",
    a,
    type TokenType::UInt,
    val "1"
  );

  // Цепочки: каждый шаг слева направо.
  check!(
    "a = 100 / 0 / 2",
    a,
    type TokenType::UInt,
    val "50"
  );
  check!(
    "a = 100 / 5 / 0",
    a,
    type TokenType::UInt,
    val "20"
  );
  check!(
    "a = 100 / 0 / 0",
    a,
    type TokenType::UInt,
    val "100"
  );

  // Приоритет: деление раньше + и -.
  check!(
    "a = 10 - 6 / 0",
    a,
    type TokenType::UInt,
    val "4"
  );
  check!(
    "a = 10 + 6 / 0",
    a,
    type TokenType::UInt,
    val "16"
  );
  check!(
    "a = 1 / 0 + 2 / 0",
    a,
    type TokenType::UInt,
    val "3"
  );
  check!(
    "a = -1 - 1 / 0",
    a,
    type TokenType::Int,
    val "-2"
  );
}

// =================================================================================================

