use crate::tokenizer::types::token::{Token, TokenType};
// =================================================================================================

/// Проверяет что байт является одиночным знаком доступным для синтаксиса
pub fn isSingleChar(c: &u8) -> bool
{
  matches!(*c, 
    b'+' | b'-' | b'*' | b'/' | b'=' | b'%' | b'^' |
    b'>' | b'<' | b'?' | b'!' | b'&' | b'|' | 
    b'(' | b')' | b'{' | b'}' | b'[' | b']' | 
    b':' | b',' | b'.' | b'~'
  )
}

// =================================================================================================

/// Проверяет buffer по index и так находит возможные двойные и одиночные операторы
pub fn getOperator(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> Token 
{
  let currentByte: u8 = buffer[*index]; // current byte
  let nextByte: u8 =                    // next byte or \0
    match *index+1 < *bufferLength 
    {
      true  => { buffer[*index+1] } 
      false => { b'\0'}
    };

  let mut increment = |count: usize| 
  { // index increment for single & duble operators
    *index += count;
  };

  match currentByte 
  {
    b'+' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::PlusEquals) }
        b'+' => { increment(2); Token::newEmpty(TokenType::UnaryPlus) }
        _    => { increment(1); Token::newEmpty(TokenType::Plus) }
      }
    }
    b'-' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::MinusEquals) }
        b'-' => { increment(2); Token::newEmpty(TokenType::UnaryMinus) }
        b'>' => { increment(2); Token::newEmpty(TokenType::Pointer) }
        _    => { increment(1); Token::newEmpty(TokenType::Minus) }
      }
    }
    b'*' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::MultiplyEquals) }
        b'*' => { increment(2); Token::newEmpty(TokenType::UnaryMultiply) }
        _    => { increment(1); Token::newEmpty(TokenType::Multiply) }
      }
    }
    b'/' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::DivideEquals) }
        b'/' => { increment(2); Token::newEmpty(TokenType::UnaryDivide) }
        _    => { increment(1); Token::newEmpty(TokenType::Divide) }
      }
    }
    b'%' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::Modulo) } // todo: add new type in Token
        b'%' => { increment(2); Token::newEmpty(TokenType::Modulo) } // todo: add new type in Token
        _    => { increment(1); Token::newEmpty(TokenType::Modulo) }
      }
    }
    b'^' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::Exponent) } // todo: add new type in Token
        b'^' => { increment(2); Token::newEmpty(TokenType::Exponent) } // todo: add new type in Token
        _    => { increment(1); Token::newEmpty(TokenType::Disjoint) }
      }
    }
    b'>' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::GreaterThanOrEquals) }
        _    => { increment(1); Token::newEmpty(TokenType::GreaterThan) }
      }
    }
    b'<' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::LessThanOrEquals) }
        _    => { increment(1); Token::newEmpty(TokenType::LessThan) }
      }
    }
    b'!' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::NotEquals) }
        _    => { increment(1); Token::newEmpty(TokenType::Exclusion) }
      }
    }
    b'~' =>
    {
      match nextByte
      {
        b'~' => { increment(2); Token::newEmpty(TokenType::DoubleTilde) }
        _    => { increment(1); Token::newEmpty(TokenType::Tilde) }
      }
    }
    b'&' => { increment(1); Token::newEmpty(TokenType::Joint) }
    b'|' => { increment(1); Token::newEmpty(TokenType::Inclusion) }
    b'=' => { increment(1); Token::newEmpty(TokenType::Equals) }
    // brackets
    b'(' => { increment(1); Token::newEmpty(TokenType::CircleBracketBegin) }
    b')' => { increment(1); Token::newEmpty(TokenType::CircleBracketEnd) }
    b'{' => { increment(1); Token::newEmpty(TokenType::FigureBracketBegin) }
    b'}' => { increment(1); Token::newEmpty(TokenType::FigureBracketEnd) }
    b'[' => { increment(1); Token::newEmpty(TokenType::SquareBracketBegin) }
    b']' => { increment(1); Token::newEmpty(TokenType::SquareBracketEnd) }
    // other
    b';' => { increment(1); Token::newEmpty(TokenType::Endline) }
    b':' => { increment(1); Token::newEmpty(TokenType::Colon) }
    b',' => { increment(1); Token::newEmpty(TokenType::Comma) }
    b'.' => { increment(1); Token::newEmpty(TokenType::Dot) }
    b'?' => { increment(1); Token::newEmpty(TokenType::Question) }
    _ => Token::newEmpty(TokenType::None)
  }
}

// =================================================================================================