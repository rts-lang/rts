use crate::tokenizer::read::primitives::numbers::isDigit;
use crate::tokenizer::types::token::{Token};
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Проверяет что байт является буквой a-z A-Z
pub fn isLetter(c: &u8) -> bool
{
  (c|32)>=b'a'&&(c|32)<=b'z'
}

// =================================================================================================

pub const keywords: &[(&str, TokenType)] = &[
  ("None", TokenType::None),
  ("Link", TokenType::Link),
  ("Any", TokenType::Any),
  //
  ("Bool", TokenType::Bool), // todo issue #65
  ("true", TokenType::Bool), // todo issue #65
  ("false", TokenType::Bool), // todo issue #65
  //
  ("UInt", TokenType::UInt),
  ("Int", TokenType::Int),
  ("UFloat", TokenType::UFloat),
  ("Float", TokenType::Float),
  //("Rational", TokenType::Rational), // todo Rational пока что нет как типа
  //
  ("Char", TokenType::Char),
  ("String", TokenType::String),
  ("RawString", TokenType::RawString),
  ("FormattedChar", TokenType::FormattedChar),
  ("FormattedString", TokenType::FormattedString),
  ("FormattedRawString", TokenType::FormattedRawString)
];

/// Проверяет buffer по index и так находит возможные слова;
/// Из них также выделяет сразу определяемые зарезервированные
pub fn getWord(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> Token
{
  let mut savedIndex: usize = *index; // index buffer
  let mut result: String = String::from(buffer[savedIndex] as char);
  savedIndex += 1;
  let mut isLink: bool = false;

  let mut byte1: u8; // Текущий символ
  while savedIndex < *bufferLength
  {
    byte1 = buffer[savedIndex]; // Значение текущего символа
    
    // todo: use match case
    if (isDigit(&byte1) || byte1 == b'.') || // Либо число, либо . как ссылка
      (isLink && (byte1 == b'[' || byte1 == b']')) // В случае ссылки мы можем читать динамические []
    {
      result.push(byte1 as char);
      savedIndex += 1;
      match byte1 == b'.'
      { false => {} true =>
      { // Только если есть . то мы знаем что это ссылка
        isLink = true;
      }}
    } else
    {
      match isLetter(&byte1)
      { false => { break; } true =>
      {
        result.push(byte1 as char);
        savedIndex += 1;
      }}
      //
    }
  }

  *index = savedIndex;

  // next return
  match isLink
  {
    true => Token::new(TokenType::Link, result),
    false =>
    {
      // Ключевое слово 
      for (keyword, tokenType) in keywords.iter() 
      {
        if result == *keyword {
          // todo true и false – особые случаи ?
          return if result == "true" || result == "false" {
            Token::new(*tokenType, result)
          } else {
            Token::newEmpty(*tokenType)
          };
          //
        }
      }
      // Обычное слово (идентификатор)
      Token::new(TokenType::Word, result)
      //
    }
  }
  //
}

// =================================================================================================

#[cfg(test)]
mod tests
{
  use crate::tokenizer::read::primitives::words::{getWord, keywords};
  use crate::tokenizer::types::token::{Token};
  use crate::tokenizer::types::tokenType::TokenType;
  // ===============================================================================================

  /// todo desk
  #[test]
  fn value() 
  {
    for (keyword, expectedType) in keywords.iter() 
    {
      let buffer: &[u8] = keyword.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;
      let token: Token = getWord(buffer, &mut index, &bufferLength);

      //
      let tokenType: String = token.getDataType().to_string();
      let expectedType: String = expectedType.to_string();
      assert_eq!(
        tokenType,
        expectedType,
        "Для '{}' ожидался тип {}, получен {}",
        keyword,
        expectedType,
        tokenType
      );

      //
      let tokenData: String = token.getData().toString().unwrap_or_default();
      let expectedData: String = if *keyword == "true" || *keyword == "false" {
        keyword.to_string()
      } else {
        String::new()
      };
      assert_eq!(
        tokenData,
        expectedData,
        "Ключевое слово '{}' должно иметь значение '{}', получено '{}'",
        keyword,
        expectedData,
        tokenData
      );

      //
      assert_eq!(
        index, bufferLength,
        "Индекс для '{}' должен продвинуться на {} (длина строки), остановился на {}",
        keyword, bufferLength, index
      );
    }
    //
  }

  /// todo desk
  #[test]
  fn links() 
  {
    for (input, expectedType, expectedData) in vec![
      ("hello", TokenType::Word, "hello"),
      ("world123", TokenType::Word, "world123"),
      ("myVar", TokenType::Word, "myVar"),
      ("a.", TokenType::Link, "a."),
      ("var.name", TokenType::Link, "var.name"),
      ("obj.prop[0]", TokenType::Link, "obj.prop[0]"),
      ("data.list[1].value", TokenType::Link, "data.list[1].value"),
      ("arr.[42].field", TokenType::Link, "arr.[42].field"),
      ("true", TokenType::Bool, "true"),
      ("false", TokenType::Bool, "false"),
      ("None", TokenType::None, ""),
      ("abc123", TokenType::Word, "abc123"),
    ] {
      let buffer: &[u8] = input.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;
      let token: Token = getWord(buffer, &mut index, &bufferLength);

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

      //
      let tokenData: String = token.getData().toString().unwrap_or_default();
      assert_eq!(
        tokenData,
        expectedData,
        "Для '{}' ожидалось значение '{}', получено '{}'",
        input,
        expectedData,
        tokenData
      );

      //
      assert_eq!(
        index, bufferLength,
        "Для '{}' индекс должен продвинуться на {} (вся строка), остановился на {}",
        input, bufferLength, index
      );
    }
    //
  }

  /// todo desk
  #[test]
  fn index() 
  {
    for (input, expectedWord, expectedType, expectedIndex) in vec![
      ("hello world", "hello", TokenType::Word, 5),
      ("myVar=123", "myVar", TokenType::Word, 5),
      ("a.b.c;", "a.b.c", TokenType::Link, 5),
      ("true false", "true", TokenType::Bool, 4),
      ("None;", "", TokenType::None, 4),
      ("obj.[0].prop,", "obj.[0].prop", TokenType::Link, 12),
      ("abc123+", "abc123", TokenType::Word, 6),
    ] {
      let buffer: &[u8] = input.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;
      let token: Token = getWord(buffer, &mut index, &bufferLength);
      
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

      //
      let tokenData: String = token.getData().toString().unwrap_or_default();
      assert_eq!(
        tokenData,
        expectedWord,
        "Для '{}' ожидалось слово '{}', получено '{}'",
        input,
        expectedWord,
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