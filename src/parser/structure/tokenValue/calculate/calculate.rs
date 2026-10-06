use std::num::IntErrorKind;
use crate::parser::structure::tokenValue::uf64::*;
use crate::parser::structure::tokenValue::value::Value;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Это вычисление на основе Value от токенов.
//
// Это сама реализация операций математики и логики.
//
// Схема такая: calculate -> tokenizer+parser expression -> StructureType -> FFI.

// =================================================================================================

/// Вычисляет по операции значение и тип нового токена из двух.
pub fn calculate(op: &TokenType, leftToken: &Token, rightToken: &Token) -> Token 
{
  // Получаем значение левой части выражения.
  let leftTokenDataType: TokenType = *leftToken.getDataType();
  let leftValue: Value = getValue(
    leftToken.getData().toString().unwrap_or_default(),
    &leftTokenDataType
  );
  // Получаем значение правой части выражения.
  let rightTokenDataType: TokenType = *rightToken.getDataType();
  let rightValue: Value = getValue(
    rightToken.getData().toString().unwrap_or_default(),
    &rightTokenDataType
  );

  // -----------------------------------------------------------------------------------------------
  // Логические операции и сравнения: результат всегда токен True или False (#39, #65).
  //
  // todo #39: `&` и `|` пока двоичные (None приводится к False). Таблицы троичной логики
  //  с None добавятся вместе с операторами, сейчас они не подключены в expressionWith.
  let truth: Option<bool> = match *op
  {
    TokenType::Inclusion           => Some(leftValue.toBool() || rightValue.toBool()),
    TokenType::Joint               => Some(leftValue.toBool() && rightValue.toBool()),
    TokenType::Equals              => Some(leftValue == rightValue),
    TokenType::NotEquals           => Some(leftValue != rightValue),
    TokenType::GreaterThan         => Some(leftValue > rightValue),
    TokenType::LessThan            => Some(leftValue < rightValue),
    TokenType::GreaterThanOrEquals => Some(leftValue >= rightValue),
    TokenType::LessThanOrEquals    => Some(leftValue <= rightValue),
    _ => None
  };
  if let Some(truth) = truth
  {
    return if truth {
      Token::new(TokenType::True, String::from("True")) // todo По идее не должно быть String, а только newEmpty(...)
    } else {
      Token::new(TokenType::False, String::from("False")) // todo По идее не должно быть String, а только newEmpty(...)
    };
  }

  // -----------------------------------------------------------------------------------------------
  // Получаем значение выражения, а также предварительный тип.
  let mut resultType: TokenType = TokenType::UInt;
  let resultValue: String = match *op 
  {
    TokenType::Plus     => normalizeValue(leftValue + rightValue).to_string(),
    TokenType::Minus    => normalizeValue(leftValue - rightValue).to_string(),
    TokenType::Multiply => normalizeValue(leftValue * rightValue).to_string(),
    TokenType::Divide   => normalizeValue(leftValue / rightValue).to_string(),
    _ => "0".to_string(),
  };
  // После того как значение было получено,
  // смотрим какой точно тип выдать новому токену.
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
    //
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
    //
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
  // return
  Token::new(resultType, resultValue)
  // -----------------------------------------------------------------------------------------------
}

// =================================================================================================

/// Нормализация (#71) Token по его TokenType: токен бесконечен, а математика Rust нет.
///
/// 4 класса токенов для чисел, у каждого свои min и max значения:
///   UInt    0 ... usize::MAX;
///   Int     isize::MIN ... -1.
///   UFloat  0 ... f64::MAX;
///   Float   f64::MIN ... -0.
///
/// todo У floats inf становится границей в виде `{:e}`, как в структуре; NaN - 0.
pub fn normalizeToken(token: &mut Token) -> ()
{
  let data: String = token.getData().toString().unwrap_or_default();
  let normalized: Option<String> = match *token.getDataType()
  {
    TokenType::UInt => match data.parse::<u64>()
    {
      Ok(value) if value > usize::MAX as u64 => Some(usize::MAX.to_string()),
      Err(error) if *error.kind() == IntErrorKind::PosOverflow => Some(usize::MAX.to_string()),
      _ => None
    },
    TokenType::Int => match data.parse::<i64>()
    {
      Ok(value) if value > isize::MAX as i64 => Some(isize::MAX.to_string()),
      Ok(value) if value < isize::MIN as i64 => Some(isize::MIN.to_string()),
      Err(error) if *error.kind() == IntErrorKind::PosOverflow => Some(isize::MAX.to_string()),
      Err(error) if *error.kind() == IntErrorKind::NegOverflow => Some(isize::MIN.to_string()),
      _ => None
    },
    TokenType::UFloat | TokenType::Float => match data.parse::<f64>()
    {
      Ok(value) if value.is_nan() => Some(String::from("0")),
      Ok(value) if value.is_infinite() => Some(format!("{:e}", normalizeFloat(value))),
      _ => None
    },
    _ => None
  };
  if let Some(normalized) = normalized { 
    token.setData(normalized);
  }
}

/// Зависимость для calculate;
/// Значение за границей f64 зажимается в границу, как в структуре: inf - f64::MAX/MIN, NaN - 0.
fn normalizeFloat(value: f64) -> f64
{
  if value.is_nan() { 0.0 } else { value.clamp(f64::MIN, f64::MAX) }
}

/// Зависимость для calculate;
/// Зажимает результат операции: промежуточное значение не должно быть inf или NaN (#71).
///
/// Иначе `(1e308 + 1e308) - (1e308 + 1e308)` считалось бы как `inf - inf = NaN`,
/// а не как `f64::MAX - f64::MAX = 0`, потому что структура зажимает только итог.
fn normalizeValue(value: Value) -> Value
{
  match value
  {
    // UInt - usize, Int - isize (Value хранит u64/i64, на 32 битах потолок ниже).
    Value::UInt(x) => Value::UInt(x.min(usize::MAX as u64)),
    Value::Int(x) => Value::Int(x.clamp(isize::MIN as i64, isize::MAX as i64)),
    Value::Float(x) => Value::Float(normalizeFloat(x)),
    // Отрицательный UFloat сохраняется: из него calculate делает Float (`1.0 - 2.0`).
    Value::UFloat(x) if !f64::from(x).is_finite() =>
      Value::UFloat(uf64::from(normalizeFloat(f64::from(x)))),
    other => other
  }
}

// =================================================================================================

/// Зависимость для calculate;
/// 
/// Считает значение левой и правой части выражения
fn getValue(tokenData: String, tokenDataType: &TokenType) -> Value 
{
  match tokenDataType
  {
    TokenType::None => Value::None(),
    TokenType::Int =>
    { // Токен бесконечен, а Int нет: всё что больше или меньше - граница isize (#71).
      match tokenData.parse::<i64>() 
      {
        Ok(value) => Value::Int(value.clamp(isize::MIN as i64, isize::MAX as i64)),
        Err(error) => match error.kind() 
        {
          IntErrorKind::PosOverflow => Value::Int(isize::MAX as i64),
          IntErrorKind::NegOverflow => Value::Int(isize::MIN as i64),
          _ => Value::Int(0)
        }
      }
    },
    TokenType::UInt =>
    { // Токен бесконечен, а UInt нет: всё что больше - граница usize (#71).
      match tokenData.parse::<u64>() 
      {
        Ok(value) => Value::UInt(value.min(usize::MAX as u64)),
        Err(error) => match error.kind() 
        {
          IntErrorKind::PosOverflow => Value::UInt(usize::MAX as u64),
          _ => Value::UInt(0)
        }
      }
    },
    TokenType::Float =>
    { // Токен бесконечен, а Value::Float(f64) нет: inf - граница f64 (#71).
      tokenData.parse::<f64>()
        .map(|value| Value::Float(normalizeFloat(value)))
        .unwrap_or_else(|_| Value::Float(0.0))
    },
    TokenType::UFloat =>
    { // Токен бесконечен, а Value::UFloat(uf64) нет: inf - граница f64 (#71).
      tokenData.parse::<f64>()
        .map(|value| Value::UFloat(uf64::from(normalizeFloat(value))))
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
    TokenType::True => Value::UInt(1),
    TokenType::False => Value::UInt(0),
    _ => Value::UInt(0)
  }
}

// =================================================================================================
