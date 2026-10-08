use crate::parser::structure::tokenValue::calculate::calculate;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Это тесты calculate. Они работают через вычисление, а не через токены.
//
// Это дает тестирование на самом низком уровне в обход токенайзера и парсера.
// Т.е. тестирует саму реализацию операций математики и логики.
//
// todo Здесь не хватает тестов, а еще бы я раскрыл их т.к. они длинные.

// =================================================================================================

/// Проверяет тип и значение результата операции.
fn check(
  op: TokenType, 
  left: (TokenType, &str), 
  right: (TokenType, &str), 
  expectedType: TokenType, 
  expectedData: &str
)
{
  let result: Token = calculate(
    &op,
    &Token::new(left.0, String::from(left.1)),
    &Token::new(right.0, String::from(right.1))
  );
  let data: String = result.getData().toString().unwrap_or_default();
  assert!(
    *result.getDataType() == expectedType && data == expectedData,
    "'{} and {}' expected '{}', got '{}'",
    left.1, right.1, expectedData, data
  );
}

// =================================================================================================

/// Сравнения возвращают True/False, а не число (#39, #65).
#[test]
fn comparison() -> ()
{
  let t: TokenType = TokenType::True;
  let f: TokenType = TokenType::False;
  check(TokenType::Equals,              (TokenType::UInt, "1"), (TokenType::UInt, "1"), t, "True");
  check(TokenType::Equals,              (TokenType::UInt, "1"), (TokenType::UInt, "2"), f, "False");
  check(TokenType::NotEquals,           (TokenType::UInt, "1"), (TokenType::UInt, "1"), f, "False");
  check(TokenType::NotEquals,           (TokenType::UInt, "1"), (TokenType::UInt, "2"), t, "True");
  check(TokenType::LessThan,            (TokenType::UInt, "1"), (TokenType::UInt, "2"), t, "True");
  check(TokenType::GreaterThan,         (TokenType::UInt, "1"), (TokenType::UInt, "2"), f, "False");
  check(TokenType::LessThanOrEquals,    (TokenType::UInt, "3"), (TokenType::UInt, "2"), f, "False");
  check(TokenType::GreaterThanOrEquals, (TokenType::UInt, "2"), (TokenType::UInt, "2"), t, "True");
}

/// Явная проверка на пустоту: `x = None` / `x != None` (#39).
#[test]
fn comparisonNone() -> ()
{
  let t: TokenType = TokenType::True;
  let f: TokenType = TokenType::False;
  check(TokenType::Equals,    (TokenType::None, ""),  (TokenType::None, ""), t, "True");
  check(TokenType::NotEquals, (TokenType::None, ""),  (TokenType::None, ""), f, "False");
  check(TokenType::Equals,    (TokenType::UInt, "5"), (TokenType::None, ""), f, "False");
  check(TokenType::NotEquals, (TokenType::UInt, "5"), (TokenType::None, ""), t, "True");
  // 0 - это не пустота.
  check(TokenType::Equals,    (TokenType::UInt, "0"), (TokenType::None, ""), f, "False");
}

// =================================================================================================

/// Умножение смешанных типов: `*` в выражениях пока отключён (structure.rs, expressionWith),
/// поэтому Value::Mul проверяется здесь напрямую, до его включения.
#[test]
fn multiplyMixed() -> ()
{
  // Int * UFloat: раньше стояло деление x / y, и -3 * 2.5 давало -1.2.
  check(TokenType::Multiply, (TokenType::Int,   "-3"), (TokenType::UFloat, "2.5"),  TokenType::Float,  "-7.5");
  check(TokenType::Multiply, (TokenType::UFloat, "2.5"), (TokenType::Int,   "-3"),  TokenType::Float,  "-7.5");
  // Соседние ветки
  check(TokenType::Multiply, (TokenType::UInt,  "3"),  (TokenType::UFloat, "2.5"),  TokenType::UFloat, "7.5");
  check(TokenType::Multiply, (TokenType::Int,   "-3"), (TokenType::Float,  "-2.5"), TokenType::Float,  "7.5");
  check(TokenType::Multiply, (TokenType::UInt,  "3"),  (TokenType::Float,  "-2.5"), TokenType::Float,  "-7.5");
}

/// Умножение коммутативно: a * b == b * a для любой пары числовых типов.
#[test]
fn multiplyCommutative() -> ()
{
  let samples: [(TokenType, &str); 4] = [
    (TokenType::UInt,   "3"),
    (TokenType::Int,    "-3"),
    (TokenType::UFloat, "2.5"),
    (TokenType::Float,  "-2.5"),
  ];
  for left in samples 
  {
    for right in samples 
    {
      let leftToken:  Token = Token::new(left.0,  String::from(left.1));
      let rightToken: Token = Token::new(right.0, String::from(right.1));
      let ab: Token = calculate(&TokenType::Multiply, &leftToken,  &rightToken);
      let ba: Token = calculate(&TokenType::Multiply, &rightToken, &leftToken);
      let abData: String = ab.getData().toString().unwrap_or_default();
      let baData: String = ba.getData().toString().unwrap_or_default();
      assert!(
        abData == baData && ab.getDataType() == ba.getDataType(),
        "'{} * {}' = '{}', but '{} * {}' = '{}'",
        left.1, right.1, abData, right.1, left.1, baData
      );
    }
  }
}

// =================================================================================================

/// Результат за границей f64 зажимается, а не становится inf/NaN (#71).
#[test]
fn floatOverflow() -> ()
{
  let max: String = f64::MAX.to_string();
  let min: String = f64::MIN.to_string();
  // Сумма за границей.
  check(TokenType::Plus,  (TokenType::UFloat, "1e308"),  (TokenType::UFloat, "1e308"), TokenType::UFloat, &max);
  check(TokenType::Minus, (TokenType::Float,  "-1e308"), (TokenType::UFloat, "1e308"), TokenType::Float,  &min);
  // Примитив за границей зажимается до операции: MAX - MAX = 0, а не inf - inf.
  check(TokenType::Minus, (TokenType::UFloat, "1e309"),  (TokenType::UFloat, "1e309"),  TokenType::UFloat, "0");
  // todo Это бред, inf нет в RTS, если оно есть где-то в обработке - это нужно удалить.
  check(TokenType::Minus, (TokenType::UFloat, "inf"),    (TokenType::UFloat, "inf"),    TokenType::UFloat, "0");
  check(TokenType::Minus, (TokenType::Float,  "-1e309"), (TokenType::Float,  "-1e309"), TokenType::Float,  "0");
  // Отрицательный UFloat не теряется: из него получается Float.
  check(TokenType::Minus, (TokenType::UFloat, "1.0"),    (TokenType::UFloat, "2.0"),    TokenType::Float,  "-1");
}

// =================================================================================================
