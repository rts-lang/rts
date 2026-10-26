use crate::parser::testing::checkStructure;
// =================================================================================================

// Тесты деления на `0`. Результатом должна быть левая часть выражения.
//
//
// todo Не знаю нужны ли тут `stype` кому-то еще для большей проверки?
//  Оно частично используется тут, но вообще это же математика а `StructureType` check.

// =================================================================================================

/// Целые `UInt`/`Int`: примитивы и константы.
#[test]
fn divideByZeroInteger() -> ()
{
  // `UInt / UInt`.
  checkStructure!(
    "a := 10 / 0", "a",
    type UInt,
    value "10"
  );
  checkStructure!(
    "a := 0 / 0", "a",
    type UInt,
    value "0"
  );
  checkStructure!(
    "a := 255 / 0", "a",
    type UInt,
    value "255"
  );
  checkStructure!(
    "a := 18446744073709551615 / 0", "a",
    type UInt,
    stype U64,
    value "18446744073709551615"
  );

  // `Int / UInt`.
  checkStructure!(
    "a := -10 / 0", "a",
    type Int,
    value "-10"
  );
  checkStructure!(
    "a := -9223372036854775808 / 0", "a",
    type Int,
    stype I64,
    value "-9223372036854775808"
  );

  // Нулевой делитель со знаком.
  checkStructure!(
    "a := 10 / -0", "a",
    type UInt,
    value "10"
  );
  checkStructure!(
    "a := -10 / -0", "a",
    type Int,
    value "-10"
  );

  // Константы.
  checkStructure!(
    r#"
      x := 10
      n := -10
      z := 0
      a := x / z
    "#,
    "a",
    type UInt,
    value "10"
  );
  checkStructure!(
    r#"
      x := 10
      z := 0
      a := x / 0
    "#,
    "a",
    type UInt,
    value "10"
  );
  checkStructure!(
    r#"
      x := 10
      z := 0
      a := 10 / z
    "#,
    "a",
    type UInt,
    value "10"
  );
  checkStructure!(
    r#"
      n := -10
      z := 0
      a := n / z
    "#,
    "a",
    type Int,
    value "-10"
  );
  checkStructure!(
    r#"
      z := 0
      a := z / z
    "#,
    "a",
    type UInt,
    value "0"
  );
}

/// Дробные `UFloat`/`Float`: примитивы и константы.
#[test]
fn divideByZeroFloat() -> ()
{
  checkStructure!(
    "a := 10.5 / 0.0", "a",
    type UFloat,
    value "10.5"
  );
  checkStructure!(
    "a := 0.0 / 0.0", "a",
    type UFloat,
    value "0"
  );
  checkStructure!(
    "a := -10.5 / 0.0", "a",
    type Float,
    value "-10.5"
  );
  
  // `UFloat / Float` (отрицательный ноль тоже ноль).
  checkStructure!(
    "a := 10.5 / -0.0", "a",
    type UFloat,
    value "10.5"
  );
  checkStructure!(
    "a := -10.5 / -0.0", "a",
    type Float,
    value "-10.5"
  );

  // Константы.
  checkStructure!(
    r#"
      x := 10.5
      z := 0.0
      a := x / z
    "#,
    "a",
    type UFloat,
    value "10.5"
  );
  checkStructure!(
    r#"
      x := 10.5
      a := x / 0.0
    "#,
    "a",
    type UFloat,
    value "10.5"
  );
  checkStructure!(
    r#"
      n := -10.5
      z := 0.0
      a := n / z
    "#,
    "a",
    type Float,
    value "-10.5"
  );
}

/// Смешанные целое/дробное: тип результата определяют типы операндов.
#[test]
fn divideByZeroMixed() -> ()
{
  // Целое / дробный ноль.
  checkStructure!(
    "a := 10 / 0.0", "a",
    type UFloat,
    value "10"
  );
  checkStructure!(
    "a := 0 / 0.0", "a",
    type UFloat,
    value "0"
  );
  checkStructure!(
    "a := -10 / 0.0", "a",
    type Float,
    value "-10"
  );
  checkStructure!(
    "a := 10 / -0.0", "a",
    type Float,
    value "10"
  );
  checkStructure!(
    "a := -10 / -0.0", "a",
    type Float,
    value "-10"
  );

  // Дробное / целый ноль.
  checkStructure!(
    "a := 10.5 / 0", "a",
    type UFloat,
    value "10.5"
  );
  checkStructure!(
    "a := -10.5 / 0", "a",
    type Float,
    value "-10.5"
  );
  checkStructure!(
    "a := 10.5 / -0", "a",
    type UFloat,
    value "10.5"
  );
  checkStructure!(
    "a := -10.5 / -0", "a",
    type Float,
    value "-10.5"
  );
}

/// ABI типы как делитель и как делимое.
#[test]
fn divideByZeroAbi() -> ()
{
  // U8.
  checkStructure!(
    r#"
      zA: U8 = 0
      a := 10 / zA
    "#,
    "a",
    type UInt,
    value "10"
  );
  checkStructure!(
    r#"
      xA: U8 = 10
      a := xA / 0
    "#,
    "a",
    type UInt,
    value "10"
  );
  // U16.
  checkStructure!(
    r#"
      zB: U16 = 0
      a := 10 / zB
    "#,
    "a",
    type UInt,
    value "10"
  );
  checkStructure!(
    r#"
      xB: U16 = 10
      a := xB / 0
    "#,
    "a",
    type UInt,
    value "10"
  );
  // U32.
  checkStructure!(
    r#"
      zC: U32 = 0
      a := 10 / zC
    "#,
    "a",
    type UInt,
    value "10"
  );
  checkStructure!(
    r#"
      xC: U32 = 10
      a := xC / 0
    "#,
    "a",
    type UInt,
    value "10"
  );
  // U64.
  checkStructure!(
    r#"
      zD: U64 = 0
      a := 10 / zD
    "#,
    "a",
    type UInt,
    value "10"
  );
  checkStructure!(
    r#"
      xD: U64 = 10
      a := xD / 0
    "#,
    "a",
    type UInt,
    value "10"
  );
  // Usize.
  checkStructure!(
    r#"
      zE: Usize = 0
      a := 10 / zE
    "#,
    "a",
    type UInt,
    value "10"
  );
  checkStructure!(
    r#"
      xE: Usize = 10
      a := xE / 0
    "#,
    "a",
    type UInt,
    value "10"
  );

  // I8.
  checkStructure!(
    r#"
      zF: I8 = 0
      a := -10 / zF
    "#,
    "a",
    type Int,
    value "-10"
  );
  checkStructure!(
    r#"
      xF: I8 = -10
      a := xF / 0
    "#,
    "a",
    type Int,
    value "-10"
  );
  // I16.
  checkStructure!(
    r#"
      zG: I16 = 0
      a := -10 / zG
    "#,
    "a",
    type Int,
    value "-10"
  );
  checkStructure!(
    r#"
      xG: I16 = -10
      a := xG / 0
    "#,
    "a",
    type Int,
    value "-10"
  );
  // I32.
  checkStructure!(
    r#"
      zH: I32 = 0
      a := -10 / zH
    "#,
    "a",
    type Int,
    value "-10"
  );
  checkStructure!(
    r#"
      xH: I32 = -10
      a := xH / 0
    "#,
    "a",
    type Int,
    value "-10"
  );
  // I64.
  checkStructure!(
    r#"
      zI: I64 = 0
      a := -10 / zI
    "#,
    "a",
    type Int,
    value "-10"
  );
  checkStructure!(
    r#"
      xI: I64 = -10
      a := xI / 0
    "#,
    "a",
    type Int,
    value "-10"
  );
  // Isize.
  checkStructure!(
    r#"
      zJ: Isize = 0
      a := -10 / zJ
    "#,
    "a",
    type Int,
    value "-10"
  );
  checkStructure!(
    r#"
      xJ: Isize = -10
      a := xJ / 0
    "#,
    "a",
    type Int,
    value "-10"
  );

  // F32.
  checkStructure!(
    r#"
      zK: F32 = 0.0
      a := 10.5 / zK
    "#,
    "a",
    type UFloat,
    value "10.5"
  );
  checkStructure!(
    r#"
      xK: F32 = 10.5
      a := xK / 0.0
    "#,
    "a",
    type UFloat,
    value "10.5"
  );
  checkStructure!(
    r#"
      yK: F32 = 0.0
      a := -10.5 / yK
    "#,
    "a",
    type Float,
    value "-10.5"
  );

  // F64.
  checkStructure!(
    r#"
      zL: F64 = 0.0
      a := 10.5 / zL
    "#,
    "a",
    type UFloat,
    value "10.5"
  );
  checkStructure!(
    r#"
      xL: F64 = 10.5
      a := xL / 0.0
    "#,
    "a",
    type UFloat,
    value "10.5"
  );
  checkStructure!(
    r#"
      yL: F64 = 0.0
      a := -10.5 / yL
    "#,
    "a",
    type Float,
    value "-10.5"
  );
}

/// `Char`, `String`, `True`/`False`; цепочки и приоритет.
#[test]
fn divideByZeroOther() -> ()
{
  // `Char` и `String`.
  checkStructure!(
    "a := 'a' / 0", "a",
    type Char
  );
  checkStructure!(
    "a := \"abc\" / 0", "a",
    type String
  );
  checkStructure!(
    "a := 10 / 'a'", "a",
    type UInt,
    value "10"
  );
  checkStructure!(
    "a := True / False", "a",
    type UInt,
    value "1"
  );

  // Цепочки: каждый шаг слева направо.
  checkStructure!(
    "a := 100 / 0 / 2", "a",
    type UInt,
    value "50"
  );
  checkStructure!(
    "a := 100 / 5 / 0", "a",
    type UInt,
    value "20"
  );
  checkStructure!(
    "a := 100 / 0 / 0", "a",
    type UInt,
    value "100"
  );

  // Приоритет: деление раньше `+` и `-`.
  checkStructure!(
    "a := 10 - 6 / 0", "a",
    type UInt,
    value "4"
  );
  checkStructure!(
    "a := 10 + 6 / 0", "a",
    type UInt,
    value "16"
  );
  checkStructure!(
    "a := 1 / 0 + 2 / 0", "a",
    type UInt,
    value "3"
  );
  checkStructure!(
    "a := -1 - 1 / 0", "a",
    type Int,
    value "-2"
  );
}

// =================================================================================================
