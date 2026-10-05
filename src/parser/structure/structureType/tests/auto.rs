use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::StructureType;
use crate::parser::testing::check;
use crate::tokenizer::tools::splitByType::splitByType;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

/// Число без знака получает наименьший подходящий ABI тип.
#[test]
fn autoUnsigned() -> ()
{
  // U8
  check!(
    "a = 0",
    a,
    type TokenType::UInt,
    stype StructureType::U8,
    val "0"
  );
  check!(
    "a = 255",
    a,
    type TokenType::UInt,
    stype StructureType::U8,
    val "255"
  );
  
  // U16
  check!(
    "a = 256",
    a,
    type TokenType::UInt,
    stype StructureType::U16,
    val "256"
  );
  check!(
    "a = 65535",
    a,
    type TokenType::UInt,
    stype StructureType::U16,
    val "65535"
  );
  
  // U32
  check!(
    "a = 65536",
    a,
    type TokenType::UInt,
    stype StructureType::U32,
    val "65536"
  );
  check!(
    "a = 4294967295",
    a,
    type TokenType::UInt,
    stype StructureType::U32,
    val "4294967295"
  );
  
  // U64
  check!(
    "a = 4294967296",
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    val "4294967296"
  );
  check!(
    "a = 18446744073709551615",
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    val "18446744073709551615"
  );
}

/// Число со знаком получает наименьший подходящий ABI тип.
#[test]
fn autoSigned() -> ()
{
  // I8
  check!(
    "a = -1",
    a,
    type TokenType::Int,
    stype StructureType::I8,
    val "-1"
  );
  check!(
    "a = -128",
    a,
    type TokenType::Int,
    stype StructureType::I8,
    val "-128"
  );
  
  // I16
  check!(
    "a = -129",
    a,
    type TokenType::Int,
    stype StructureType::I16,
    val "-129"
  );
  check!(
    "a = -32768",
    a,
    type TokenType::Int,
    stype StructureType::I16,
    val "-32768"
  );
  
  // I32
  check!(
    "a = -32769",
    a,
    type TokenType::Int,
    stype StructureType::I32,
    val "-32769"
  );
  check!(
    "a = -2147483648",
    a,
    type TokenType::Int,
    stype StructureType::I32,
    val "-2147483648"
  );
  
  // I64
  check!(
    "a = -2147483649",
    a,
    type TokenType::Int,
    stype StructureType::I64,
    val "-2147483649"
  );
  check!(
    "a = -9223372036854775808",
    a,
    type TokenType::Int,
    stype StructureType::I64,
    val "-9223372036854775808"
  );
}

/// Число за крайним ABI типом становится его границей (#71).
#[test]
fn autoSaturation() -> ()
{
  // Больше U64 - и очень большое число.
  check!(
    "a = 18446744073709551616",
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    val &u64::MAX.to_string()
  );
  check!(
    "a = 99999999999999999999999999999999999999999999",
    a,
    type TokenType::UInt,
    stype StructureType::U64,
    val &u64::MAX.to_string()
  );
  
  // Меньше I64 - и очень большое отрицательное число.
  check!(
    "a = -9223372036854775809",
    a,
    type TokenType::Int,
    stype StructureType::I64,
    val &i64::MIN.to_string()
  );
  check!(
    "a = -99999999999999999999999999999999999999999999",
    a,
    type TokenType::Int,
    stype StructureType::I64,
    val &i64::MIN.to_string()
  );
}

/// Usize и Isize не выводятся автоматически, только явным типом.
/// Они зависят от платформы.
#[test]
fn explicitPlatform() -> ()
{
  check!(
    "b: Usize = 18446744073709551616",
    b, type TokenType::UInt,
    stype StructureType::Usize,
    val &usize::MAX.to_string()
  );
  check!(
    "c: Isize = -9223372036854775809",
    c, type TokenType::Int,
    stype StructureType::Isize,
    val &isize::MIN.to_string()
  );
}

/// Дробное число получает F32, если помещается, иначе F64.
#[test]
fn autoFloat() -> ()
{
  // f32
  check!(
    "a = 0.0",
    a,
    type TokenType::UFloat,
    stype StructureType::F32
  );
  check!(
    "a = 1.5",
    a,
    type TokenType::UFloat,
    stype StructureType::F32,
    val "1.5"
  );
  check!(
    "a = 3.4028234663852886e38",
    a,
    type TokenType::UFloat,
    stype StructureType::F32
  );
  check!(
    "a = -3.4028234663852886e38",
    a,
    type TokenType::Float,
    stype StructureType::F32
  );
  
  // f64
  check!(
    "a = 1.7976931348623157e308",
    a,
    type TokenType::UFloat,
    stype StructureType::F64
  );
  check!(
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
  check!(
    "a = 1.7976931348623157e309",
    a,
    type TokenType::UFloat,
    stype StructureType::F64,
    val &format!("{:e}", f64::MAX)
  );
  check!(
    "a = -1.7976931348623157e309",
    a,
    type TokenType::Float,
    stype StructureType::F64,
    val &format!("{:e}", f64::MIN)
  );
}

// =================================================================================================