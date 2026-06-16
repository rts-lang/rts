use crate::parser::structure::value::uf64::*;
use crate::parser::structure::value::value::Value;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
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
  let resultValue: String = match *op 
  {
    TokenType::Plus     => (leftValue + rightValue).to_string(),
    TokenType::Minus    => (leftValue - rightValue).to_string(),
    TokenType::Multiply => (leftValue * rightValue).to_string(),
    TokenType::Divide   => (leftValue / rightValue).to_string(),
    TokenType::Inclusion => 
    { 
      resultType = TokenType::Bool;
      match leftValue.toBool() || rightValue.toBool() 
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    TokenType::Joint => 
    { 
      resultType = TokenType::Bool;
      match leftValue.toBool() && rightValue.toBool() 
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    TokenType::Equals => 
    { 
      resultType = TokenType::Bool;
      match leftValue == rightValue 
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    TokenType::NotEquals => 
    { 
      resultType = TokenType::Bool;
      match leftValue != rightValue
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    TokenType::GreaterThan => 
    { 
      resultType = TokenType::Bool;
      match leftValue > rightValue 
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    TokenType::LessThan => 
    { 
      resultType = TokenType::Bool;
      match leftValue < rightValue 
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    TokenType::GreaterThanOrEquals => 
    { 
      resultType = TokenType::Bool;
      match leftValue >= rightValue 
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    TokenType::LessThanOrEquals => 
    { 
      resultType = TokenType::Bool;
      match leftValue <= rightValue 
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    _ => "0".to_string(),
  };
  // После того как значение было получено,
  // Смотрим какой точно тип выдать новому токену
  // todo: if -> match
  match resultType != TokenType::Bool 
  {
    false => {}
    true => 
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
      if leftTokenDataType == TokenType::Float || rightTokenDataType == TokenType::Float
      {
        resultType = TokenType::Float;
      } else
      if leftTokenDataType == TokenType::UFloat || rightTokenDataType == TokenType::UFloat
      {
        resultType = TokenType::UFloat;
      } else
      if leftTokenDataType == TokenType::Int || rightTokenDataType == TokenType::Int
      {
        resultType = TokenType::Int;
      }
    }
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
    {
      tokenData.parse::<i64>()
        .map(Value::Int)
        .unwrap_or(Value::Int(0))
    },
    TokenType::UInt =>
    {
      tokenData.parse::<u64>()
        .map(Value::UInt)
        .unwrap_or(Value::UInt(0))
    },
    TokenType::Float =>
    {
      tokenData.parse::<f64>()
        .map(Value::Float)
        .unwrap_or(Value::Float(0.0))
    },
    TokenType::UFloat =>
    {
      tokenData.parse::<f64>()
        .map(uf64::from)
        .map(Value::UFloat)
        .unwrap_or(Value::UFloat(uf64::from(0.0)))
    },
    TokenType::Char =>
    { // todo: добавить поддержку операций с TokenType::formattedChar
      tokenData.parse::<char>()
        .map(|x| Value::Char(x))
        .unwrap_or(Value::Char('\0'))
    },
    TokenType::String =>
    {
      tokenData.parse::<String>()
        .map(|x| Value::String(x))
        .unwrap_or(Value::String("".to_string()))
    },
    TokenType::Bool =>
    {
      match tokenData == "true"
      {
        true  => Value::UInt(1),
        false => Value::UInt(0)
      }
    },
    _ => Value::UInt(0)
  }
}

// =================================================================================================