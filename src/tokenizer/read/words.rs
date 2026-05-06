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
      (isLink && byte1 == b'[' || byte1 == b']') // В случае ссылки мы можем читать динамические []
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
    true => Token::new( TokenType::Link, result.clone() ),
    false =>
    {
      match result.as_str()
      {
        "true"     => Token::newEmpty( TokenType::Bool ),
        "false"    => Token::newEmpty( TokenType::Bool ),
        //
        "UInt"     => Token::newEmpty( TokenType::UInt ),
        "Int"      => Token::newEmpty( TokenType::Int ),
        "UFloat"   => Token::newEmpty( TokenType::UFloat ),
        "Float"    => Token::newEmpty( TokenType::Float ),
        //
        "Char"     => Token::newEmpty( TokenType::Char ),
        "String"   => Token::newEmpty( TokenType::String ),
        "RawString"=> Token::newEmpty( TokenType::RawString ),
        //
        "FormattedChar"     => Token::newEmpty( TokenType::FormattedChar ),
        "FormattedString"   => Token::newEmpty( TokenType::FormattedString ),
        "FormattedRawString"=> Token::newEmpty( TokenType::FormattedRawString ),
        //
        "None"     => Token::newEmpty(TokenType::None),
        //
        _          => Token::new( TokenType::Word, result ),
      }
      //
    }
  }
  //
}

// =================================================================================================