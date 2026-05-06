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

#[cfg(test)]
mod tests {
  use crate::tokenizer::read::tests::{checkSplit, checkThroughOthers, checkValues};
  use crate::tokenizer::types::token::{TokenType};

  // ===============================================================================================

  /*
  /// Проверяем тип и значение
  #[test]
  fn values() 
  {
    checkValues([
      // single math
      ("+", TokenType::Plus, false),
      ("-", TokenType::Minus, false),
      ("*", TokenType::Multiply, false),
      ("/", TokenType::Divide, false),
      ("=", TokenType::Equals, false),
      ("%", TokenType::Modulo, false),
      ("^", TokenType::Disjoint, false),
      // double math
      ("++", TokenType::UnaryPlus, false),
      ("+=", TokenType::PlusEquals, false),
      ("--", TokenType::UnaryMinus, false),
      ("-=", TokenType::MinusEquals, false),
      ("**", TokenType::UnaryMultiply, false),
      ("*=", TokenType::MultiplyEquals, false),
      ("//", TokenType::UnaryDivide, false),
      ("/=", TokenType::DivideEquals, false),
      ("%%", TokenType::Modulo, false),
      ("%=", TokenType::Modulo, false),
      ("^^", TokenType::Exponent, false),
      ("^=", TokenType::Exponent, false),
      // single logical
      (">", TokenType::GreaterThan, false),
      ("<", TokenType::LessThan, false),
      ("?", TokenType::Question, false),
      ("!", TokenType::Exclusion, false),
      // double logical
      (">=", TokenType::GreaterThanOrEquals, false),
      ("<=", TokenType::LessThanOrEquals, false),
      ("!=", TokenType::NotEquals, false),
      // other
      (":", TokenType::Colon, false),
      ("->", TokenType::Pointer, false),
      ("~", TokenType::Tilde, false),
      ("~~", TokenType::DoubleTilde, false),
      ("&", TokenType::Joint, false),
      ("|", TokenType::Inclusion, false),
      // brackets
      ("(", TokenType::CircleBracketBegin, false),
      (")", TokenType::CircleBracketEnd, false),
      ("[", TokenType::SquareBracketBegin, false),
      ("]", TokenType::SquareBracketEnd, false),
      ("{", TokenType::FigureBracketBegin, false),
      ("}", TokenType::FigureBracketEnd, false),
      // separators
      (";", TokenType::Endline, false),
      (",", TokenType::Comma, false),
      (".", TokenType::Dot, false),
    ]);
  }
  */
  
  /// Проверяет разделение пробелами на несколько токенов
  #[test]
  fn split() {
    checkSplit(&[
      // Простейшие операторы
      ("+ - * / = % ^", &[
        TokenType::Plus, TokenType::Minus, TokenType::Multiply,
        TokenType::Divide, TokenType::Equals, TokenType::Modulo, TokenType::Disjoint
      ]),
      // Двойные операторы
      ("++ += -- -=", &[
        TokenType::UnaryPlus, TokenType::PlusEquals,
        TokenType::UnaryMinus, TokenType::MinusEquals
      ]),
      ("** *= // /=", &[
        TokenType::UnaryMultiply, TokenType::MultiplyEquals,
        TokenType::UnaryDivide, TokenType::DivideEquals
      ]),
      ("%% %= ^^ ^=", &[
        TokenType::Modulo, TokenType::Modulo, // todo: add new type in Token
        TokenType::Exponent, TokenType::Exponent // todo: add new type in Token
      ]),
      // Логические операторы
      ("> < ? !", &[
        TokenType::GreaterThan, TokenType::LessThan,
        TokenType::Question, TokenType::Exclusion
      ]),
      (">= <= !=", &[
        TokenType::GreaterThanOrEquals, TokenType::LessThanOrEquals,
        TokenType::NotEquals
      ]),
      // Скобки и разделители
      ("( ) [ ] { } , .", &[
        TokenType::CircleBracketBegin, TokenType::CircleBracketEnd,
        TokenType::SquareBracketBegin, TokenType::SquareBracketEnd,
        TokenType::FigureBracketBegin, TokenType::FigureBracketEnd,
        TokenType::Comma, TokenType::Dot // Endline не рассматривается т.к. будет использован при чтении
      ]),
      // Прочие
      (": -> ~ ~~ & |", &[
        TokenType::Colon, TokenType::Pointer, TokenType::Tilde,
        TokenType::DoubleTilde, TokenType::Joint, TokenType::Inclusion
      ]),
      // Смесь
      ("+ += ++", &[
        TokenType::Plus, TokenType::PlusEquals, TokenType::UnaryPlus
      ]),
      ("- -= --", &[
        TokenType::Minus, TokenType::MinusEquals, TokenType::UnaryMinus
      ]),
    ]);
  }

  /// Проверяет через несколько токенов
  #[test]
  fn throughOthers() 
  {
    checkThroughOthers([
      ("1a+", "1", "a", TokenType::Plus),
      ("1a-", "1", "a", TokenType::Minus),
      ("1a*", "1", "a", TokenType::Multiply),
      ("1a/", "1", "a", TokenType::Divide),
      ("1a=", "1", "a", TokenType::Equals),
      ("1a%", "1", "a", TokenType::Modulo),
      ("1a^", "1", "a", TokenType::Disjoint),

      ("1a++", "1", "a", TokenType::UnaryPlus),
      ("1a+=", "1", "a", TokenType::PlusEquals),
      ("1a--", "1", "a", TokenType::UnaryMinus),
      ("1a-=", "1", "a", TokenType::MinusEquals),
      ("1a**", "1", "a", TokenType::UnaryMultiply),
      ("1a*=", "1", "a", TokenType::MultiplyEquals),
      ("1a//", "1", "a", TokenType::UnaryDivide),
      ("1a/=", "1", "a", TokenType::DivideEquals),
      ("1a%%", "1", "a", TokenType::Modulo), // todo: add new type in Token
      ("1a%=", "1", "a", TokenType::Modulo), // todo: add new type in Token
      ("1a^^", "1", "a", TokenType::Exponent), // todo: add new type in Token
      ("1a^=", "1", "a", TokenType::Exponent), // todo: add new type in Token

      ("1a>", "1", "a", TokenType::GreaterThan),
      ("1a<", "1", "a", TokenType::LessThan),
      ("1a?", "1", "a", TokenType::Question),
      ("1a!", "1", "a", TokenType::Exclusion),

      ("1a>=", "1", "a", TokenType::GreaterThanOrEquals),
      ("1a<=", "1", "a", TokenType::LessThanOrEquals),
      ("1a!=", "1", "a", TokenType::NotEquals),

      ("1a:", "1", "a", TokenType::Colon),
      ("1a->", "1", "a", TokenType::Pointer),
      ("1a~", "1", "a", TokenType::Tilde),
      ("1a~~", "1", "a", TokenType::DoubleTilde),
      ("1a&", "1", "a", TokenType::Joint),
      ("1a|", "1", "a", TokenType::Inclusion),

      ("1a(", "1", "a", TokenType::CircleBracketBegin),
      ("1a[", "1", "a", TokenType::SquareBracketBegin),
      ("1a{", "1", "a", TokenType::FigureBracketBegin),
      
      ("1a)", "1", "a", TokenType::CircleBracketEnd),
      ("1a]", "1", "a", TokenType::SquareBracketEnd),
      ("1a}", "1", "a", TokenType::FigureBracketEnd),

      ("1a,", "1", "a", TokenType::Comma),
      ("1a.", "1", "a", TokenType::Dot),

      // Endline не рассматривается т.к. будет использован при чтении
    ]);
  }

  /* todo
  /// Неверные последовательности выдают None
  #[test]
  fn invalid_sequences() {
    // Например, точка в середине числа не должна стать оператором Dot
    let tokens = getTokensFromBuffer("1.2");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].getDataType().to_string(), TokenType::UFloat.to_string());

    // Двойной минус внутри числа не должен быть оператором
    let tokens = getTokensFromBuffer("1--2");
    assert_eq!(tokens.len(), 3); // 1, --, 2
    assert_eq!(tokens[1].getDataType().to_string(), TokenType::UnaryMinus.to_string());

    // Символы, не входящие в операторы, игнорируются
    let tokens = getTokensFromBuffer("@#$");
    assert_eq!(tokens.len(), 0); // Ни одного токена
  }
  */

  // ===============================================================================================
}

// =================================================================================================