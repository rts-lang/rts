use crate::tokenizer::read::primitives::skipWhitespaceBytes;
use crate::tokenizer::types::token::{Token};
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Проверяет что байт является одиночным знаком
pub fn isSingleChar(byte: &u8) -> bool
{
  matches!(*byte, 
    b'+' | b'-' | b'*' | b'/' | b'=' | b'%' | b'^' |
    b'>' | b'<' | b'?' | b'!' | b'&' | b'|' | 
    b'(' | b')' | b'{' | b'}' | b'[' | b']' | 
    b':' | b',' | b'.' | b'~'
  )
}

// =================================================================================================

pub const operators: &[(&str, TokenType)] = &[
  // Одиночные математические
  ("+", TokenType::Plus),
  ("-", TokenType::Minus),
  ("*", TokenType::Multiply),
  ("/", TokenType::Divide),
  ("=", TokenType::Equals),
  ("%", TokenType::Modulo),
  ("^", TokenType::Exponent),

  // Двойные математические
  ("++", TokenType::UnaryPlus),
  ("+=", TokenType::PlusEquals),
  ("--", TokenType::UnaryMinus),
  ("-=", TokenType::MinusEquals),
  ("**", TokenType::UnaryMultiply),
  ("*=", TokenType::MultiplyEquals),
  ("//", TokenType::UnaryDivide),
  ("/=", TokenType::DivideEquals),
  ("%%", TokenType::UnaryModulo),
  ("%=", TokenType::ModuloEquals),
  ("^^", TokenType::UnaryExponent),
  ("^=", TokenType::ExponentEquals),

  // Логические
  (">", TokenType::GreaterThan),
  ("<", TokenType::LessThan),
  ("!", TokenType::Not),
  (">=", TokenType::GreaterThanOrEquals),
  ("<=", TokenType::LessThanOrEquals),
  ("!=", TokenType::NotEquals),
  ("&", TokenType::Joint),
  ("|", TokenType::Inclusion),
  ("?", TokenType::Question),

  // Скобки
  ("(", TokenType::CircleBracketBegin),
  (")", TokenType::CircleBracketEnd),
  ("[", TokenType::SquareBracketBegin),
  ("]", TokenType::SquareBracketEnd),
  ("{", TokenType::FigureBracketBegin),
  ("}", TokenType::FigureBracketEnd),

  // Прочее
  (":", TokenType::Colon),
  ("->", TokenType::Pointer),
  ("~", TokenType::Tilde),
  ("~~", TokenType::DoubleTilde),
  (",", TokenType::Comma),
  (".", TokenType::Dot),
];

/// Проверяет buffer по index и так находит возможные двойные и одиночные операторы
pub fn getOperator(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> Token
{
  // Ищем паттерн
  let mut best: Option<(usize, TokenType, usize)> = None;
  for (pattern, tokenType) in operators.iter()
  {
    let byte1: u8 = buffer[*index];
    let mut byte2: u8 = 0;
    let mut endIndex: usize = *index + 1; // для одиночного знака

    let patternLength: usize = pattern.len();
    
    // Пропуск пустот;
    // Ожидает только знаки.
    if patternLength == 2
    {
      let mut scanIndex: usize = *index + 1;
      skipWhitespaceBytes(buffer, &mut scanIndex, *bufferLength, b" \t\n");
      if scanIndex < *bufferLength && isSingleChar(&buffer[scanIndex]) {
        byte2 = buffer[scanIndex];
        endIndex = scanIndex + 1;
      }
    }

    //
    let bytes: &[u8] = if patternLength == 2 {
      &[byte1, byte2]
    } else {
      &[byte1]
    };

    //
    if bytes == pattern.as_bytes()
    {
      match best
      {
        Some((bestLength, _, _)) if patternLength <= bestLength => {} // keep the longer one
        _ => best = Some((patternLength, *tokenType, endIndex)),
      }
    }
  }

  // result
  if let Some((_length, tokenType, endIndex)) = best {
    *index = endIndex;
    return Token::newEmpty(tokenType);
  }
  Token::newEmpty(TokenType::None)
}

// =================================================================================================

#[cfg(test)]
mod tests 
{
  use crate::tokenizer::read::primitives::operators::{getOperator, operators};
  use crate::tokenizer::types::token::{Token};
  use crate::tokenizer::types::tokenType::TokenType;
  // ===============================================================================================

  /// todo desk
  #[test]
  fn value() 
  {
    for (pat, expectedType) in operators.iter() 
    {
      let buffer: &[u8] = pat.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;
      let token: Token = getOperator(buffer, &mut index, &bufferLength);

      //
      let tokenType: String = token.getDataType().to_string();
      let expectedType: String = expectedType.to_string();
      assert_eq!(
        tokenType,
        expectedType,
        "Для '{}' ожидался тип {}, получен {}",
        pat,
        expectedType,
        tokenType
      );

      // Для операторов значение всегда пустое
      let tokenData: String = token.getData().toString().unwrap_or_default();
      assert_eq!(
        tokenData,
        "",
        "Оператор '{}' должен иметь пустое значение, получено '{}'",
        pat,
        tokenData
      );

      //
      assert_eq!(
        index, bufferLength,
        "Индекс для '{}' должен продвинуться на {} (длина строки), остановился на {}",
        pat, bufferLength, index
      );
    }
    //
  }

  /// todo desk
  #[test]
  fn index() 
  {
    for (input, expectedType, expectedIndex) in [
      ("+ 1", TokenType::Plus, 1),
      /* todo Может плохо работать с #85, нужен контроль
      ("++x", TokenType::UnaryPlus, 2),
      ("-=abc", TokenType::MinusEquals, 2),
      ("**123", TokenType::UnaryMultiply, 2),
      */
      ("!=   ", TokenType::NotEquals, 2),
      ("->7", TokenType::Pointer, 2),
      ("~~ ", TokenType::DoubleTilde, 2),
      ("...", TokenType::Dot, 1),
    ] {
      let buffer: &[u8] = input.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;
      let token: Token = getOperator(buffer, &mut index, &bufferLength);

      //
      let tokenType: String = token.getDataType().to_string();
      let expectedType: String = expectedType.to_string();
      assert_eq!(
        tokenType,
        expectedType,
        "Для '{}' ожидался тип {}, получен {}",
        input,
        expectedType,
        tokenType
      );

      // Для операторов значение всегда пустое
      let tokenData: String = token.getData().toString().unwrap_or_default();
      assert_eq!(
        tokenData,
        "",
        "Оператор '{}' должен иметь пустое значение, получено '{}'",
        input,
        tokenData
      );

      //
      assert_eq!(
        index, expectedIndex,
        "Для '{}' индекс должен остановиться на {}, а остановился на {}",
        input, expectedIndex, index
      );
    }
    //
  }
  
  // ===============================================================================================
}

// =================================================================================================