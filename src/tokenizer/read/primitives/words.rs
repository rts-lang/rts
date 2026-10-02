use crate::tokenizer::read::primitives::numbers::isDigit;
use crate::tokenizer::read::primitives::skipWhitespaceBytes;
use crate::tokenizer::types::token::{Token};
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Проверяет что байт является буквой a-z A-Z.
pub const fn isLetter(byte: u8) -> bool
{
  (byte|32) >= b'a' && (byte|32) <= b'z'
}

// =================================================================================================

/// todo desc
pub const keywords: &[(&str, TokenType)] = &[
  ("None", TokenType::None),
  ("Link", TokenType::Link),
  ("Any", TokenType::Any),
  //
  ("Bool", TokenType::Bool),
  ("True", TokenType::True),
  ("False", TokenType::False),
  //
  ("UInt", TokenType::UInt),
  ("Int", TokenType::Int),
  ("UFloat", TokenType::UFloat),
  ("Float", TokenType::Float),
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
pub fn getWord(buffer: &[u8], index: &mut usize, bufferLength: usize) -> Token
{
  let mut savedIndex: usize = *index; // index buffer
  let mut result: String = String::from(buffer[savedIndex] as char);
  savedIndex += 1;
  let mut isLink: bool = false;

  let mut currentByte: u8; // Текущий символ.
  while savedIndex < bufferLength
  {
    currentByte = buffer[savedIndex]; // Значение текущего символа.

    // Пропуск пустот;
    // В языке нет наводящих слов статуса - поэтому не будет `let a`.
    if currentByte == b' ' || currentByte == b'\t'
    {
      savedIndex += 1;
      continue;
    }
    
    if (isDigit(currentByte) || currentByte == b'.') || // Либо число, либо . как ссылка.
      (isLink && (currentByte == b'[' || currentByte == b']')) // В случае ссылки мы можем читать динамические [].
    {
      result.push(currentByte as char);
      savedIndex += 1;
      if currentByte == b'.'
      { // Только если есть . то мы знаем что это ссылка.
        isLink = true;
      }

      // Пропуск пустот;
      // Перенос строки допустим только после `.`: это оператор в конце строки,
      // выражение не завершено (`a.\n  b`, `a.\n  0`). Без оператора строка завершена (`a.0\nb`).
      let mut temp: usize = savedIndex;
      let whitespace: &[u8] = if currentByte == b'.' { b" \t\n" } else { b" \t" };
      skipWhitespaceBytes(buffer, &mut temp, bufferLength, whitespace);
      // После `.` можно продолжить и буквой (`a.b`), и цифрой (`a.0` / `a.\n0`)
      if temp < bufferLength && (isLetter(buffer[temp]) || isDigit(buffer[temp])) {
        savedIndex = temp;
      }
    } else
    {
      if isLetter(currentByte)
      {
        result.push(currentByte as char);
        savedIndex += 1;
      } else { break; }
      //
    }
  }

  *index = savedIndex;

  // next return.
  if isLink {
    Token::new(TokenType::Link, result)
  } else
  { // Ключевое слово.
    for (keyword, tokenType) in keywords.iter() 
    {
      if result == *keyword 
      { // todo true и false – особые случаи ?
        //   Мб просто их сделать True/False как и 
        //   должно быть и они будут отдельный от number в Parser?
        // True/False/true/false — с data; остальные keywords — empty.
        return if matches!(result.as_str(), "True" | "False") {
          Token::new(*tokenType, result)
        } else {
          Token::newEmpty(*tokenType)
        };
        //
      }
    }
    // Обычное слово (идентификатор).
    Token::new(TokenType::Word, result)
    //
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
  fn value() -> ()
  {
    for (keyword, expectedType) in keywords.iter()
    {
      let buffer: &[u8] = keyword.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;
      let token: Token = getWord(buffer, &mut index, bufferLength);

      //
      let tokenType: String = token.getDataType().to_string();
      let expectedType: String = expectedType.to_string();
      assert_eq!(
        tokenType,
        expectedType,
        "For '{}' the expected type was {}, received {}",
        keyword,
        expectedType,
        tokenType
      );

      //
      let tokenData: String = token.getData().toString().unwrap_or_default();
      let expectedData: String = if matches!(*keyword, "True" | "False") {
        keyword.to_string()
      } else {
        String::new()
      };
      assert_eq!(
        tokenData,
        expectedData,
        "Keyword '{}' should have the value '{}', received '{}'",
        keyword,
        expectedData,
        tokenData
      );

      //
      assert_eq!(
        index, bufferLength,
        "The index for '{}' should advance by {} (string length), stopped at {}",
        keyword, bufferLength, index
      );
    }
    //
  }

  /// todo desk
  #[test]
  fn links() -> ()
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
      ("True", TokenType::True, "True"),
      ("False", TokenType::False, "False"),
      ("True", TokenType::True, "True"),
      ("False", TokenType::False, "False"),
      ("None", TokenType::None, ""),
      ("abc123", TokenType::Word, "abc123")
    ] 
    {
      let buffer: &[u8] = input.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;
      let token: Token = getWord(buffer, &mut index, bufferLength);

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
        expectedData,
        "For '{}' value '{}' was expected, got '{}'",
        input,
        expectedData,
        tokenData
      );

      //
      assert_eq!(
        index, bufferLength,
        "For '{}' index should advance by {} (the entire string), stopped at {}",
        input, bufferLength, index
      );
    }
    //
  }

  /// todo desk
  #[test]
  fn index() -> ()
  {
    for (input, expectedWord, expectedType, expectedIndex) in vec![
      ("hello world", "helloworld", TokenType::Word, 11),
      ("myVar=123", "myVar", TokenType::Word, 5),
      ("a.b.c;", "a.b.c", TokenType::Link, 5),
      ("True", "True", TokenType::True, 4),
      ("False", "False", TokenType::False, 5),
      ("None;", "", TokenType::None, 4),
      ("obj.[0].prop,", "obj.[0].prop", TokenType::Link, 12),
      ("a. b(", "a.b", TokenType::Link, 4),        // Пробел после . сливает
      ("a.\n  b", "a.b", TokenType::Link, 6),      // Оператор в конце строки сливает
      ("a.\n  0", "a.0", TokenType::Link, 6),      // Перенос + цифровой индекс поля
      ("a.0\nprintln", "a.0", TokenType::Link, 3), // Без оператора строка завершена
      ("a.b\nprintln", "a.b", TokenType::Link, 3),
      ("abc1\nfoo", "abc1", TokenType::Word, 4),
      ("abc123+", "abc123", TokenType::Word, 6)
    ] 
    {
      let buffer: &[u8] = input.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;
      let token: Token = getWord(buffer, &mut index, bufferLength);

      //
      let tokenType: String = token.getDataType().to_string();
      let expectedType: String = expectedType.to_string();
      assert_eq!(
        tokenType,
        expectedType,
        "For '{}' the expected type was {}, got {}",
        input,
        expectedType,
        tokenType
      );

      //
      let tokenData: String = token.getData().toString().unwrap_or_default();
      assert_eq!(
        tokenData,
        expectedWord,
        "For '{}' the expected word was '{}', got '{}'",
        input,
        expectedWord,
        tokenData
      );

      //
      assert_eq!(
        index, expectedIndex,
        "For '{}' the index should stop at {}, but stopped at {}",
        input, expectedIndex, index
      );
    }
    //
  }

  // ===============================================================================================
}

// =================================================================================================