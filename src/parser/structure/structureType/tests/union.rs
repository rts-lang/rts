use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::StructureType;
use crate::tokenizer::tools::splitByType::splitByType;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

/// Сравнивает типы через to_string(): у StructureType нет Debug, а печать
/// заодно показывает, как объединение выглядит в коде.
/// 
/// todo rewrite desc.
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
/// todo rewrite desc.
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
/// 
/// todo rewrite desc.
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

/// Литералы - это значения, а не типы (issue #59).
/// 
/// todo При поддержке литералов как типов - стоит изменить эти проверки.
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
  // Помещается без потерь, а не первый в списке:
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
  
  // Отрицательное в беззнаковый - приведение.
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