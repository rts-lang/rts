use crate::tokenizer::types::token::{Token, TokenType};
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