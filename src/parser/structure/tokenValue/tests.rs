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
    "x = 10\nn = -10\nz = 0\na = x / z",
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    "x = 10\nz = 0\na = x / 0",
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    "x = 10\nz = 0\na = 10 / z",
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    "n = -10\nz = 0\na = n / z",
    a,
    type TokenType::Int,
    val "-10"
  );
  check!(
    "z = 0\na = z / z",
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
    "x = 10.5\nz = 0.0\na = x / z",
    a,
    type TokenType::UFloat,
    val "10.5"
  );
  check!(
    "x = 10.5\na = x / 0.0",
    a,
    type TokenType::UFloat,
    val "10.5"
  );
  check!(
    "n = -10.5\nz = 0.0\na = n / z",
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
    "zA: U8 = 0\na = 10 / zA",
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    "xA: U8 = 10\na = xA / 0",
    a,
    type TokenType::UInt,
    val "10"
  );
  // U16
  check!(
    "zB: U16 = 0\na = 10 / zB",
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    "xB: U16 = 10\na = xB / 0",
    a,
    type TokenType::UInt,
    val "10"
  );
  // U32
  check!(
    "zC: U32 = 0\na = 10 / zC",
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    "xC: U32 = 10\na = xC / 0",
    a,
    type TokenType::UInt,
    val "10"
  );
  // U64
  check!(
    "zD: U64 = 0\na = 10 / zD",
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    "xD: U64 = 10\na = xD / 0",
    a,
    type TokenType::UInt,
    val "10"
  );
  // Usize
  check!(
    "zE: Usize = 0\na = 10 / zE",
    a,
    type TokenType::UInt,
    val "10"
  );
  check!(
    "xE: Usize = 10\na = xE / 0",
    a,
    type TokenType::UInt,
    val "10"
  );

  // I8
  check!(
    "zF: I8 = 0\na = -10 / zF",
    a,
    type TokenType::Int,
    val "-10"
  );
  check!(
    "xF: I8 = -10\na = xF / 0",
    a,
    type TokenType::Int,
    val "-10"
  );
  // I16
  check!(
    "zG: I16 = 0\na = -10 / zG",
    a,
    type TokenType::Int,
    val "-10"
  );
  check!(
    "xG: I16 = -10\na = xG / 0",
    a,
    type TokenType::Int,
    val "-10"
  );
  // I32
  check!(
    "zH: I32 = 0\na = -10 / zH",
    a,
    type TokenType::Int,
    val "-10"
  );
  check!(
    "xH: I32 = -10\na = xH / 0",
    a,
    type TokenType::Int,
    val "-10"
  );
  // I64
  check!(
    "zI: I64 = 0\na = -10 / zI",
    a,
    type TokenType::Int,
    val "-10"
  );
  check!(
    "xI: I64 = -10\na = xI / 0",
    a,
    type TokenType::Int,
    val "-10"
  );
  // Isize
  check!(
    "zJ: Isize = 0\na = -10 / zJ",
    a,
    type TokenType::Int,
    val "-10"
  );
  check!(
    "xJ: Isize = -10\na = xJ / 0",
    a,
    type TokenType::Int,
    val "-10"
  );

  // F32
  check!(
    "zK: F32 = 0.0\na = 10.5 / zK",
    a,
    type TokenType::UFloat,
    val "10.5"
  );
  check!(
    "xK: F32 = 10.5\na = xK / 0.0",
    a,
    type TokenType::UFloat,
    val "10.5"
  );
  check!(
    "yK: F32 = 0.0\na = -10.5 / yK",
    a,
    type TokenType::Float,
    val "-10.5"
  );

  // F64
  check!(
    "zL: F64 = 0.0\na = 10.5 / zL",
    a,
    type TokenType::UFloat,
    val "10.5"
  );
  check!(
    "xL: F64 = 10.5\na = xL / 0.0",
    a,
    type TokenType::UFloat,
    val "10.5"
  );
  check!(
    "yL: F64 = 0.0\na = -10.5 / yL",
    a,
    type TokenType::Float,
    val "-10.5"
  );
}

/// Char, String, True/False; цепочки и приоритет.
#[test]
fn divideByZeroOther() -> ()
{
  // Char и String (в .rt — println выражения; после assignment data пустая, сверяем type).
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

