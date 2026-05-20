use crate::tokenizer::types::token::{Token};
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Проверяет buffer по index и так находит возможные
/// Char, String, RawString
pub fn getQuotes(buffer: &[u8], index: &mut usize, formatted: bool) -> Token 
{
  let byte1: u8 = buffer[*index]; // Начальный символ кавычки
  let mut result: String = String::new();

  *index += 1;

  let length: usize = buffer.len();
  let mut byte2: u8;

  let mut backslashCount: usize;
  let mut i: usize;
  while *index < length 
  {
    byte2 = buffer[*index]; // Текущий байт
    // Ошибка: конец строки внутри кавычек
    match byte2 
    {
      // Возврат строки не возможен, поскольку она может выйти за скобки и т.п. 
      // если он достиг конца строки уже;
      // todo Комментарии же читают далее - значит возможно;
      //  Должно читать до закрывающего quote.
      b'\n' => { return Token::newEmpty(TokenType::None); }
      // Если мы нашли символ похожий на первый, значит закрываем,
      // но возможно это экранированная кавычка, и не закрываем.
      byte if byte == byte1 =>
      { // Проверка обратных слэшей перед закрывающей кавычкой
        backslashCount = 0;
        i = *index-1;

        while i > 0 && buffer[i] == b'\\' 
        {
          backslashCount += 1;
          i -= 1;
        }

        // Нечетное количество обратных слэшей — кавычка экранирована
        match backslashCount%2 
        {
          1 => result.push(byte2 as char), // Экранированная кавычка
          _ => 
          {
            *index += 1; // Завершение строки
            break;
          }
        }
      }
      // Все иные символы, входящие между кавычек;
      _ => { result.push(byte2 as char); }
    }

    *index += 1;
  }

  // Проверяем тип кавычки и возвращаем соответствующий токен
  match byte1 
  {
    b'\'' => 
    { 
      if formatted || result.len() == 1 
      { // Одинарные кавычки должны содержать только один символ - если не formatted
        Token::new(
          if formatted { TokenType::FormattedChar } else { TokenType::Char },
          result,
        )
      } else {
        Token::newEmpty(TokenType::None)
      }
    }
    b'"' => Token::new(TokenType::String, result),
    b'`' => Token::new(TokenType::RawString, result),
    _ => Token::newEmpty(TokenType::None),
  }
}

// =================================================================================================

#[cfg(test)]
mod tests
{
  use crate::tokenizer::read::primitives::quotes::getQuotes;
  use crate::tokenizer::types::token::{Token};
  use crate::tokenizer::types::tokenType::TokenType;
  // ===============================================================================================

  /// todo desk
  #[test]
  fn value()
  {
    for (input, expectedType, expectedData, formatted) in [
      ("'c'", TokenType::Char, "c", false),
      ("'ab'", TokenType::None, "", false),
      //
      ("\"text\"", TokenType::String, "text", false),
      ("`raw`", TokenType::RawString, "raw", false),
      //
      ("'a", TokenType::Char, "a", false),
      ("'f", TokenType::FormattedChar, "f", true),
      //
      ("\"esc'\"", TokenType::String, "esc'", false),
      ("\"esc`\"", TokenType::String, "esc`", false),
      ("`esc'`", TokenType::RawString, "esc'", false),
      // todo Тут проблемы что нельзя отладить \ перед quotes - оно не работает;
      //  Хотя в коде обычно это работало. Возможно что тут передать нельзя.
    ] {
      let buffer: &[u8] = input.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;
      let token: Token = getQuotes(buffer, &mut index, formatted);
      
      //
      let tokenType: String = token.getDataType().to_string();
      let expectedTypeStr: String = expectedType.to_string();
      assert_eq!(
        tokenType,
        expectedTypeStr,
        "Для '{}' ожидался тип {}, получен {}",
        input,
        expectedTypeStr,
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
        "Индекс для '{}' должен продвинуться на {}, остановился на {}",
        input, bufferLength, index
      );
    }
    //
  }
  
  /// todo desk
  #[test]
  fn index()
  {
    for (input, expectedType, expectedData, expectedIndex, formatted) in [
      ("'a'!", TokenType::Char, "a", 3, false),
      ("\"123\"xyz", TokenType::String, "123", 5, false),
      ("`test`end", TokenType::RawString, "test", 6, false),
      ("\"unterminated", TokenType::String, "unterminated", 13, false),
      //("\"line\n", TokenType::String, "", 5, false) // todo Должно было читать до закрывающей quote
    ] {
      let buffer: &[u8] = input.as_bytes();
      let mut index: usize = 0;
      let token: Token = getQuotes(buffer, &mut index, formatted);
      
      //
      let tokenType: String = token.getDataType().to_string();
      let expectedTypeStr: String = expectedType.to_string();
      assert_eq!(
        tokenType,
        expectedTypeStr,
        "Для '{}' ожидался тип {}, получен {}",
        input,
        expectedTypeStr,
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