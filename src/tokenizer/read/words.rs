use crate::tokenizer::read::numbers::isDigit;
use crate::tokenizer::types::token::{Token, TokenType};
// =================================================================================================

/// Проверяет что байт является буквой a-z A-Z
pub fn isLetter(c: &u8) -> bool
{
  (c|32)>=b'a'&&(c|32)<=b'z'
}

// =================================================================================================

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
      match result.as_str()
      {
        "Bool"  => Token::newEmpty(TokenType::Bool), // todo Важно: оно не имеет значения, что спорно
        "true"  => Token::new(TokenType::Bool, result),
        "false" => Token::new(TokenType::Bool, result),
        //
        "UInt"   => Token::newEmpty(TokenType::UInt),
        "Int"    => Token::newEmpty(TokenType::Int),
        "UFloat" => Token::newEmpty(TokenType::UFloat),
        "Float"  => Token::newEmpty(TokenType::Float),
        //"Rational"  => Token::newEmpty(TokenType::Rational), // todo Rational пока что нет как типа
        //
        "Char"      => Token::newEmpty(TokenType::Char),
        "String"    => Token::newEmpty(TokenType::String),
        "RawString" => Token::newEmpty(TokenType::RawString),
        //
        "FormattedChar"      => Token::newEmpty(TokenType::FormattedChar),
        "FormattedString"    => Token::newEmpty(TokenType::FormattedString),
        "FormattedRawString" => Token::newEmpty(TokenType::FormattedRawString),
        //
        "None" => Token::newEmpty(TokenType::None),
        //
        _ => Token::new(TokenType::Word, result),
      }
      //
    }
  }
  //
}

// =================================================================================================

#[cfg(test)]
mod tests
{
  use crate::tokenizer::read::tests::{checkSplit, checkThroughOthers, checkValues, getTokensFromBuffer};
  use crate::tokenizer::types::token::{Token, TokenType};

  // ===============================================================================================

  /// Проверяем тип и значение
  #[test]
  fn values() -> ()
  {
    checkValues([
      // Bool
      ("true", TokenType::Bool),
      ("false", TokenType::Bool),

      // Numbers
      ("UInt", TokenType::UInt),
      ("Int", TokenType::Int),
      ("UFloat", TokenType::UFloat),
      ("Float", TokenType::Float),

      // Strings
      ("Char", TokenType::Char),
      ("String", TokenType::String),
      ("RawString", TokenType::RawString),

      // Formatted strings
      ("FormattedChar", TokenType::FormattedChar),
      ("FormattedString", TokenType::FormattedString),
      ("FormattedRawString", TokenType::FormattedRawString),

      // None
      ("None", TokenType::None)
    ], true);
  }

  /// Проверяем ссылки - статические и динамические
  #[test]
  fn links() -> ()
  {
    for (src, expectedType) in vec![
      ("a.",        TokenType::Link),
      ("var.name",  TokenType::Link),
      ("obj.prop[0]", TokenType::Link),
      ("data.list[1].value", TokenType::Link),
    ] {
      let tokens: Vec<Token> = getTokensFromBuffer(src);
      
      //
      assert_eq!(tokens.len(), 1,
                 "Байты '{}' должны были создать ровно 1 токен, а создали {}", src, tokens.len());
      
      //
      let tokenType: String = tokens[0].getDataType().to_string();
      let expectedType: String = expectedType.to_string();
      assert_eq!(tokenType, expectedType,
                 "Байты '{}' должны были создать токен {:?}, а создали {:?}", src, expectedType, tokenType);
      
      //
      let tokenData: String = tokens[0].to_string();
      assert_eq!(tokenData, src,
                 "Ожидались исходные байты '{}', а получили '{}':'{}'", src, tokenData, tokenType);
    }
  }

  /// Проверяет разделение пробелами на несколько токенов
  #[test]
  fn split() -> ()
  {
    checkSplit(&[
      // Простейшие custom слова
      ("a b c", &[TokenType::Word, TokenType::Word, TokenType::Word]),
      // Длинные custom слова
      ("hello world", &[TokenType::Word, TokenType::Word]),
      // Все типы
      ("Bool UInt Int UFloat Float Char String RawString FormattedChar FormattedString FormattedRawString None", 
       &[
         TokenType::Bool, 
         //
         TokenType::UInt, 
         TokenType::Int, 
         TokenType::UFloat, 
         TokenType::Float, 
         // TokenType::Rational, // todo Rational пока что нет как типа
         //
         TokenType::Char, 
         TokenType::String,
         TokenType::RawString,
         //
         TokenType::FormattedChar,
         TokenType::FormattedString,
         TokenType::FormattedRawString,
         //
         TokenType::None
       ]
      ),
      // Статические ссылки (динамические здесь не должны проверяться)
      ("a.name b.prop", &[TokenType::Link, TokenType::Link]),
    ]);
  }

  /// Проверяет через несколько токенов
  #[test]
  fn throughOthers() -> ()
  {
    checkThroughOthers([
      ("a=true", "a", "=", TokenType::Bool),
      ("a=false", "a", "=", TokenType::Bool),

      ("a:UInt", "a", ":", TokenType::UInt),
      ("a:Int", "a", ":", TokenType::Int),
      ("a:UFloat", "a", ":", TokenType::UFloat),
      ("a:Float", "a", ":", TokenType::Float),
      //("xxx:Rational", "g", ":", TokenType::Float), // todo Rational пока что нет как типа

      ("a:Char", "a", ":", TokenType::Char),
      ("a:String", "a", ":", TokenType::String),
      ("a:RawString", "a", ":", TokenType::RawString),

      ("a:FormattedChar", "a", ":", TokenType::FormattedChar),
      ("a:FormattedString", "a", ":", TokenType::FormattedString),
      ("a:FormattedRawString", "a", ":", TokenType::FormattedRawString),

      ("m=None", "m", "=", TokenType::None)
    ]);
  }

  // ===============================================================================================
}

// =================================================================================================