use crate::parser::structure::structureType::StructureType;
use crate::parser::testing::checkStructure;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Число без знака получает наименьший подходящий ABI тип.
#[test]
fn autoUnsigned() -> ()
{
  // U8
  checkStructure!(
    "a = 0",
    a,
    type TokenType::UInt,
    stype StructureType::U8,
    value "0"
  );
  checkStructure!(
    "a = 255",
    a,
    type TokenType::UInt,
    stype StructureType::U8,
    value "255"
  );
  
  // U16
  checkStructure!(
    "a = 256",
    a,
    type TokenType::UInt,
    stype StructureType::U16,
    value "256"
  );
  checkStructure!(
    "a = 65535",
    a,
    type TokenType::UInt,
    stype StructureType::U16,
    value "65535"
  );
  
  // U32
  checkStructure!(
    "a = 65536",
    a,
    type TokenType::UInt,
    stype StructureType::U32,
    value "65536"
  );
  checkStructure!(
    "a = 4294967295",
    a,
    type TokenType::UInt,
    stype StructureType::U32,
    value "4294967295"
  );
  
  // U64
  checkStructure!(
    "a = 4294967296",
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    value "4294967296"
  );
  checkStructure!(
    "a = 18446744073709551615",
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    value "18446744073709551615"
  );
}

/// Число со знаком получает наименьший подходящий ABI тип.
#[test]
fn autoSigned() -> ()
{
  // I8
  checkStructure!(
    "a = -1",
    a,
    type TokenType::Int,
    stype StructureType::I8,
    value "-1"
  );
  checkStructure!(
    "a = -128",
    a,
    type TokenType::Int,
    stype StructureType::I8,
    value "-128"
  );
  
  // I16
  checkStructure!(
    "a = -129",
    a,
    type TokenType::Int,
    stype StructureType::I16,
    value "-129"
  );
  checkStructure!(
    "a = -32768",
    a,
    type TokenType::Int,
    stype StructureType::I16,
    value "-32768"
  );
  
  // I32
  checkStructure!(
    "a = -32769",
    a,
    type TokenType::Int,
    stype StructureType::I32,
    value "-32769"
  );
  checkStructure!(
    "a = -2147483648",
    a,
    type TokenType::Int,
    stype StructureType::I32,
    value "-2147483648"
  );
  
  // I64
  checkStructure!(
    "a = -2147483649",
    a,
    type TokenType::Int,
    stype StructureType::I64,
    value "-2147483649"
  );
  checkStructure!(
    "a = -9223372036854775808",
    a,
    type TokenType::Int,
    stype StructureType::I64,
    value "-9223372036854775808"
  );
}

/// Число за крайним ABI типом становится его границей (#71).
#[test]
fn autoSaturation() -> ()
{
  // Больше U64 - и очень большое число.
  checkStructure!(
    "a = 18446744073709551616",
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    value &u64::MAX.to_string()
  );
  checkStructure!(
    "a = 99999999999999999999999999999999999999999999",
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    value &u64::MAX.to_string()
  );
  
  // Меньше I64 - и очень большое отрицательное число.
  checkStructure!(
    "a = -9223372036854775809",
    a,
    type TokenType::Int,
    stype StructureType::I64,
    value &i64::MIN.to_string()
  );
  checkStructure!(
    "a = -99999999999999999999999999999999999999999999",
    a,
    type TokenType::Int,
    stype StructureType::I64,
    value &i64::MIN.to_string()
  );
}

/// Usize и Isize не выводятся автоматически, только явным типом.
/// Они зависят от платформы.
#[test]
fn explicitPlatform() -> ()
{
  checkStructure!(
    "b: Usize = 18446744073709551616",
    b, type TokenType::UInt,
    stype StructureType::Usize,
    value &usize::MAX.to_string()
  );
  checkStructure!(
    "c: Isize = -9223372036854775809",
    c, type TokenType::Int,
    stype StructureType::Isize,
    value &isize::MIN.to_string()
  );
}

/// Дробное число получает F32, если помещается, иначе F64.
#[test]
fn autoFloat() -> ()
{
  // f32
  checkStructure!(
    "a = 0.0",
    a,
    type TokenType::UFloat,
    stype StructureType::F32
  );
  checkStructure!(
    "a = 1.5",
    a,
    type TokenType::UFloat,
    stype StructureType::F32,
    value "1.5"
  );
  checkStructure!(
    "a = 3.4028234663852886e38",
    a,
    type TokenType::UFloat,
    stype StructureType::F32
  );
  checkStructure!(
    "a = -3.4028234663852886e38",
    a,
    type TokenType::Float,
    stype StructureType::F32
  );
  
  // f64
  checkStructure!(
    "a = 1.7976931348623157e308",
    a,
    type TokenType::UFloat,
    stype StructureType::F64
  );
  checkStructure!(
    "a = -1.7976931348623157e308",
    a,
    type TokenType::Float,
    stype StructureType::F64
  );
}

/// Дробное число за F64 остаётся границей F64 (#71).
#[test]
fn autoFloatSaturation() -> ()
{
  checkStructure!(
    "a = 1.7976931348623157e309",
    a,
    type TokenType::UFloat,
    stype StructureType::F64,
    value &format!("{:e}", f64::MAX)
  );
  checkStructure!(
    "a = -1.7976931348623157e309",
    a,
    type TokenType::Float,
    stype StructureType::F64,
    value &format!("{:e}", f64::MIN)
  );
}

// =================================================================================================