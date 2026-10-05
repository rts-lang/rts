use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::StructureType;
use crate::parser::testing::check;
use crate::tokenizer::tools::splitByType::splitByType;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

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

/// Сравнивает типы через to_string(): у StructureType нет Debug, а печать
/// заодно показывает, как объединение выглядит в коде.
fn isType(actual: StructureType, expected: StructureType) -> bool
{
  assert!(
    actual == expected,
    "Expected type '{}', got '{}'",
    expected.to_string(), actual.to_string()
  );
  true
}

/// Разбирает type-секцию объявления в StructureType (issue #59).
///
/// `a: U8` — это по-прежнему обычный одиночный тип, а не Union из одного
/// элемента: иначе поменялось бы поведение всех существующих объявлений.
fn parseType(code: &str) -> StructureType
{
  let mut buffer: Vec<u8> = code.as_bytes().to_vec();
  let lines: Vec< Arc<RwLock<Line>> > = crate::tokenizer::tokenizer::readTokensSimple(&mut buffer);

  for line in lines
  {
    let line: RwLockReadGuard<Line> = line.read().unwrap();
    if let Some(tokens) = &line.tokens
    {
      // Отрезаем всё после `:` — это и есть type-секция.
      let typeTokens: Vec<Token> = match splitByType(tokens.clone(), &[TokenType::Colon])
      {
        parts if parts.len() == 2 =>
          parts[1].tokens.clone().unwrap_or_default(),
        _ => continue
      };
      return StructureType::fromTypeTokens(&typeTokens);
    }
  }
  StructureType::None
}

/// Проверяет, что значение легло в объединение: подходящий вариант и результат.
fn union(
  tokenType: TokenType,
  data: &str,
  variants: Vec<StructureType>,
  expectedType: &StructureType,
  expectedData: &str
)
{
  let mut token: Token = Token::new(tokenType, String::from(data));
  let union: StructureType = StructureType::Union(variants.clone());
  let resultType: StructureType = Structure::normalizeUnion(&mut token, &union);

  let tokenData: String = token.getData().toString().unwrap_or_default();
  assert!(
    resultType == *expectedType && tokenData == expectedData,
    "For '{}' in '{}' the '{}' variant with value '{}' was expected, got '{}' with value '{}'",
    data, union.to_string(), expectedType.to_string(), expectedData, resultType.to_string(), tokenData
  );
}

// =================================================================================================

/// Один вариант — обычный тип; несколько — Union (issue #59).
#[test]
fn unionParse() -> ()
{
  isType(parseType("a: U8 = 10"), StructureType::U8);
  isType(parseType("a: String = \"x\""), StructureType::String);
  isType(
    parseType("a: U8 | String = 10"),
    StructureType::Union(vec![StructureType::U8, StructureType::String])
  );
  isType(
    parseType("b: U8|String = 10"), // Без пробелов.
    StructureType::Union(vec![StructureType::U8, StructureType::String])
  );
  isType(
    parseType("a: I8 | U8 | F64 | None = 1"),
    StructureType::Union(vec![
      StructureType::I8, StructureType::U8, StructureType::F64, StructureType::None
    ])
  );
  
  // Ключевые слова-типы без данных - настоящие имена типов.
  isType(
    parseType("a: UInt | Int = 1"),
    StructureType::Union(vec![
      StructureType::Custom(String::from("UInt")), StructureType::Custom(String::from("Int"))
    ])
  );
  
  // Повторы схлопываются.
  isType(
    parseType("a: U8 | U8 | String = 10"),
    StructureType::Union(vec![StructureType::U8, StructureType::String])
  );
}

/// Литералы - это значения, а не типы: они не поддерживаются (issue #59).
#[test]
fn unionLiteralsAreNotTypes() -> ()
{
  // Ни одного имени типа - тип не указан, объявление ведёт себя как `a = 10`.
  isType(parseType("a: 1 | 2 = 10"), StructureType::None);
  
  // Строковый литерал отбрасывается, `U8` остаётся единственным вариантом.
  isType(parseType("a: \"name\" | 10 = 10"), StructureType::None);
  isType(parseType("a: \"name\" | U8 = 10"), StructureType::U8);
  
  // Незакрытый `|` не ломает разбор.
  isType(parseType("a: U8 | = 10"), StructureType::U8);
}

/// Значение ложится в тот вариант, в который помещается как есть (issue #59).
#[test]
fn unionExactMatch() -> ()
{
  let u8String: Vec<StructureType> = vec![StructureType::U8, StructureType::String];
  union(
    TokenType::UInt,
    "10",
    u8String.clone(),
    &StructureType::U8,
    "10"
  );
  union(
    TokenType::String,
    "hi",
    u8String.clone(),
    &StructureType::String,
    "hi"
  );
  union(
    TokenType::UFloat,
    "1.5",
    vec![StructureType::U8, StructureType::F32],
    &StructureType::F32,
    "1.5"
  );

  // Из нескольких подходящих вариантов выбирается тот, в который значение.
  // помещается без потерь, а не первый в списке:
  // -10 помещается в I8, поэтому U8 | I8 даёт I8, а не зажатое в U8 ноль.
  union(
    TokenType::Int,
    "-10",
    vec![StructureType::U8, StructureType::I8],
    &StructureType::I8,
    "-10"
  );
  union(
    TokenType::UInt,
    "70000",
    vec![StructureType::U8, StructureType::U32],
    &StructureType::U32,
    "70000"
  );
}

/// Не подошёл ни один вариант - приводим в первый, куда приведение возможно (#59/#71).
#[test]
fn unionConvert() -> ()
{
  let u8String: Vec<StructureType> = vec![StructureType::U8, StructureType::String];
  
  // 300 не помещается в U8, но приводится в него с зажимом (#71).
  union(TokenType::UInt, "300",  u8String.clone(), &StructureType::U8, "255");
  
  // Отрицательное в беззнаковый - тоже приведение, не совпадение.
  union(TokenType::Int,  "-10",  u8String.clone(), &StructureType::U8, "0");
  
  // Float приводится в целый вариант с округлением.
  union(TokenType::UFloat, "1.5", u8String.clone(), &StructureType::U8, "2");
}

/// Ни один вариант не подошёл и привести нельзя - None (issue #59).
#[test]
fn unionNone() -> ()
{
  let u8String: Vec<StructureType> = vec![StructureType::U8, StructureType::String];
  
  // True/False не приводятся ни к числу, ни к строке.
  union(TokenType::True, "True",  u8String.clone(), &StructureType::None, "");
  union(TokenType::Link, "a.b",   u8String.clone(), &StructureType::None, "");
  
  // Явный `| None` в объединении: значение не подходит - всё равно None.
  union(TokenType::True, "True",  vec![StructureType::String, StructureType::None],
        &StructureType::None, "");
  
  // Само None в объединении - законный вариант.
  union(TokenType::None, "", vec![StructureType::String, StructureType::None],
        &StructureType::None, "");
}

/// Объединение не должно ломать обычные одиночные типы.
#[test]
fn unionSingleVariantBehavesLikeType() -> ()
{
  // Union из одного варианта - это просто этот тип.
  let one: StructureType = StructureType::Union(vec![StructureType::U8]);
  isType(
    StructureType::Union(one.variants()),
    StructureType::Union(vec![StructureType::U8])
  );
  union(
    TokenType::UInt,
    "300",
    vec![StructureType::U8],
    &StructureType::U8,
    "255"
  );
  
  // Пустое объединение равносильно отсутствию типа.
  assert!(
    StructureType::Union(vec![]).variants().is_empty(),
    "An empty union should produce an empty list of variants"
  );
}

/// Union печатается так же, как записывается в коде.
#[test]
fn unionToString() -> ()
{
  assert_eq!(StructureType::Union(
    vec![StructureType::U8, StructureType::String]
  ).to_string(), "U8 | String");
  assert_eq!(StructureType::Union(
    vec![StructureType::I8, StructureType::F64, StructureType::None]
  ).to_string(), "I8 | F64 | None");
  
  // Вложенный union схлопывается при разборе, но и сам печатается нормально.
  assert_eq!(StructureType::Union(vec![
    StructureType::Union(vec![StructureType::U8, StructureType::String]),
    StructureType::U16
  ]).to_string(), "U8 | String | U16");
}

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
  // Больше U64 - и очень большое число
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
  
  // Меньше I64 - и очень большое отрицательное число
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

/// Явный целый тип зажимает значение в свои границы.
#[test]
fn castInteger() -> ()
{
  check!(
    "a: U8 = 300",
    a,
    type TokenType::UInt,
    stype StructureType::U8,
    val "255"
  );
  check!(
    "b: U8 = -10",
    b,
    type TokenType::Int,
    stype StructureType::U8,
    val "0"
  );
  check!(
    "c: I8 = -300",
    c,
    type TokenType::Int,
    stype StructureType::I8,
    val "-128"
  );
  
  //
  check!(
    "d: U64 = 18446744073709551616",
    d,
    type TokenType::UInt,
    stype StructureType::U64,
    val "18446744073709551615"
  );
  check!(
    "e: Usize = 18446744073709551616",
    e,
    type TokenType::UInt,
    stype StructureType::Usize,
    val "18446744073709551615"
  );
  check!(
    "f: Isize = -9223372036854775809",
    f,
    type TokenType::Int,
    stype StructureType::Isize,
    val "-9223372036854775808"
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
    val "255"
  );
  check!(
    "h: I8 = 44444444444444444444444444444444444444444444",
    h,
    type TokenType::UInt,
    stype StructureType::I8,
    val "127"
  );
  check!(
    "i: I64 = -44444444444444444444444444444444444444444444",
    i,
    type TokenType::Int,
    stype StructureType::I64,
    val "-9223372036854775808"
  );
  check!(
    "j: Usize = 44444444444444444444444444444444444444444444",
    j,
    type TokenType::UInt,
    stype StructureType::Usize,
    val "18446744073709551615"
  );
  check!(
    "k: Isize = -44444444444444444444444444444444444444444444",
    k,
    type TokenType::Int,
    stype StructureType::Isize,
    val "-9223372036854775808"
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
    val "340282350000000000000000000000000000000"
  );
  check!(
    "m: F32 = -1.7976931348623157e309",
    m,
    type TokenType::Float,
    stype StructureType::F32,
    val "-340282350000000000000000000000000000000"
  );
  check!(
    "n: U8 = 1.7976931348623157e309",
    n,
    type TokenType::UFloat,
    stype StructureType::U8,
    val "255"
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
    val "5"
  );
  check!(
    "r: I8 = 5.5",
    r,
    type TokenType::UFloat,
    stype StructureType::I8,
    val "6")
  ;
  check!(
    "s: U8 = -10.0",
    s,
    type TokenType::Float,
    stype StructureType::U8,
    val "0"
  );
  check!(
    "t: U8 = -5.5",
    t,
    type TokenType::Float,
    stype StructureType::U8,
    val "0"
  );
  check!(
    "u: I8 = -5.5",
    u,
    type TokenType::Float,
    stype StructureType::I8,
    val "-6"
  );
  check!(
    "v: I8 = -5.4",
    v,
    type TokenType::Float,
    stype StructureType::I8,
    val "-5"
  );
  check!(
    "w: I8 = -200.5",
    w,
    type TokenType::Float,
    stype StructureType::I8,
    val "-128"
  );
  check!(
    "x: I64 = -1000000.7",
    x,
    type TokenType::Float,
    stype StructureType::I64,
    val "-1000001"
  );
}

// =================================================================================================