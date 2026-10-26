use crate::parser::testing::checkStructure;
// =================================================================================================

//Тесты на получение `StructureType` для структуры:
// левая часть выводится по правой части выражения.
// 
// Лимиты `StructureType` уже включены сюда.

// =================================================================================================

/// Число без знака получает наименьший подходящий ABI тип.
#[test]
fn autoUnsigned() -> ()
{
  // U8.
  checkStructure!(
    "a := 0", "a",
    type UInt,
    stype U8,
    value "0"
  );
  checkStructure!(
    "a := 255", "a",
    type UInt,
    stype U8,
    value "255"
  );
  
  // U16.
  checkStructure!(
    "a := 256", "a",
    type UInt,
    stype U16,
    value "256"
  );
  checkStructure!(
    "a := 65535", "a",
    type UInt,
    stype U16,
    value "65535"
  );
  
  // U32.
  checkStructure!(
    "a := 65536", "a",
    type UInt,
    stype U32,
    value "65536"
  );
  checkStructure!(
    "a := 4294967295", "a",
    type UInt,
    stype U32,
    value "4294967295"
  );
  
  // U64.
  checkStructure!(
    "a := 4294967296", "a",
    type UInt,
    stype U64,
    value "4294967296"
  );
  checkStructure!(
    "a := 18446744073709551615", "a",
    type UInt,
    stype U64,
    value "18446744073709551615"
  );
}

/// Число со знаком получает наименьший подходящий ABI тип.
#[test]
fn autoSigned() -> ()
{
  // I8.
  checkStructure!(
    "a := -1", "a",
    type Int,
    stype I8,
    value "-1"
  );
  checkStructure!(
    "a := -128", "a",
    type Int,
    stype I8,
    value "-128"
  );
  
  // I16.
  checkStructure!(
    "a := -129", "a",
    type Int,
    stype I16,
    value "-129"
  );
  checkStructure!(
    "a := -32768", "a",
    type Int,
    stype I16,
    value "-32768"
  );
  
  // I32.
  checkStructure!(
    "a := -32769", "a",
    type Int,
    stype I32,
    value "-32769"
  );
  checkStructure!(
    "a := -2147483648", "a",
    type Int,
    stype I32,
    value "-2147483648"
  );
  
  // I64.
  checkStructure!(
    "a := -2147483649", "a",
    type Int,
    stype I64,
    value "-2147483649"
  );
  checkStructure!(
    "a := -9223372036854775808", "a",
    type Int,
    stype I64,
    value "-9223372036854775808"
  );
}

// =================================================================================================

/// Число за крайним ABI типом становится его границей.
#[test]
fn autoSaturation() -> ()
{
  // Больше `U64` - и очень большое число.
  checkStructure!(
    "a := 18446744073709551616", "a",
    type UInt,
    stype U64,
    value &u64::MAX.to_string()
  );
  checkStructure!(
    "a := 99999999999999999999999999999999999999999999", "a",
    type UInt,
    stype U64,
    value &u64::MAX.to_string()
  );
  
  // Меньше `I64` - и очень большое отрицательное число.
  checkStructure!(
    "a := -9223372036854775809", "a",
    type Int,
    stype I64,
    value &i64::MIN.to_string()
  );
  checkStructure!(
    "a := -99999999999999999999999999999999999999999999", "a",
    type Int,
    stype I64,
    value &i64::MIN.to_string()
  );
}

/// `Usize` и `Isize` не выводятся автоматически, только явным типом;
/// Они зависят от платформы.
#[test]
fn explicitPlatform() -> ()
{
  checkStructure!(
    "b: Usize = 18446744073709551616", "b",
    type UInt,
    stype Usize,
    value &usize::MAX.to_string()
  );
  checkStructure!(
    "c: Isize = -9223372036854775809", "c",
    type Int,
    stype Isize,
    value &isize::MIN.to_string()
  );
}

// =================================================================================================

/// Дробное число получает `F32`, если помещается, иначе `F64`.
#[test]
fn autoFloat() -> ()
{
  // f32
  checkStructure!(
    "a := 0.0", "a",
    type UFloat,
    stype F32
  );
  checkStructure!(
    "a := 1.5", "a",
    type UFloat,
    stype F32,
    value "1.5"
  );
  checkStructure!(
    "a := 3.4028234663852886e38", "a",
    type UFloat,
    stype F32
  );
  checkStructure!(
    "a := -3.4028234663852886e38", "a",
    type Float,
    stype F32
  );
  
  // f64
  checkStructure!(
    "a := 1.7976931348623157e308", "a",
    type UFloat,
    stype F64
  );
  checkStructure!(
    "a := -1.7976931348623157e308", "a",
    type Float,
    stype F64
  );
}

/// Дробное число за `F64` остаётся границей `F64`.
#[test]
fn autoFloatSaturation() -> ()
{
  checkStructure!(
    "a := 1.7976931348623157e309", "a",
    type UFloat,
    stype F64,
    value &format!("{:e}", f64::MAX)
  );
  checkStructure!(
    "a := -1.7976931348623157e309", "a",
    type Float,
    stype F64,
    value &format!("{:e}", f64::MIN)
  );
}

// =================================================================================================

/// Строка получает `String`, сырая строка - `RawString`.
///
/// `CString` не выводится: у него нет токена, только явный тип.
#[test]
fn autoString() -> ()
{
  checkStructure!(
    r#"a := "text""#, "a",
    type String,
    stype String,
    value "text"
  );
  checkStructure!(
    "b := `raw`", "b",
    type RawString,
    stype RawString,
    value "raw"
  );

  // Пустая строка.
  checkStructure!(
    r#"c := """#, "c",
    type String,
    stype String,
    value ""
  );

  // Dynamic.
  checkStructure!(
    r#"d~~ := "text""#, "d",
    stype String,
    value "text"
  );
}

/// Форматированная строка после подстановки получает тот же тип, что и обычная.
#[test]
fn autoFormattedString() -> ()
{
  checkStructure!(
    r#"a := f"x{1}""#, "a",
    type String,
    stype String,
    value "x1"
  );
  checkStructure!(
    "b := f`x{1}`", "b",
    type RawString,
    stype RawString,
    value "x1"
  );
}

/// У строки без аннотации есть поля `pointer` и `length`, как у `a: String`.
#[test]
fn autoStringFields() -> ()
{
  checkStructure!(
    r#"a := "text""#, "a.length",
    value "4"
  );
  checkStructure!(
    r#"a := "text""#, "a.pointer",
    stype Pointer
  );
}

// =================================================================================================
