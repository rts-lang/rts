use std::num::IntErrorKind;
use crate::parser::structure::tokenValue::uf64::*;
use crate::parser::structure::tokenValue::value::Value;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Это вычисление на основе Value от токенов.
// Это отдельные вещи от Structure - потому что это внешний (над ним) инструмент.

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

#[cfg(test)]
mod tests
{
  use crate::parser::structure::tokenValue::calculate::calculate;
  use crate::tokenizer::types::token::Token;
  use crate::tokenizer::types::tokenType::TokenType;
  // ===============================================================================================
  
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

  // ===============================================================================================

  /// Число больше u64 и i64 в операции становится границей, а не 0 (#71).
  #[test]
  fn bigLiteral()
  {
    let big: &str = "99999999999999999999999";
    let negBig: &str = "-99999999999999999999999";
    let uMax: String = u64::MAX.to_string();
    let iMin: String = i64::MIN.to_string();
    // Деление на 0 возвращает левую часть (#30), деление на 1 тоже.
    check(TokenType::Divide, (TokenType::UInt, big),    (TokenType::UInt, "0"), TokenType::UInt, &uMax);
    check(TokenType::Divide, (TokenType::UInt, big),    (TokenType::UInt, "1"), TokenType::UInt, &uMax);
    check(TokenType::Divide, (TokenType::Int,  negBig), (TokenType::UInt, "0"), TokenType::Int,  &iMin);
    check(TokenType::Divide, (TokenType::Int,  negBig), (TokenType::UInt, "1"), TokenType::Int,  &iMin);
  }

  /// Переполнение самой операции зажимается в границу, а не паникует (#71).
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
    // u64 больше i64::MAX не ломает знак в смешанных операциях (раньше u64::MAX as i64 = -1).
    check(TokenType::Plus,     (TokenType::Int, "-5"),   (TokenType::UInt, &uMax), TokenType::UInt, &(i64::MAX - 5).to_string());
  }

  // ===============================================================================================

  /// Сравнения возвращают True/False, а не число (#39, #65).
  #[test]
  fn comparison()
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
  fn comparisonNone()
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

  // ===============================================================================================
  
  /// Умножение смешанных типов: `*` в выражениях пока отключён (structure.rs, expressionWith),
  /// поэтому Value::Mul проверяется здесь напрямую, до его включения.
  #[test]
  fn multiplyMixed()
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
          "'{} * {}' = '{}', but '{} * {}' = '{}'",
          left.1, right.1, abData, right.1, left.1, baData
        );
      }
    }
  }

  /// Результат за границей f64 зажимается, а не становится inf/NaN (#71).
  #[test]
  fn floatOverflow() -> ()
  {
    let max: String = f64::MAX.to_string();
    let min: String = f64::MIN.to_string();
    // Сумма за границей.
    check(TokenType::Plus,  (TokenType::UFloat, "1e308"),  (TokenType::UFloat, "1e308"), TokenType::UFloat, &max);
    check(TokenType::Minus, (TokenType::Float,  "-1e308"), (TokenType::UFloat, "1e308"), TokenType::Float,  &min);
    // Литерал за границей зажимается до операции: MAX - MAX = 0, а не inf - inf.
    check(TokenType::Minus, (TokenType::UFloat, "1e309"),  (TokenType::UFloat, "1e309"),  TokenType::UFloat, "0");
    check(TokenType::Minus, (TokenType::UFloat, "inf"),    (TokenType::UFloat, "inf"),    TokenType::UFloat, "0");
    check(TokenType::Minus, (TokenType::Float,  "-1e309"), (TokenType::Float,  "-1e309"), TokenType::Float,  "0");
    // Отрицательный UFloat не теряется: из него получается Float.
    check(TokenType::Minus, (TokenType::UFloat, "1.0"),    (TokenType::UFloat, "2.0"),    TokenType::Float,  "-1");
  }

  // ===============================================================================================
}

// =================================================================================================
