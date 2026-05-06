use crate::tokenizer::types::token::{Token, TokenType};
// =================================================================================================

/// Проверяет что байт является числом
pub fn isDigit(c: &u8) -> bool
{
  *c >= b'0' && *c <= b'9'
}

// =================================================================================================

/// Проверяет buffer по index и так находит возможные примитивные числовые типы данных;
/// `UInt, Int, UFloat, Float, Rational, Complex`
///
/// todo: Ввести Complex числа;
///
/// todo: Ввести работу float с .1 или . как 0.0
pub fn getNumber(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> Token
{
  let mut savedIndex: usize = *index; // index buffer
  let mut result: String = String::from(buffer[savedIndex] as char);
  savedIndex += 1;

  let mut      dot: bool = false; // dot check
  let mut negative: bool = false; // negative check
  let rational: bool = false; // rational check

  let mut byte1: u8; // Текущий символ
  let mut byte2: u8; // Следующий символ
  while savedIndex < *bufferLength
  {
    byte1 = buffer[savedIndex]; // Значение текущего символа
    byte2 =                     // Значение следующего символа
      match savedIndex+1 < *bufferLength
      {
        true  => { buffer[savedIndex+1] }
        false => { b'\0' }
      };

    // todo: use match case
    if !negative && buffer[*index] == b'-'
    { // Int/Float
      result.push(byte1 as char);
      negative = true;
      savedIndex += 1;
    } else
    if isDigit(&byte1)
    { // UInt
      result.push(byte1 as char);
      savedIndex += 1;
    } else
    if byte1 == b'.' && !dot && isDigit(&byte2) //&&
      //savedIndex > 1 && buffer[*index-1] != b'.' // fixed for a.0.1 // todo Я убрал это, но мб зря
    { // UFloat
      match rational
      { false => {}
        true => { break; }
      }
      dot = true;
      result.push(byte1 as char);
      savedIndex += 1;
    } /*else // todo Rational пока что нет как типа
    if byte1 == b'/' && byte2 == b'/' && !dot &&
      (savedIndex+2 < *bufferLength && isDigit(&buffer[savedIndex+2]))
    { // Rational
      rational = true;
      result.push_str("//");
      savedIndex += 2;
    }*/ else { break; }
  }

  *index = savedIndex;

  // next return
  match (rational, dot, negative)
  { // rational, dot, negative
    // (true, _, _)     => Token::new( TokenType::Rational, result ), // todo Rational пока что нет как типа
    (_, true, true)  => Token::new( TokenType::Float,    result ),
    (_, true, false) => Token::new( TokenType::UFloat,   result ),
    (_, false, true) => Token::new( TokenType::Int,      result ),
    _                => Token::new( TokenType::UInt,     result ),
  }
  //
}

// =================================================================================================

#[cfg(test)]
mod tests 
{
  use std::mem;
  use std::sync::{Arc, RwLock, RwLockWriteGuard};
  use crate::tokenizer::tokenizer::readTokens;
  use crate::tokenizer::types::line::Line;
  use crate::tokenizer::types::token::{Token, TokenType};

  // ===============================================================================================
  
  /// Получает bytes -> выдает token types для проверки
  fn getTokensFromBuffer(src: &str) -> Vec<Token> 
  {
    let buffer: Vec<u8> = src.as_bytes().to_vec();
    let lines: Vec< Arc<RwLock<Line>> > = readTokens(buffer, false);
    
    let mut types: Vec<Token> = Vec::new();
    for lineLink in lines 
    {
      let mut line: RwLockWriteGuard<Line> = lineLink.write().unwrap();
      if let Some(tokens) = &mut line.tokens 
      {
        let taken: Vec<Token> = mem::take(tokens); // Изымаем, т.к. нужно только нам
        for token in taken 
        {
          types.push(token);
        }
        //
      }
    }
    
    types
  }

  // ===============================================================================================

  /// Проверяем тип и значение
  #[test]
  fn values() 
  {
    for (src, expectedType) in vec![
      // UInt
      ("0", TokenType::UInt),
      ("1", TokenType::UInt),
      ("1234567890", TokenType::UInt),
      
      // Int
      ("-0", TokenType::Int),
      ("-1", TokenType::Int),
      ("-987654321", TokenType::Int),
      
      // UFloat
      ("3.14", TokenType::UFloat),
      //("3.", TokenType::UFloat), // todo Сейчас нет реализации 3. = 3.0
      //(".14", TokenType::UFloat), // todo Сейчас нет реализации .14 = 0.14
      //(".", TokenType::UInt), // todo Сейчас нет реализации . = 0.0
      
      // UInt
      ("-14.3", TokenType::Float),
      ("-2.5", TokenType::Float),
      ("-100.1000", TokenType::Float),
      
      // Rational
      //("1//2", TokenType::Rational), // todo Rational пока что нет как типа
    ] {
      let tokens: Vec<Token> = getTokensFromBuffer(src);
      
      // Должен быть 1 токен
      assert_eq!(tokens.len(), 1, "Байты '{}' должны были создать ровно 1 токен, а создали '{:?}'", src, tokens);
      
      // Тип должен совпадать
      let tokenType: String = tokens[0].getDataType().to_string();
      let expectedTokenType: String = expectedType.to_string();
      assert_eq!(
        tokenType, expectedTokenType, 
        "Байты '{}' должны были вернуть тип '{}', а вернули '{}'", src, expectedTokenType, tokenType, 
      );
      
      // Значение должно совпадать с изначальным
      let tokenData: String = tokens[0].to_string();
      assert_eq!(tokenData, src, "Ожидались исходные байты '{}', а получили '{}'", src, tokenData);
    }
  }
  
  /// Проверяет разделение пробелами
  #[test]
  fn split()
  {
    // UInt
    let tokens: Vec<Token> = getTokensFromBuffer("1 2 3");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].getDataType().to_string(), TokenType::UInt.to_string());
    assert_eq!(tokens[1].getDataType().to_string(), TokenType::UInt.to_string());
    assert_eq!(tokens[2].getDataType().to_string(), TokenType::UInt.to_string());

    // Int
    let tokens: Vec<Token> = getTokensFromBuffer("-1 -2 -3");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].getDataType().to_string(), TokenType::Int.to_string());
    assert_eq!(tokens[1].getDataType().to_string(), TokenType::Int.to_string());
    assert_eq!(tokens[2].getDataType().to_string(), TokenType::Int.to_string());

    // UFloat
    let tokens: Vec<Token> = getTokensFromBuffer("1.1 2.2 3.3");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].getDataType().to_string(), TokenType::UFloat.to_string());
    assert_eq!(tokens[1].getDataType().to_string(), TokenType::UFloat.to_string());
    assert_eq!(tokens[2].getDataType().to_string(), TokenType::UFloat.to_string());

    // Float
    let tokens: Vec<Token> = getTokensFromBuffer("-1.1 -2.2 -3.3");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].getDataType().to_string(), TokenType::Float.to_string());
    assert_eq!(tokens[1].getDataType().to_string(), TokenType::Float.to_string());
    assert_eq!(tokens[2].getDataType().to_string(), TokenType::Float.to_string());

    // Rational
    // todo Rational пока что нет как типа
    //let tokens: Vec<Token> = getTokensFromBuffer("1//2 3//4 5//6");
    //assert_eq!(tokens.len(), 3);
    //assert_eq!(tokens[0].getDataType().to_string(), TokenType::Rational.to_string());
    //assert_eq!(tokens[1].getDataType().to_string(), TokenType::Rational.to_string());
    //assert_eq!(tokens[2].getDataType().to_string(), TokenType::Rational.to_string());
  }

  /// Проверяет максимальный размер
  #[test]
  fn maxSize()
  {
    // todo В целом это логика не tokenizer, а value;
    //  Но пока что не известно будут ли лимиты на типы данных, 
    //  следует ли их проверять заранее - поэтому пока что оно останется тут.
    //  Хотя лимитов не должно быть у примитивов.
  }
  
  /// Проверяет приравнивание (т.е. он не один токен и не в начале)
  #[test]
  fn equal()
  {
    // UInt
    let tokens: Vec<Token> = getTokensFromBuffer("a=10");
    assert!(tokens.len() >= 3);
    assert_eq!(tokens[0].to_string(), "a");
    assert_eq!(tokens[1].to_string(), "=");
    assert_eq!(tokens[2].getDataType().to_string(), TokenType::UInt.to_string());

    // Int
    let tokens: Vec<Token> = getTokensFromBuffer("b=-20");
    assert!(tokens.len() >= 3);
    assert_eq!(tokens[0].to_string(), "b");
    assert_eq!(tokens[1].to_string(), "=");
    assert_eq!(tokens[2].getDataType().to_string(), TokenType::Int.to_string());

    // UFloat
    let tokens: Vec<Token> = getTokensFromBuffer("c=3.14");
    assert!(tokens.len() >= 3);
    assert_eq!(tokens[0].to_string(), "c");
    assert_eq!(tokens[1].to_string(), "=");
    assert_eq!(tokens[2].getDataType().to_string(), TokenType::UFloat.to_string());

    // Float
    let tokens: Vec<Token> = getTokensFromBuffer("d=-2.5");
    assert!(tokens.len() >= 3);
    assert_eq!(tokens[0].to_string(), "d");
    assert_eq!(tokens[1].to_string(), "=");
    assert_eq!(tokens[2].getDataType().to_string(), TokenType::Float.to_string());

    // todo Rational пока что нет как типа
    // let tokens: Vec<Token> = getTokensFromBuffer("e=1//2");
  }

  // ===============================================================================================
}

// =================================================================================================