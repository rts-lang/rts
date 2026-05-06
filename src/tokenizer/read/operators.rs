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
  use crate::tokenizer::read::tests::{checkSplit, checkThroughOthers, checkValues, getTokensFromBuffer};
  use crate::tokenizer::types::token::{Token, TokenType};

  // ===============================================================================================
  
  /// Проверяем тип и значение
  #[test]
  fn values() -> ()
  {
    checkValues([
      // single math
      ("+", TokenType::Plus),
      ("-", TokenType::Minus),
      ("*", TokenType::Multiply),
      ("/", TokenType::Divide),
      ("=", TokenType::Equals),
      ("%", TokenType::Modulo),
      ("^", TokenType::Disjoint),
      // double math
      ("++", TokenType::UnaryPlus),
      ("+=", TokenType::PlusEquals),
      ("--", TokenType::UnaryMinus),
      ("-=", TokenType::MinusEquals),
      ("**", TokenType::UnaryMultiply),
      ("*=", TokenType::MultiplyEquals),
      ("//", TokenType::UnaryDivide),
      ("/=", TokenType::DivideEquals),
      ("%%", TokenType::Modulo), // todo: add new type in Token
      ("%=", TokenType::Modulo), // todo: add new type in Token
      ("^^", TokenType::Exponent), // todo: add new type in Token
      ("^=", TokenType::Exponent), // todo: add new type in Token
      // single logical
      (">", TokenType::GreaterThan),
      ("<", TokenType::LessThan),
      ("?", TokenType::Question),
      ("!", TokenType::Exclusion),
      // double logical
      (">=", TokenType::GreaterThanOrEquals),
      ("<=", TokenType::LessThanOrEquals),
      ("!=", TokenType::NotEquals),
      // other
      (":", TokenType::Colon),
      ("->", TokenType::Pointer),
      ("~", TokenType::Tilde),
      ("~~", TokenType::DoubleTilde),
      ("&", TokenType::Joint),
      ("|", TokenType::Inclusion),
      // brackets
      ("(", TokenType::CircleBracketBegin),
      (")", TokenType::CircleBracketEnd),
      ("[", TokenType::SquareBracketBegin),
      ("]", TokenType::SquareBracketEnd),
      ("{", TokenType::FigureBracketBegin),
      ("}", TokenType::FigureBracketEnd),
      // separators
      (",", TokenType::Comma),
      (".", TokenType::Dot)
      // Endline не рассматривается т.к. будет использован при чтении
    ], false);
  }
  
  /// Проверяет разделение пробелами на несколько токенов
  #[test]
  fn split() -> ()
  {
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
  fn throughOthers() -> ()
  {
    checkThroughOthers([
      ("1 a +", "1", "a", TokenType::Plus),
      ("1 a -", "1", "a", TokenType::Minus),
      ("1 a *", "1", "a", TokenType::Multiply),
      ("1 a /", "1", "a", TokenType::Divide),
      ("1 a =", "1", "a", TokenType::Equals),
      ("1 a %", "1", "a", TokenType::Modulo),
      ("1 a ^", "1", "a", TokenType::Disjoint),

      ("1 a ++", "1", "a", TokenType::UnaryPlus),
      ("1 a +=", "1", "a", TokenType::PlusEquals),
      ("1 a --", "1", "a", TokenType::UnaryMinus),
      ("1 a -=", "1", "a", TokenType::MinusEquals),
      ("1 a **", "1", "a", TokenType::UnaryMultiply),
      ("1 a *=", "1", "a", TokenType::MultiplyEquals),
      ("1 a //", "1", "a", TokenType::UnaryDivide),
      ("1 a /=", "1", "a", TokenType::DivideEquals),
      ("1 a %%", "1", "a", TokenType::Modulo), // todo: add new type in Token
      ("1 a %=", "1", "a", TokenType::Modulo), // todo: add new type in Token
      ("1 a ^^", "1", "a", TokenType::Exponent), // todo: add new type in Token
      ("1 a ^=", "1", "a", TokenType::Exponent), // todo: add new type in Token

      ("1 a >", "1", "a", TokenType::GreaterThan),
      ("1 a <", "1", "a", TokenType::LessThan),
      ("1 a ?", "1", "a", TokenType::Question),
      ("1 a !", "1", "a", TokenType::Exclusion),

      ("1 a >=", "1", "a", TokenType::GreaterThanOrEquals),
      ("1 a <=", "1", "a", TokenType::LessThanOrEquals),
      ("1 a !=", "1", "a", TokenType::NotEquals),

      ("1 a :", "1", "a", TokenType::Colon),
      ("1 a ->", "1", "a", TokenType::Pointer),
      ("1 a ~", "1", "a", TokenType::Tilde),
      ("1 a ~~", "1", "a", TokenType::DoubleTilde),
      ("1 a &", "1", "a", TokenType::Joint),
      ("1 a |", "1", "a", TokenType::Inclusion),

      ("1 a (", "1", "a", TokenType::CircleBracketBegin),
      ("1 a [", "1", "a", TokenType::SquareBracketBegin),
      ("1 a {", "1", "a", TokenType::FigureBracketBegin),
      
      ("1 a )", "1", "a", TokenType::CircleBracketEnd),
      ("1 a ]", "1", "a", TokenType::SquareBracketEnd),
      ("1 a }", "1", "a", TokenType::FigureBracketEnd),

      ("1 a ,", "1", "a", TokenType::Comma),
      ("1 a .", "1", "a", TokenType::Dot)

      // Endline не рассматривается т.к. будет использован при чтении
    ]);
  }
  
  /// Неверные последовательности выдают None
  #[test]
  fn invalid() -> ()
  {
    for (input, expectedLen, index, expectedType) in [
      ("1.2", 1, 0, TokenType::UFloat),
      ("1--2", 3, 1, TokenType::UnaryMinus),
      ("@ $", 0, 0, TokenType::None),
    ] {
      let tokens: Vec<Token> = getTokensFromBuffer(input);

      //
      let tokensLen: usize = tokens.len();
      assert_eq!(tokensLen, expectedLen,
                 "Байты '{}' должны были создать {} токенов, а создали {}:{:?}",
                 input, expectedLen, tokensLen, tokens);

      //
      if expectedLen > 0
      {
        let tokenType: String = tokens[index].getDataType().to_string();
        let expectedTokenType: String = expectedType.to_string();
        assert_eq!(tokenType, expectedTokenType,
                   "Байты '{}' должны были создать токен типа '{}', а создали '{}'",
                   input, expectedTokenType, tokenType);
      }
    }
    //
  }

  // ===============================================================================================
}

// =================================================================================================