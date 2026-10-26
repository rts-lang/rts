use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::parser::testing::checkStructure;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Тесты на приведение к `StructureType`:
// выражения справа приводится к границам указанным слева.

// =================================================================================================

// todo Я не уверен что normalize и тесты кода - это не одно и то же.
//  По сути normalize лишний? Или он проверяет что-то еще?
//  Если это устаревшее - его можно адаптировать под новый стиль просто.

/// Проверяет значение токена после `normalizeToken()` в явный тип.
fn normalizeToken(
  tokenType: TokenType,
  data: &str,
  structureType: StructureType,
  expectedData: &str
) -> ()
{
  let mut token: Token = Token::new(tokenType, String::from(data));
  Structure::normalizeToken(&mut token, structureType);
  let tokenData: String = token.getData().toString().unwrap_or_default();
  assert!(
    tokenData == expectedData,
    "For '{}' the value '{}' was expected, got '{}'",
    data, expectedData, tokenData
  );
}

// =================================================================================================

/// Явный тип зажимает значение в свои границы, даже если число больше `u64` и `i64`.
#[test]
fn normalize() -> ()
{
  let big: &str = "44444444444444444444444444444444444444444444"; // 44 цифры
  let negBig: String = format!("-{}", big);
  
  // Обычный `normalize`.
  normalizeToken(
    TokenType::UInt, "300",
    StructureType::Primitive(PrimitiveType::U8), "255"
  );
  normalizeToken(
    TokenType::Int, "-10",
    StructureType::Primitive(PrimitiveType::U8), "0"
  );
  normalizeToken(
    TokenType::Int, "-300",
    StructureType::Primitive(PrimitiveType::I8), "-128"
  );
  
  // Больше `u64`.
  normalizeToken(
    TokenType::UInt, "18446744073709551616",
    StructureType::Primitive(PrimitiveType::U64), &u64::MAX.to_string()
  );
  normalizeToken(
    TokenType::UInt, big,
    StructureType::Primitive(PrimitiveType::U8), "255"
  );
  normalizeToken(
    TokenType::UInt, big,
    StructureType::Primitive(PrimitiveType::I8), "127"
  );
  normalizeToken(
    TokenType::UInt, big,
    StructureType::Primitive(PrimitiveType::I64), &i64::MAX.to_string()
  );
  normalizeToken(
    TokenType::UInt, big,
    StructureType::Primitive(PrimitiveType::Usize), &usize::MAX.to_string()
  );
  
  // Меньше `i64`.
  normalizeToken(
    TokenType::Int, &negBig,
    StructureType::Primitive(PrimitiveType::I8), "-128"
  );
  normalizeToken(
    TokenType::Int, &negBig,
    StructureType::Primitive(PrimitiveType::I64), &i64::MIN.to_string()
  );
  normalizeToken(
    TokenType::Int, &negBig,
    StructureType::Primitive(PrimitiveType::Isize), &isize::MIN.to_string()
  );
  normalizeToken(
    TokenType::Int, &negBig,
    StructureType::Primitive(PrimitiveType::U64), "0"
  );
  
  // Не число - базовое значение.
  normalizeToken(
    TokenType::UInt, "abc",
    StructureType::Primitive(PrimitiveType::U8), "0"
  );
}

/// Большие числа в `float` сохраняют величину, а не сжимаются в `u64`.
#[test]
fn normalizeFloat() -> ()
{
  let big: &str = "44444444444444444444444444444444444444444444"; // 44 цифры
  normalizeToken(
    TokenType::UInt, big,
    StructureType::Primitive(PrimitiveType::F32), &f32::MAX.to_string()
  );
  normalizeToken(
    TokenType::UInt, big,
    StructureType::Primitive(PrimitiveType::F64), &big.parse::<f64>().unwrap().to_string()
  );
  
  // Бесконечность зажимается в границу типа.
  normalizeToken(
    TokenType::Float, "-1e309",
    StructureType::Primitive(PrimitiveType::F32), &f32::MIN.to_string()
  );
  normalizeToken(
    TokenType::UFloat, "1e309",
    StructureType::Primitive(PrimitiveType::F64), &f64::MAX.to_string()
  );
  normalizeToken(
    TokenType::UFloat, "1e309",
    StructureType::Primitive(PrimitiveType::U8), "255"
  );
}

/// `Float` в целый тип: округляется и зажимается в границы типа;
///
/// Отрицательное значение остаётся для знаковых типов и становится `0` для беззнаковых.
#[test]
fn normalizeFloatToInteger() -> ()
{
  normalizeToken(
    TokenType::UFloat, "5.4",
    StructureType::Primitive(PrimitiveType::I8), "5"
  );
  normalizeToken(
    TokenType::UFloat, "5.5",
    StructureType::Primitive(PrimitiveType::I8), "6"
  );
  
  // Беззнаковые.
  normalizeToken(
    TokenType::Float, "-10.0",
    StructureType::Primitive(PrimitiveType::U8), "0"
  );
  normalizeToken(
    TokenType::Float, "-5.5",
    StructureType::Primitive(PrimitiveType::U8), "0"
  );
  normalizeToken(
    TokenType::Float, "-0.4",
    StructureType::Primitive(PrimitiveType::U8), "0"
  );
  
  // Знаковые принимают отрицательное, если оно в диапазоне.
  normalizeToken(
    TokenType::Float, "-5.5",
    StructureType::Primitive(PrimitiveType::I8), "-6"
  );
  normalizeToken(
    TokenType::Float, "-5.4",
    StructureType::Primitive(PrimitiveType::I8), "-5"
  );
  normalizeToken(
    TokenType::Float, "-0.4",
    StructureType::Primitive(PrimitiveType::I8), "0"
  );
  normalizeToken(
    TokenType::Float, "-1000000.7",
    StructureType::Primitive(PrimitiveType::I64), "-1000001"
  );
  
  // Вне диапазона - граница типа.
  normalizeToken(
    TokenType::Float, "-200.5",
    StructureType::Primitive(PrimitiveType::I8), "-128"
  );
  normalizeToken(
    TokenType::UFloat, "200.5",
    StructureType::Primitive(PrimitiveType::I8), "127"
  );
  normalizeToken(
    TokenType::UFloat, "300.5",
    StructureType::Primitive(PrimitiveType::U8), "255"
  );
  normalizeToken(
    TokenType::Float, "-1e309",
    StructureType::Primitive(PrimitiveType::I64), &i64::MIN.to_string()
  );
  normalizeToken(
    TokenType::UFloat, "1e309",
    StructureType::Primitive(PrimitiveType::I64), &i64::MAX.to_string()
  );
  normalizeToken(
    TokenType::UFloat, "1e309",
    StructureType::Primitive(PrimitiveType::U64), &u64::MAX.to_string()
  );
}

// =================================================================================================

/// Явный целый тип зажимает значение в свои границы.
#[test]
fn castInteger() -> ()
{
  checkStructure!(
    "a: U8 = 300", "a",
    type TokenType::UInt,
    stype U8,
    value "255"
  );
  checkStructure!(
    "b: U8 = -10", "b",
    type TokenType::Int,
    stype U8,
    value "0"
  );
  checkStructure!(
    "c: I8 = -300", "c",
    type TokenType::Int,
    stype I8,
    value "-128"
  );
  
  //
  checkStructure!(
    "d: U64 = 18446744073709551616", "d",
    type TokenType::UInt,
    stype U64,
    value "18446744073709551615"
  );
  checkStructure!(
    "e: Usize = 18446744073709551616", "e",
    type TokenType::UInt,
    stype Usize,
    value "18446744073709551615"
  );
  checkStructure!(
    "f: Isize = -9223372036854775809", "f",
    type TokenType::Int,
    stype Isize,
    value "-9223372036854775808"
  );
}

/// Число за `u64`/`i64`: сначала граница `u64`/`i64`, затем зажим в явный тип.
#[test]
fn castBigNumbers() -> ()
{
  checkStructure!(
    "g: U8 = 44444444444444444444444444444444444444444444", "g",
    type TokenType::UInt,
    stype U8,
    value "255"
  );
  checkStructure!(
    "h: I8 = 44444444444444444444444444444444444444444444", "h",
    type TokenType::UInt,
    stype I8,
    value "127"
  );
  checkStructure!(
    "i: I64 = -44444444444444444444444444444444444444444444", "i",
    type TokenType::Int,
    stype I64,
    value "-9223372036854775808"
  );
  checkStructure!(
    "j: Usize = 44444444444444444444444444444444444444444444", "j",
    type TokenType::UInt,
    stype Usize,
    value "18446744073709551615"
  );
  checkStructure!(
    "k: Isize = -44444444444444444444444444444444444444444444", "k",
    type TokenType::Int,
    stype Isize,
    value "-9223372036854775808"
  );
}

/// Огромное число в `F32` и `Float` за `F64` в `F32`/`U8`: зажим в границу целевого типа.
#[test]
fn castFloats() -> ()
{
  // `UInt` токен бесконечный: в expression он обрубается до `usize::MAX`,
  // а `l: F32 =` приводит уже обрубленное значение к `f32`.
  checkStructure!(
    "l: F32 = 44444444444444444444444444444444444444444444", "l",
    type TokenType::UInt,
    stype F32,
    value &(usize::MAX as f32).to_string()
  );
  checkStructure!(
    "m: F32 = -1.7976931348623157e309", "m",
    type TokenType::Float,
    stype F32,
    value "-340282350000000000000000000000000000000"
  );
  checkStructure!(
    "n: U8 = 1.7976931348623157e309", "n",
    type TokenType::UFloat,
    stype U8,
    value "255"
  );
}

/// `Float` в целый явный тип: округляется и зажимается;
/// 
/// Знак сохраняется для знаковых типов, беззнаковые дают `0`.
#[test]
fn castFloatToInteger() -> ()
{
  checkStructure!(
    "q: I8 = 5.4", "q",
    type TokenType::UFloat,
    stype I8,
    value "5"
  );
  checkStructure!(
    "r: I8 = 5.5", "r",
    type TokenType::UFloat,
    stype I8,
    value "6")
  ;
  checkStructure!(
    "s: U8 = -10.0", "s",
    type TokenType::Float,
    stype U8,
    value "0"
  );
  checkStructure!(
    "t: U8 = -5.5", "t",
    type TokenType::Float,
    stype U8,
    value "0"
  );
  checkStructure!(
    "u: I8 = -5.5", "u",
    type TokenType::Float,
    stype I8,
    value "-6"
  );
  checkStructure!(
    "v: I8 = -5.4", "v",
    type TokenType::Float,
    stype I8,
    value "-5"
  );
  checkStructure!(
    "w: I8 = -200.5", "w",
    type TokenType::Float,
    stype I8,
    value "-128"
  );
  checkStructure!(
    "x: I64 = -1000000.7", "x",
    type TokenType::Float,
    stype I64,
    value "-1000001"
  );
}

// =================================================================================================

/// Явный тип строки не меняет токен, только тип структуры.
///
/// `CString` = `Pointer` (алиас).
#[test]
fn castStrings() -> ()
{
  // Строка.
  checkStructure!(
    r#"a: String = "text""#, "a",
    type TokenType::String,
    stype String,
    value "text"
  );
  checkStructure!(
    r#"b: CString = "text""#, "b",
    type TokenType::String,
    stype CString,
    value "text"
  );
  checkStructure!(
    r#"c: RawString = "text""#, "c",
    type TokenType::String,
    stype RawString,
    value "text"
  );

  // Сырая строка.
  checkStructure!(
    "d: String = `raw`", "d",
    type TokenType::RawString,
    stype String,
    value "raw"
  );
  checkStructure!(
    "e: CString = `raw`", "e",
    type TokenType::RawString,
    stype CString,
    value "raw"
  );
  checkStructure!(
    "f: RawString = `raw`", "f",
    type TokenType::RawString,
    stype RawString,
    value "raw"
  );
}

// =================================================================================================
