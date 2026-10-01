use std::num::IntErrorKind;
use crate::parser::structure::tokenValue::uf64::*;
use crate::parser::structure::tokenValue::value::Value;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Это вычисление на основе Value от токенов.
// Это отдельные вещи от Structure - потому что это внешний (над ним) инструмент.

// =================================================================================================

/// Вычисляет по математической операции значение и тип нового токена из двух
pub fn calculate(op: &TokenType, leftToken: &Token, rightToken: &Token) -> Token 
{
  // Получаем значение левой части выражения
  let leftTokenDataType: TokenType = *leftToken.getDataType();
  let leftValue: Value = getValue(leftToken.getData().toString().unwrap_or_default(), &leftTokenDataType);
  // Получаем значение правой части выражения
  let rightTokenDataType: TokenType = *rightToken.getDataType();
  let rightValue: Value = getValue(rightToken.getData().toString().unwrap_or_default(), &rightTokenDataType);
  // Получаем значение выражения, а также предварительный тип
  let mut resultType: TokenType = TokenType::UInt;
  let mut resultValue: String = match *op 
  {
    TokenType::Plus     => (leftValue + rightValue).to_string(),
    TokenType::Minus    => (leftValue - rightValue).to_string(),
    TokenType::Multiply => (leftValue * rightValue).to_string(),
    TokenType::Divide   => (leftValue / rightValue).to_string(),
    TokenType::Inclusion => 
    { 
      resultType = TokenType::Bool;
      if leftValue.toBool() || rightValue.toBool() {
        String::from("1")
      } else {
        String::from("0")
      }
    }
    TokenType::Joint => 
    { 
      resultType = TokenType::Bool;
      if leftValue.toBool() && rightValue.toBool() {
        String::from("1")
      } else {
        String::from("0")
      }
    }
    TokenType::Equals => 
    { 
      resultType = TokenType::Bool;
      if leftValue == rightValue {
        String::from("1")
      } else {
        String::from("0")
      }
    }
    TokenType::NotEquals => 
    { 
      resultType = TokenType::Bool;
      if leftValue != rightValue {
        String::from("1")
      } else {
        String::from("0")
      }
    }
    TokenType::GreaterThan => 
    { 
      resultType = TokenType::Bool;
      if leftValue > rightValue {
        String::from("1")
      } else {
        String::from("0")
      }
    }
    TokenType::LessThan => 
    { 
      resultType = TokenType::Bool;
      if leftValue < rightValue {
        String::from("1")
      } else {
        String::from("0")
      }
    }
    TokenType::GreaterThanOrEquals => 
    { 
      resultType = TokenType::Bool;
      if leftValue >= rightValue {
        String::from("1")
      } else {
        String::from("0")
      }
    }
    TokenType::LessThanOrEquals => 
    { 
      resultType = TokenType::Bool;
      if leftValue <= rightValue {
        String::from("1")
      } else {
        String::from("0")
      }
    }
    _ => "0".to_string(),
  };
  // После того как значение было получено,
  // Смотрим какой точно тип выдать новому токену
  if resultType != TokenType::Bool 
  {
    if leftTokenDataType == TokenType::String || rightTokenDataType == TokenType::String
    {
      resultType = TokenType::String;
    } else
    if matches!(leftTokenDataType, TokenType::Int | TokenType::UInt) &&
        rightTokenDataType == TokenType::Char
    { //
      resultType = leftTokenDataType;
    } else
    if leftTokenDataType == TokenType::Char
    {
      resultType = TokenType::Char;
    } else
    if leftTokenDataType == TokenType::UFloat || rightTokenDataType == TokenType::UFloat
    {
      // Проверяем смену типа;
      // TokenType::UFloat не ограничен - это тип токена, а не Value::UFloat(uf64).
      // Ограничение по размеру накладывается структурой (F32/F64), а не здесь.
      // Поэтому тип результата зависит только от знака строкового значения.
      if resultValue.starts_with('-') {
        resultType = TokenType::Float;
      } else {
        resultType = TokenType::UFloat;
      }
    } else
    if leftTokenDataType == TokenType::Float || rightTokenDataType == TokenType::Float
    {
      resultType = TokenType::Float;
    } else
    if leftTokenDataType == TokenType::UInt || rightTokenDataType == TokenType::UInt
    {
      // Проверяем смену типа;
      // TokenType::UInt не ограничен сверху - это тип токена, а не Value::UInt(u64).
      // Ограничение по размеру накладывается структурой (USize/ABI), а не здесь.
      // Поэтому тип результата зависит только от знака строкового значения.
      if resultValue.starts_with('-') {
        resultType = TokenType::Int;
      } else {
        resultType = TokenType::UInt;
      }
    } else
    if leftTokenDataType == TokenType::Int || rightTokenDataType == TokenType::Int {
      resultType = TokenType::Int;
    }
    //
  }
  // return
  Token::new(resultType, resultValue)
}
/// Зависимость для calculate;
/// Считает значение левой и правой части выражения
fn getValue(tokenData: String, tokenDataType: &TokenType) -> Value 
{
  match tokenDataType
  {
    TokenType::None =>
    {
      Value::None()
    }
    TokenType::Int =>
    { // Токен бесконечен, а Value::Int(i64) нет: всё что больше или меньше - граница i64 (#71)
      match tokenData.parse::<i64>() 
      {
        Ok(value) => Value::Int(value),
        Err(error) => match error.kind() 
        {
          IntErrorKind::PosOverflow => Value::Int(i64::MAX),
          IntErrorKind::NegOverflow => Value::Int(i64::MIN),
          _ => Value::Int(0)
        }
      }
    },
    TokenType::UInt =>
    { // Токен бесконечен, а Value::UInt(u64) нет: всё что больше - граница u64 (#71)
      match tokenData.parse::<u64>() 
      {
        Ok(value) => Value::UInt(value),
        Err(error) => match error.kind() 
        {
          IntErrorKind::PosOverflow => Value::UInt(u64::MAX),
          _ => Value::UInt(0)
        }
      }
    },
    TokenType::Float =>
    {
      tokenData.parse::<f64>()
        .map(Value::Float)
        .unwrap_or_else(|_| Value::Float(0.0))
    },
    TokenType::UFloat =>
    {
      tokenData.parse::<f64>()
        .map(uf64::from)
        .map(Value::UFloat)
        .unwrap_or_else(|_| Value::UFloat(uf64::from(0.0)))
    },
    TokenType::Char =>
    { // todo: добавить поддержку операций с TokenType::formattedChar
      tokenData.parse::<char>()
        .map(Value::Char)
        .unwrap_or_else(|_| Value::Char('\0'))
    },
    TokenType::String =>
    {
      tokenData.parse::<String>()
        .map(Value::String)
        .unwrap_or_else(|_| Value::String(String::new()))
    },
    TokenType::Bool =>
    {
      if tokenData == "True" {
        Value::UInt(1)
      } else {
        Value::UInt(0)
      }
    },
    TokenType::True => Value::UInt(1),
    TokenType::False => Value::UInt(0),
    _ => Value::UInt(0)
  }
}

// =================================================================================================

#[cfg(test)]
mod tests
{
  use super::*;
  // ===============================================================================================

  /// Проверяет тип и значение результата операции;
  fn check(op: TokenType, left: (TokenType, &str), right: (TokenType, &str), expectedType: TokenType, expectedData: &str)
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

  /// Число больше u64 и i64 в операции становится границей, а не 0 (#71);
  #[test]
  fn bigLiteral()
  {
    let big: &str = "99999999999999999999999";
    let negBig: &str = "-99999999999999999999999";
    let uMax: String = u64::MAX.to_string();
    let iMin: String = i64::MIN.to_string();
    // Деление на 0 возвращает левую часть (#30), деление на 1 тоже
    check(TokenType::Divide, (TokenType::UInt, big),    (TokenType::UInt, "0"), TokenType::UInt, &uMax);
    check(TokenType::Divide, (TokenType::UInt, big),    (TokenType::UInt, "1"), TokenType::UInt, &uMax);
    check(TokenType::Divide, (TokenType::Int,  negBig), (TokenType::UInt, "0"), TokenType::Int,  &iMin);
    check(TokenType::Divide, (TokenType::Int,  negBig), (TokenType::UInt, "1"), TokenType::Int,  &iMin);
  }

  /// Переполнение самой операции зажимается в границу, а не паникует (#71);
  #[test]
  fn overflow()
  {
    let uMax: String = u64::MAX.to_string();
    let iMin: String = i64::MIN.to_string();
    let iMax: String = i64::MAX.to_string();
    check(TokenType::Plus,     (TokenType::UInt, &uMax), (TokenType::UInt, "1"), TokenType::UInt, &uMax);
    check(TokenType::Multiply, (TokenType::UInt, &uMax), (TokenType::UInt, "2"), TokenType::UInt, &uMax);
    check(TokenType::Minus,    (TokenType::Int,  &iMin), (TokenType::UInt, "1"), TokenType::Int,  &iMin);
    check(TokenType::Divide,   (TokenType::Int,  &iMin), (TokenType::Int, "-1"), TokenType::Int,  &iMax);
    // u64 больше i64::MAX не ломает знак в смешанных операциях (раньше u64::MAX as i64 = -1)
    check(TokenType::Plus,     (TokenType::Int, "-5"),   (TokenType::UInt, &uMax), TokenType::UInt, &(i64::MAX - 5).to_string());
  }

  // ===============================================================================================

  /// Умножение смешанных типов: `*` в выражениях пока отключён (structure.rs, expressionWith),
  /// поэтому Value::Mul проверяется здесь напрямую, до его включения;
  #[test]
  fn multiplyMixed()
  {
    // Int * UFloat: раньше стояло деление x / y, и -3 * 2.5 давало -1.2
    check(TokenType::Multiply, (TokenType::Int,   "-3"), (TokenType::UFloat, "2.5"),  TokenType::Float,  "-7.5");
    check(TokenType::Multiply, (TokenType::UFloat, "2.5"), (TokenType::Int,   "-3"),  TokenType::Float,  "-7.5");
    // Соседние ветки
    check(TokenType::Multiply, (TokenType::UInt,  "3"),  (TokenType::UFloat, "2.5"),  TokenType::UFloat, "7.5");
    check(TokenType::Multiply, (TokenType::Int,   "-3"), (TokenType::Float,  "-2.5"), TokenType::Float,  "7.5");
    check(TokenType::Multiply, (TokenType::UInt,  "3"),  (TokenType::Float,  "-2.5"), TokenType::Float,  "-7.5");
  }

  /// Умножение коммутативно: a * b == b * a для любой пары числовых типов;
  #[test]
  fn multiplyCommutative()
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
          "'{} * {}' = '{}', но '{} * {}' = '{}'",
          left.1, right.1, abData, right.1, left.1, baData
        );
      }
    }
  }

  // ===============================================================================================
}

// =================================================================================================
