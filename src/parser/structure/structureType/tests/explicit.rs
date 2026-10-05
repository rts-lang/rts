use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::StructureType;
use crate::parser::testing::check;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// todo Я не уверен что normalize и тесты кода - это не одно и то же.
//  По сути normalize лишний? Или он проверяет что-то еще?
//  Если это устаревшее - его можно адаптировать под новый стиль просто.

/// Проверяет значение токена после normalizeToken() в явный тип.
fn normalize(
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

/// Явный тип зажимает значение в свои границы, даже если число больше u64 и i64 (#71).
#[test]
fn normalizeClamp() -> ()
{
  let big: &str = "44444444444444444444444444444444444444444444"; // 44 цифры
  let negBig: String = format!("-{}", big);
  
  // Обычный clamp
  normalize(TokenType::UInt, "300", StructureType::U8, "255");
  normalize(TokenType::Int, "-10", StructureType::U8, "0");
  normalize(TokenType::Int, "-300", StructureType::I8, "-128");
  
  // Больше u64
  normalize(TokenType::UInt, "18446744073709551616", StructureType::U64, &u64::MAX.to_string());
  normalize(TokenType::UInt, big, StructureType::U8, "255");
  normalize(TokenType::UInt, big, StructureType::I8, "127");
  normalize(TokenType::UInt, big, StructureType::I64, &i64::MAX.to_string());
  normalize(TokenType::UInt, big, StructureType::Usize, &usize::MAX.to_string());
  
  // Меньше i64
  normalize(TokenType::Int, &negBig, StructureType::I8, "-128");
  normalize(TokenType::Int, &negBig, StructureType::I64, &i64::MIN.to_string());
  normalize(TokenType::Int, &negBig, StructureType::Isize, &isize::MIN.to_string());
  normalize(TokenType::Int, &negBig, StructureType::U64, "0");
  
  // Не число - базовое значение
  normalize(TokenType::UInt, "abc", StructureType::U8, "0");
}

/// Большие числа в float сохраняют величину, а не сжимаются в u64 (#71).
#[test]
fn normalizeFloat() -> ()
{
  let big: &str = "44444444444444444444444444444444444444444444"; // 44 цифры
  normalize(TokenType::UInt, big, StructureType::F32, &f32::MAX.to_string());
  normalize(TokenType::UInt, big, StructureType::F64, &big.parse::<f64>().unwrap().to_string());
  
  // Бесконечность зажимается в границу типа.
  normalize(TokenType::Float, "-1e309", StructureType::F32, &f32::MIN.to_string());
  normalize(TokenType::UFloat, "1e309", StructureType::F64, &f64::MAX.to_string());
  normalize(TokenType::UFloat, "1e309", StructureType::U8, "255");
}

/// Float в целый тип: округляется и зажимается в границы типа;
///
/// Отрицательное значение остаётся для знаковых типов и становится 0 для беззнаковых.
#[test]
fn normalizeFloatToInteger() -> ()
{
  normalize(TokenType::UFloat, "5.4", StructureType::I8, "5");
  normalize(TokenType::UFloat, "5.5", StructureType::I8, "6");
  
  // Беззнаковые
  normalize(TokenType::Float, "-10.0", StructureType::U8, "0");
  normalize(TokenType::Float, "-5.5", StructureType::U8, "0");
  normalize(TokenType::Float, "-0.4", StructureType::U8, "0");
  
  // Знаковые принимают отрицательное, если оно в диапазоне.
  normalize(TokenType::Float, "-5.5", StructureType::I8, "-6");
  normalize(TokenType::Float, "-5.4", StructureType::I8, "-5");
  normalize(TokenType::Float, "-0.4", StructureType::I8, "0");
  normalize(TokenType::Float, "-1000000.7", StructureType::I64, "-1000001");
  
  // Вне диапазона - граница типа.
  normalize(TokenType::Float, "-200.5", StructureType::I8, "-128");
  normalize(TokenType::UFloat, "200.5", StructureType::I8, "127");
  normalize(TokenType::UFloat, "300.5", StructureType::U8, "255");
  normalize(TokenType::Float, "-1e309", StructureType::I64, &i64::MIN.to_string());
  normalize(TokenType::UFloat, "1e309", StructureType::I64, &i64::MAX.to_string());
  normalize(TokenType::UFloat, "1e309", StructureType::U64, &u64::MAX.to_string());
}

// =================================================================================================

/// Явный целый тип зажимает значение в свои границы.
#[test]
fn castInteger() -> ()
{
  check!(
    "a: U8 = 300",
    a,
    type TokenType::UInt,
    stype StructureType::U8,
    value "255"
  );
  check!(
    "b: U8 = -10",
    b,
    type TokenType::Int,
    stype StructureType::U8,
    value "0"
  );
  check!(
    "c: I8 = -300",
    c,
    type TokenType::Int,
    stype StructureType::I8,
    value "-128"
  );
  
  //
  check!(
    "d: U64 = 18446744073709551616",
    d,
    type TokenType::UInt,
    stype StructureType::U64,
    value "18446744073709551615"
  );
  check!(
    "e: Usize = 18446744073709551616",
    e,
    type TokenType::UInt,
    stype StructureType::Usize,
    value "18446744073709551615"
  );
  check!(
    "f: Isize = -9223372036854775809",
    f,
    type TokenType::Int,
    stype StructureType::Isize,
    value "-9223372036854775808"
  );
}

/// Число за u64/i64: сначала граница u64/i64, затем зажим в явный тип (#71).
#[test]
fn castBigNumbers() -> ()
{
  check!(
    "g: U8 = 44444444444444444444444444444444444444444444",
    g,
    type TokenType::UInt,
    stype StructureType::U8,
    value "255"
  );
  check!(
    "h: I8 = 44444444444444444444444444444444444444444444",
    h,
    type TokenType::UInt,
    stype StructureType::I8,
    value "127"
  );
  check!(
    "i: I64 = -44444444444444444444444444444444444444444444",
    i,
    type TokenType::Int,
    stype StructureType::I64,
    value "-9223372036854775808"
  );
  check!(
    "j: Usize = 44444444444444444444444444444444444444444444",
    j,
    type TokenType::UInt,
    stype StructureType::Usize,
    value "18446744073709551615"
  );
  check!(
    "k: Isize = -44444444444444444444444444444444444444444444",
    k,
    type TokenType::Int,
    stype StructureType::Isize,
    value "-9223372036854775808"
  );
}

/// Огромное число в F32 и Float за F64 в F32/U8: зажим в границу целевого типа.
#[test]
fn castFloats() -> ()
{
  check!(
    "l: F32 = 44444444444444444444444444444444444444444444",
    l,
    type TokenType::UInt,
    stype StructureType::F32,
    value "340282350000000000000000000000000000000"
  );
  check!(
    "m: F32 = -1.7976931348623157e309",
    m,
    type TokenType::Float,
    stype StructureType::F32,
    value "-340282350000000000000000000000000000000"
  );
  check!(
    "n: U8 = 1.7976931348623157e309",
    n,
    type TokenType::UFloat,
    stype StructureType::U8,
    value "255"
  );
}

/// Float в целый явный тип: округляется и зажимается;
/// знак сохраняется для знаковых типов, беззнаковые дают 0.
#[test]
fn castFloatToInteger() -> ()
{
  check!(
    "q: I8 = 5.4",
    q,
    type TokenType::UFloat,
    stype StructureType::I8,
    value "5"
  );
  check!(
    "r: I8 = 5.5",
    r,
    type TokenType::UFloat,
    stype StructureType::I8,
    value "6")
  ;
  check!(
    "s: U8 = -10.0",
    s,
    type TokenType::Float,
    stype StructureType::U8,
    value "0"
  );
  check!(
    "t: U8 = -5.5",
    t,
    type TokenType::Float,
    stype StructureType::U8,
    value "0"
  );
  check!(
    "u: I8 = -5.5",
    u,
    type TokenType::Float,
    stype StructureType::I8,
    value "-6"
  );
  check!(
    "v: I8 = -5.4",
    v,
    type TokenType::Float,
    stype StructureType::I8,
    value "-5"
  );
  check!(
    "w: I8 = -200.5",
    w,
    type TokenType::Float,
    stype StructureType::I8,
    value "-128"
  );
  check!(
    "x: I64 = -1000000.7",
    x,
    type TokenType::Float,
    stype StructureType::I64,
    value "-1000001"
  );
}

// =================================================================================================
