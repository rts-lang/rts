use crate::tokenizer::read::primitives::skipWhitespaceBytes;
use crate::tokenizer::read::primitives::words::isLetter;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Проверяет что байт является цифрой.
pub const fn isDigit(byte: u8) -> bool
{
  byte >= b'0' && byte <= b'9'
}

// =================================================================================================

/// Точка может начинать float-примитив (`.1`, `.`), а не только быть
/// оператором member-access (`obj.field`).
///
/// Числовая точка: после опциональных пробелов идёт цифра,
/// либо это «голая» точка (конец буфера / не буква и не `_`).
pub fn isFloatDotStart(buffer: &[u8], index: usize, bufferLength: usize) -> bool
{
  if index >= bufferLength || buffer[index] != b'.' {
    return false;
  }

  let mut temp: usize = index + 1;
  skipWhitespaceBytes(buffer, &mut temp, bufferLength, b" \t");

  // `.1`, `. 14`.
  if temp < bufferLength && isDigit(buffer[temp]) {
    return true;
  }

  // голая `.` = 0.0 — не member access (после точки не идентификатор).
  if temp >= bufferLength {
    return true;
  }

  let next: u8 = buffer[temp];
  // буква / `_` → member access (`a.b`), иначе числовой ноль.
  !(next == b'_' || isLetter(next))
}

// =================================================================================================

/// Проверяет buffer по index и так находит возможные примитивные числовые типы данных;
/// `UInt, Int, UFloat, Float`
///
/// Здесь нет Complex и Rational так как это платформо-зависимые вещи и должны быть
/// реализованы отдельно. todo Однако можно рассматривать синтаксис в будущем.
pub fn getNumber(buffer: &[u8], index: &mut usize, bufferLength: usize) -> Option<Token>
{
  let mut savedIndex: usize = *index; // index buffer
  let mut result: String = String::new();

  let mut hasDot: bool = false; // dot check
  let mut negative: bool = false; // negative check
  let mut hasExponential: bool = false; // e, e+, e-
  let mut hasDigit: bool = false;

  let mut currentByte: u8; // Текущий символ
  while savedIndex < bufferLength
  {
    currentByte = buffer[savedIndex]; // Значение текущего символа.
    
    if currentByte == b' ' || currentByte == b'\t'
    { // Внутренние пробелы (`12 34 . 20`) — ок; 
      // хвостовые (`123 `) не трогаем — их забирает токенайзер между токенами.
      let mut temp: usize = savedIndex + 1;
      skipWhitespaceBytes(buffer, &mut temp, bufferLength, b" \t");
      if temp < bufferLength
      {
        let next: u8 = buffer[temp];
        let canContinue: bool = isDigit(next)
          || (next == b'.' && !hasDot)
          || (!hasExponential && (next == b'e' || next == b'E'));
        if canContinue
        {
          savedIndex = temp;
          continue;
        }
      }
      break;
    }

    if !negative && buffer[*index] == b'-'
    { // Int/Float
      // Логика тут простая - токенайзер должен вложить минус в число, потому что он рядом.
      // Любое алгебраическое выражение будь то `10-20` - всегда `-20` общая сущность.
      // Поэтому и раскрывается потом как: `10+(-20)`. Поэтому если минус перед числом -
      // он обязательно должен быть втянут в него. Поэтому для чисел он всегда унарный;
      // Для выражений: `a-b` он будет уже бинарный т.к. там логика парсера идет.
      // Т.е. - бинарный минус это когда ты не можешь применить его без парсера.
      result.push(currentByte as char);
      negative = true;
      savedIndex += 1;

      // Пропуск пустот.
      let mut temp: usize = savedIndex;
      skipWhitespaceBytes(buffer, &mut temp, bufferLength, b" \t\n");
      
      //
      if temp < bufferLength && (isDigit(buffer[temp]) || isFloatDotStart(buffer, temp, bufferLength)) {
        savedIndex = temp;
      } else {
        return None; // Это было не число.
      }
    } else
    if isDigit(currentByte)
    { // UInt
      result.push(currentByte as char);
      hasDigit = true;
      savedIndex += 1;
    } else
    if currentByte == b'.' && !hasDot
    { // UFloat

      // Нужно, чтобы читать: `12 34 . 20` и `0.\n10` → `0.10`.
      let mut temp: usize = savedIndex + 1;
      skipWhitespaceBytes(buffer, &mut temp, bufferLength, b" \t\n");
      let hasDigitAfterDot: bool = temp < bufferLength && isDigit(buffer[temp]);

      //
      if hasDigit || hasDigitAfterDot || isFloatDotStart(buffer, savedIndex, bufferLength)
      {
        hasDot = true;
        result.push(currentByte as char);
        savedIndex += 1;
        if hasDigitAfterDot {
          savedIndex = temp;
        }
      } else
      {
        break;
      }
    } else
    if !hasExponential && (currentByte == b'e' || currentByte == b'E')
    { // Это должно быть float, без повторений E.
      if !hasDigit && !hasDot {
        break;
      }

      //
      hasExponential = true;
      result.push(currentByte as char);
      savedIndex += 1;
      hasDot = true; // Если будет integer - то станет от этого float.

      // Нужно, чтобы читать: `12 34 e + 2`.
      let mut temp: usize = savedIndex;
      skipWhitespaceBytes(buffer, &mut temp, bufferLength, b" \t");
      if temp < bufferLength && (buffer[temp] == b'+' || buffer[temp] == b'-') {
        result.push(buffer[temp] as char);
        savedIndex = temp + 1;
      }
    } else { break; }
  }

  if result.is_empty() || result == "-"
  {
    return None;
  }

  // `.` / `-.` — f64::parse не принимает одиночную точку.
  if hasDot && !hasDigit
  {
    result = if negative { String::from("-0.0") } else { String::from("0.0") };
  }

  // Каноническая запись: `.0` / `0.` / `0.0` → `0`, `.5` → `0.5`, `3.` → `3`.
  // print() печатает token data как есть, а не Display от Value.
  if hasDot
  {
    if let Ok(v) = result.parse::<f64>()
    {
      result = v.to_string();
    }
  }

  *index = savedIndex;

  // next return.
  Some(
    match (hasDot, negative)
    { // dot, negative
      (true, true)  => Token::new( TokenType::Float,  result ),
      (true, false) => Token::new( TokenType::UFloat, result ),
      (false, true) => Token::new( TokenType::Int,    result ),
      _             => Token::new( TokenType::UInt,   result )
    }
  )
  //
}

// =================================================================================================

#[cfg(test)]
mod tests
{
  use crate::tokenizer::read::primitives::numbers::{getNumber, isFloatDotStart};
  use crate::tokenizer::types::token::Token;
  use crate::tokenizer::types::tokenType::TokenType;
  // ===============================================================================================
  
  /// todo desc
  #[test]
  fn value() -> ()
  {
    for (input, expectedType, expectedValue) in [
      // UInt
      ("0", TokenType::UInt, "0"),
      ("1", TokenType::UInt, "1"),
      ("1234567890", TokenType::UInt, "1234567890"),
      
      // Int
      ("-0", TokenType::Int, "-0"),
      ("-1", TokenType::Int, "-1"),
      ("-987654321", TokenType::Int, "-987654321"),
      
      // UFloat
      ("3.14", TokenType::UFloat, "3.14"),
      ("0.0", TokenType::UFloat, "0"),
      ("3.", TokenType::UFloat, "3"),
      (".14", TokenType::UFloat, "0.14"),
      (".0", TokenType::UFloat, "0"),
      (".", TokenType::UFloat, "0"),
      
      // Float
      ("-14.3", TokenType::Float, "-14.3"),
      ("-2.5", TokenType::Float, "-2.5"),
      ("-.5", TokenType::Float, "-0.5"),
      ("-.", TokenType::Float, "-0")
    ] 
    {
      let buffer: &[u8] = input.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;
      let token: Token = getNumber(buffer, &mut index, bufferLength)
        .unwrap_or_else(|| panic!("getNumber returned None for '{}'", input));

      let tokenType: String = token.getDataType().to_string();
      let expectedTypeStr: String = expectedType.to_string();
      assert_eq!(tokenType, expectedTypeStr,
                 "For '{}' expected type {}, got {}", input, expectedTypeStr, tokenType);
      let tokenData: String = token.getData().toString().unwrap_or_default();
      assert_eq!(tokenData, expectedValue,
                 "For '{}' expected '{}', got '{}'", input, expectedValue, tokenData);
      assert_eq!(index, bufferLength, "Index did not reach the end for '{}'", input);
    }
  }

  /// todo desc
  #[test]
  fn floatDotStart() -> ()
  {
    assert!(isFloatDotStart(b".", 0, 1));
    assert!(isFloatDotStart(b".0", 0, 2));
    assert!(isFloatDotStart(b".14", 0, 3));
    assert!(isFloatDotStart(b".+", 0, 2));
    assert!(!isFloatDotStart(b".field", 0, 6));
    assert!(!isFloatDotStart(b"._x", 0, 3));
    assert!(!isFloatDotStart(b"a", 0, 1));
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn index() -> ()
  {
    for (input, expectedType, expectedValue, expectedIndex) in [
      ("123 ", TokenType::UInt, "123", 3),
      ("-42x", TokenType::Int, "-42", 3),
      ("3.14+", TokenType::UFloat, "3.14", 4),
      ("-5.5abc", TokenType::Float, "-5.5", 4),
      ("100500\n", TokenType::UInt, "100500", 6)
    ] 
    {
      let buffer: &[u8] = input.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;
      let token: Token = getNumber(buffer, &mut index, bufferLength)
        .unwrap_or_else(|| panic!("getNumber returned None for '{}'", input));

      //
      let tokenType: String = token.getDataType().to_string();
      let expectedType: String = expectedType.to_string();
      assert_eq!(
        tokenType,
        expectedType,
        "For '{}' type {} was expected, got {}",
        input,
        expectedType,
        tokenType
      );

      //
      let tokenData: String = token.getData().toString().unwrap_or_default();
      assert_eq!(
        tokenData,
        expectedValue,
        "For '{}' value '{}' was expected, got '{}'",
        input,
        expectedValue,
        tokenData
      );

      //
      assert_eq!(
        index, expectedIndex,
        "For '{}' index should stop at {}, but stopped at {}",
        input, expectedIndex, index
      );
    }
    //
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn exponential() -> ()
  {
    for (input, expectedType, expectedValue, expectedIndex) in [
      ("1e3", TokenType::UFloat, "1000", 3),
      ("1.5e-2", TokenType::UFloat, "0.015", 6),
      ("-3.14e+10", TokenType::Float, "-31400000000", 9),
      ("0e0", TokenType::UFloat, "0", 3),
      ("-1E-5", TokenType::Float, "-0.00001", 5),
      ("2e+5", TokenType::UFloat, "200000", 4),
      ("10e-1", TokenType::UFloat, "1", 5)
    ] 
    {
      let buffer: &[u8] = input.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;

      let token: Token = getNumber(buffer, &mut index, bufferLength)
        .unwrap_or_else(|| panic!("getNumber returned None for '{}'", input));

      //
      let tokenType: String = token.getDataType().to_string();
      let expectedType: String = expectedType.to_string();
      assert_eq!(
        tokenType,
        expectedType,
        "For '{}' the type {} was expected, {} was received",
        input,
        expectedType,
        tokenType
      );

      //
      let tokenData: String = token.getData().toString().unwrap_or_default();
      assert_eq!(
        tokenData,
        expectedValue,
        "For '{}' the value '{}' was expected, '{}' was received",
        input,
        expectedValue,
        tokenData
      );

      //
      assert_eq!(
        index,
        expectedIndex,
        "For '{}' the index should stop at {}, but stopped at {}",
        input,
        expectedIndex,
        index
      );
    }
  }

  // ===============================================================================================
}

// =================================================================================================
