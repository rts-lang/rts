use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::parser::testing::checkStructure;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Тесты на приведение к `StructureType`:
// выражения справа приводится к границам указанным слева.

// =================================================================================================

// Юнит-тесты `castToken`: тот же cast, что и в тестах кода ниже,
// но напрямую, без парсера - на голом токене.

/// Проверяет значение токена после `castToken()` в явный тип.
fn castToken(
  tokenType: TokenType,
  data: &str,
  structureType: StructureType,
  expectedData: &str
) -> ()
{
  let mut token: Token = Token::new(tokenType, String::from(data));
  structureType.castToken(&mut token);
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
  castToken(
    TokenType::UInt, "300",
    StructureType::Primitive(PrimitiveType::U8), "255"
  );
  castToken(
    TokenType::Int, "-10",
    StructureType::Primitive(PrimitiveType::U8), "0"
  );
  castToken(
    TokenType::Int, "-300",
    StructureType::Primitive(PrimitiveType::I8), "-128"
  );
  
  // Больше `u64`.
  castToken(
    TokenType::UInt, "18446744073709551616",
    StructureType::Primitive(PrimitiveType::U64), &u64::MAX.to_string()
  );
  castToken(
    TokenType::UInt, big,
    StructureType::Primitive(PrimitiveType::U8), "255"
  );
  castToken(
    TokenType::UInt, big,
    StructureType::Primitive(PrimitiveType::I8), "127"
  );
  castToken(
    TokenType::UInt, big,
    StructureType::Primitive(PrimitiveType::I64), &i64::MAX.to_string()
  );
  castToken(
    TokenType::UInt, big,
    StructureType::Primitive(PrimitiveType::Usize), &usize::MAX.to_string()
  );
  
  // Меньше `i64`.
  castToken(
    TokenType::Int, &negBig,
    StructureType::Primitive(PrimitiveType::I8), "-128"
  );
  castToken(
    TokenType::Int, &negBig,
    StructureType::Primitive(PrimitiveType::I64), &i64::MIN.to_string()
  );
  castToken(
    TokenType::Int, &negBig,
    StructureType::Primitive(PrimitiveType::Isize), &isize::MIN.to_string()
  );
  castToken(
    TokenType::Int, &negBig,
    StructureType::Primitive(PrimitiveType::U64), "0"
  );
  
  // Не число - базовое значение.
  castToken(
    TokenType::UInt, "abc",
    StructureType::Primitive(PrimitiveType::U8), "0"
  );
}

/// Большие числа в `float`-рамке зажимаются в границу типа.
//
//  `UInt` за `u64` доходит до приведения уже как `u64::MAX`: токен насыщается
//  в границы своего собственного типа ещё при создании (`normalizeData`),
//  поэтому величина исходного числа до `float`-рамки не доходит - как и в
//  `castFloats`, где expression обрубает значение до `usize::MAX`.
#[test]
fn normalizeFloat() -> ()
{
  let big: &str = "44444444444444444444444444444444444444444444"; // 44 цифры
  castToken(
    TokenType::UInt, big,
    StructureType::Primitive(PrimitiveType::F32), &format!("{:?}", u64::MAX as f32)
  );
  castToken(
    TokenType::UInt, big,
    StructureType::Primitive(PrimitiveType::F64), &format!("{:?}", u64::MAX as f64)
  );
  
  // Бесконечность зажимается в границу типа.
  castToken(
    TokenType::Float, "-1e309",
    StructureType::Primitive(PrimitiveType::F32), &format!("{:?}", f32::MIN)
  );
  castToken(
    TokenType::UFloat, "1e309",
    StructureType::Primitive(PrimitiveType::F64), &format!("{:?}", f64::MAX)
  );
  castToken(
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
  castToken(
    TokenType::UFloat, "5.4",
    StructureType::Primitive(PrimitiveType::I8), "5"
  );
  castToken(
    TokenType::UFloat, "5.5",
    StructureType::Primitive(PrimitiveType::I8), "6"
  );
  
  // Беззнаковые.
  castToken(
    TokenType::Float, "-10.0",
    StructureType::Primitive(PrimitiveType::U8), "0"
  );
  castToken(
    TokenType::Float, "-5.5",
    StructureType::Primitive(PrimitiveType::U8), "0"
  );
  castToken(
    TokenType::Float, "-0.4",
    StructureType::Primitive(PrimitiveType::U8), "0"
  );
  
  // Знаковые принимают отрицательное, если оно в диапазоне.
  castToken(
    TokenType::Float, "-5.5",
    StructureType::Primitive(PrimitiveType::I8), "-6"
  );
  castToken(
    TokenType::Float, "-5.4",
    StructureType::Primitive(PrimitiveType::I8), "-5"
  );
  castToken(
    TokenType::Float, "-0.4",
    StructureType::Primitive(PrimitiveType::I8), "0"
  );
  castToken(
    TokenType::Float, "-1000000.7",
    StructureType::Primitive(PrimitiveType::I64), "-1000001"
  );
  
  // Вне диапазона - граница типа.
  castToken(
    TokenType::Float, "-200.5",
    StructureType::Primitive(PrimitiveType::I8), "-128"
  );
  castToken(
    TokenType::UFloat, "200.5",
    StructureType::Primitive(PrimitiveType::I8), "127"
  );
  castToken(
    TokenType::UFloat, "300.5",
    StructureType::Primitive(PrimitiveType::U8), "255"
  );
  castToken(
    TokenType::Float, "-1e309",
    StructureType::Primitive(PrimitiveType::I64), &i64::MIN.to_string()
  );
  castToken(
    TokenType::UFloat, "1e309",
    StructureType::Primitive(PrimitiveType::I64), &i64::MAX.to_string()
  );
  castToken(
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
    type UInt,
    stype U8,
    value "255"
  );
  checkStructure!(
    "b: U8 = -10", "b",
    type UInt,
    stype U8,
    value "0"
  );
  checkStructure!(
    "c: I8 = -300", "c",
    type Int,
    stype I8,
    value "-128"
  );
  
  //
  checkStructure!(
    "d: U64 = 18446744073709551616", "d",
    type UInt,
    stype U64,
    value "18446744073709551615"
  );
  checkStructure!(
    "e: Usize = 18446744073709551616", "e",
    type UInt,
    stype Usize,
    value "18446744073709551615"
  );
  checkStructure!(
    "f: Isize = -9223372036854775809", "f",
    type Int,
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
    type UInt,
    stype U8,
    value "255"
  );
  checkStructure!(
    "h: I8 = 44444444444444444444444444444444444444444444", "h",
    type Int,
    stype I8,
    value "127"
  );
  checkStructure!(
    "i: I64 = -44444444444444444444444444444444444444444444", "i",
    type Int,
    stype I64,
    value "-9223372036854775808"
  );
  checkStructure!(
    "j: Usize = 44444444444444444444444444444444444444444444", "j",
    type UInt,
    stype Usize,
    value "18446744073709551615"
  );
  checkStructure!(
    "k: Isize = -44444444444444444444444444444444444444444444", "k",
    type Int,
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
    type UFloat,
    stype F32,
    value &format!("{:?}", usize::MAX as f32)
  );
  checkStructure!(
    "m: F32 = -1.7976931348623157e309", "m",
    type Float,
    stype F32,
    value &format!("{:?}", f32::MIN)
  );
  checkStructure!(
    "n: U8 = 1.7976931348623157e309", "n",
    type UInt,
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
    type Int,
    stype I8,
    value "5"
  );
  checkStructure!(
    "r: I8 = 5.5", "r",
    type Int,
    stype I8,
    value "6")
  ;
  checkStructure!(
    "s: U8 = -10.0", "s",
    type UInt,
    stype U8,
    value "0"
  );
  checkStructure!(
    "t: U8 = -5.5", "t",
    type UInt,
    stype U8,
    value "0"
  );
  checkStructure!(
    "u: I8 = -5.5", "u",
    type Int,
    stype I8,
    value "-6"
  );
  checkStructure!(
    "v: I8 = -5.4", "v",
    type Int,
    stype I8,
    value "-5"
  );
  checkStructure!(
    "w: I8 = -200.5", "w",
    type Int,
    stype I8,
    value "-128"
  );
  checkStructure!(
    "x: I64 = -1000000.7", "x",
    type Int,
    stype I64,
    value "-1000001"
  );
}

// =================================================================================================

/// Явный строковый тип приводит значение к строке этого типа, как метод:
/// `a: String = \`raw\`` даёт токен `String` - так же, как `String(\`raw\`)`.
///
/// `CString` = `Pointer` (алиас): приведения для него нет, токен проходит как есть.
#[test]
fn castStrings() -> ()
{
  // Строка.
  checkStructure!(
    r#"a: String = "text""#, "a",
    type String,
    stype String,
    value "text"
  );
  checkStructure!(
    r#"b: CString = "text""#, "b",
    type String,
    stype CString,
    value "text"
  );
  checkStructure!(
    r#"c: RawString = "text""#, "c",
    type RawString,
    stype RawString,
    value "text"
  );

  // Сырая строка.
  checkStructure!(
    "d: String = `raw`", "d",
    type String,
    stype String,
    value "raw"
  );
  checkStructure!(
    "e: CString = `raw`", "e",
    type RawString,
    stype CString,
    value "raw"
  );
  checkStructure!(
    "f: RawString = `raw`", "f",
    type RawString,
    stype RawString,
    value "raw"
  );
}

// =================================================================================================
