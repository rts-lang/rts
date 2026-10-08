use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::{StructureType, PrimitiveKind};
use crate::tokenizer::tools::splitByType::splitByType;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

// todo desc + я бы раскрыл все строки.
//
// todo Есть баг еще вроде как с U8 = None будет 0, что ошибка. НО оно не к этому к StructureType.
//
// todo Еще тут есть вопросы, возможно использовать общие макросы + вынести часть проверок туда отсюда.

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
      // Как в парсере: сначала режем по `=`, и только левая часть - это объявление;
      // правая часть - выражение, и в type-секцию она не попадает.
      let left: Vec<Token> = match tokens.iter().position(|token: &Token| *token.getDataType() == TokenType::Equals)
      {
        Some(position) => tokens[..position].to_vec(),
        None => tokens.clone()
      };

      // Отрезаем всё после `:` — это и есть type-секция.
      let typeTokens: Vec<Token> = match splitByType(left, &[TokenType::Colon])
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

/// Числовой примитив как вариант типизации.
fn number(data: &str) -> StructureType
{
  StructureType::Primitive(PrimitiveKind::Number, String::from(data))
}

/// Строковый примитив как вариант типизации.
fn text(data: &str) -> StructureType
{
  StructureType::Primitive(PrimitiveKind::Text, String::from(data))
}

/// Примитивы - варианты типизации наравне с именами типов (type-секция слева от `=`).
#[test]
fn unionPrimitiveParse() -> ()
{
  isType(parseType("a: 10 | 20 = 10"), StructureType::Union(vec![number("10"), number("20")]));
  isType(parseType("a: 10|20 = 10"), StructureType::Union(vec![number("10"), number("20")])); // Без пробелов.
  isType(parseType("a: \"text\" | 10 = 10"), StructureType::Union(vec![text("text"), number("10")]));
  isType(parseType("a: \"name\" | U8 = 10"), StructureType::Union(vec![text("name"), StructureType::U8]));
  isType(parseType("a: U8 | 10 = 10"), StructureType::Union(vec![StructureType::U8, number("10")]));
  isType(parseType("a: 1.5 | 2 = 2"), StructureType::Union(vec![number("1.5"), number("2")]));
  // Отрицательное число - один токен Int.
  isType(parseType("a: -1 | 1 = 1"), StructureType::Union(vec![number("-1"), number("1")]));
  // Одиночный примитив - Union из одного варианта, иначе значение не проверялось бы.
  isType(parseType("a: 10 = 10"), StructureType::Union(vec![number("10")]));
  isType(parseType("a: \"x\" = \"x\""), StructureType::Union(vec![text("x")]));
  // Повторы схлопываются.
  isType(parseType("a: 10 | 10 | 20 = 10"), StructureType::Union(vec![number("10"), number("20")]));
  // Незакрытый `|` не ломает разбор.
  isType(parseType("a: U8 | = 10"), StructureType::U8);
  isType(parseType("a: 10 | = 10"), StructureType::Union(vec![number("10")]));
}

/// Значение, точно совпавшее с примитивом, остаётся как есть;
/// структура хранит естественный тип значения, а не сам примитив.
#[test]
fn unionPrimitiveMatch() -> ()
{
  let tenTwenty: Vec<StructureType> = vec![number("10"), number("20")];
  // stype = сам примитив, не natural U8/F32.
  union(TokenType::UInt, "10", tenTwenty.clone(), &number("10"), "10");
  union(TokenType::UInt, "20", tenTwenty.clone(), &number("20"), "20");
  // Числа равны по значению, а не по записи.
  union(TokenType::UFloat, "10.0", tenTwenty.clone(), &number("10"), "10.0");
  union(TokenType::Int, "-1", vec![number("-1"), number("1")], &number("-1"), "-1");
  union(TokenType::UInt, "1", vec![number("-1"), number("1")], &number("1"), "1");

  let textTen: Vec<StructureType> = vec![text("x"), number("10")];
  union(TokenType::String, "x", textTen.clone(), &text("x"), "x");
  union(TokenType::UInt, "10", textTen.clone(), &number("10"), "10");
}

/// Значение вне примитивов - `None` (#71): без приведения, без зажима и без явного `None`.
#[test]
fn unionPrimitiveNone() -> ()
{
  let tenTwenty: Vec<StructureType> = vec![number("10"), number("20")];
  // Ближайшее число не подставляется: только точное совпадение.
  union(TokenType::UInt,   "15",   tenTwenty.clone(), &StructureType::None, "");
  union(TokenType::UInt,   "300",  tenTwenty.clone(), &StructureType::None, "");
  union(TokenType::UFloat, "10.5", tenTwenty.clone(), &StructureType::None, "");
  // Другой вид значения.
  union(TokenType::String, "text", tenTwenty.clone(), &StructureType::None, "");
  union(TokenType::String, "10",   tenTwenty.clone(), &StructureType::None, "");
  union(TokenType::True,   "True", tenTwenty.clone(), &StructureType::None, "");
  // `None` как значение - тоже `None`, хотя в объединении он не указан.
  union(TokenType::None,   "",     tenTwenty.clone(), &StructureType::None, "");

  let textTen: Vec<StructureType> = vec![text("x"), number("10")];
  union(TokenType::String, "y",    textTen.clone(), &StructureType::None, "");
  union(TokenType::True,   "True", textTen.clone(), &StructureType::None, "");
  union(TokenType::UInt,   "11",   textTen.clone(), &StructureType::None, "");
}

/// Примитив вместе с типом: сначала точный примитив, потом тип как есть, потом приведение.
#[test]
fn unionPrimitiveWithType() -> ()
{
  let u8Ten: Vec<StructureType> = vec![StructureType::U8, number("10")];
  // Приоритет: примитив раньше типа.
  let mut token: Token = Token::new(TokenType::UInt, String::from("10"));
  assert!(
    Structure::matchUnion(&mut token, &StructureType::Union(u8Ten.clone())) == number("10"),
    "The exact primitive must win over the type variant"
  );
  // Не примитив - работают правила типов: приведение с зажимом (#71).
  union(TokenType::UInt,   "11",  u8Ten.clone(), &StructureType::U8, "11");
  union(TokenType::UInt,   "300", u8Ten.clone(), &StructureType::U8, "255");
  union(TokenType::Int,    "-5",  u8Ten.clone(), &StructureType::U8, "0");
  // Строковый примитив рядом с типом: матч примитива → stype = примитив.
  let textU8: Vec<StructureType> = vec![text("name"), StructureType::U8];
  union(TokenType::String, "name", textU8.clone(), &text("name"), "name");
  union(TokenType::String, "other", textU8.clone(), &StructureType::None, "");
  union(TokenType::UInt,   "300",  textU8.clone(), &StructureType::U8, "255");
}

/// Union печатается так же, как записывается в коде.
#[test]
fn unionPrimitiveToString() -> ()
{
  assert_eq!(StructureType::Union(vec![number("10"), number("20")]).to_string(), "10 | 20");
  assert_eq!(StructureType::Union(vec![text("text"), StructureType::U8]).to_string(), "\"text\" | U8");
}

/// Смешанный union: тип + примитив — stype примитив при точном матче.
#[test]
fn unionPrimitiveMixedType() -> ()
{
  let stringTen: Vec<StructureType> = vec![StructureType::String, number("10")];
  union(TokenType::UInt, "10", stringTen.clone(), &number("10"), "10");
  union(TokenType::String, "hi", stringTen.clone(), &StructureType::String, "hi");
  union(TokenType::UInt, "11", stringTen.clone(), &StructureType::None, "");

  let stringText: Vec<StructureType> = vec![StructureType::String, text("text")];
  union(TokenType::String, "text", stringText.clone(), &text("text"), "text");
  union(TokenType::String, "other", stringText.clone(), &StructureType::String, "other");
}

/// abiType сводит примитив к storage-форме.
#[test]
fn primitiveAbiType() -> ()
{
  assert!(text("text").abiType() == StructureType::String);
  assert!(number("10").abiType() == StructureType::U8);
  assert!(number("-1").abiType() == StructureType::I8);
  assert!(StructureType::U8.abiType() == StructureType::U8);
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
