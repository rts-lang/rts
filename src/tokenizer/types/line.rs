use std::sync::{Arc, RwLock};
use crate::tokenizer::types::token::Token;
// =================================================================================================

/// Это последовательный набор токенов
#[derive(Clone)]
pub struct Line
{
  /// Список вложенных токенов
  pub tokens: Option< Vec<Token> >,
  /// Вложенные линии
  pub lines: Option< Vec< Arc<RwLock<Self>> > >,
}
impl Line 
{
  pub const fn newEmpty() -> Self 
  {
    Self 
    {
      tokens: None,
      lines: None
    }
  }
}

// =================================================================================================

#[cfg(test)]
mod tests
{
  use std::sync::{Arc, RwLock};
  use crate::tokenizer::types::line::Line;
  use crate::tokenizer::types::token::Token;
  use crate::tokenizer::types::tokenType::TokenType;
  // ===============================================================================================

  /// todo desk (чек 2 токена)
  #[test]
  fn withTokens() -> ()
  {
    let token1: Token = Token::newEmpty(TokenType::Word);
    let token2: Token = Token::newEmpty(TokenType::UInt);
    let line: Line = Line {
      tokens: Some(vec![token1, token2]),
      lines: None
    };

    //
    assert!(line.tokens.is_some(), "tokens must be Some");
    assert_eq!(line.tokens.as_ref().unwrap().len(), 2, "The length of tokens must be 2");
  }
  
  /// todo desk (чек токена)
  ///  Мне кажется что тут баг так как `indent` а по итогу 1 токен чек. Видать было что-то другое,
  ///  но теперь нет indent - мб бесполезный и удалить его?
  #[test]
  fn withIndent() -> ()
  {
    let token: Token = Token::newEmpty(TokenType::True);
    let line: Line = Line {
      tokens: Some(vec![token]),
      lines: None
    };

    //
    assert_eq!(line.tokens.as_ref().unwrap().len(), 1, "The length of tokens must be 1");
  }
  
  /// todo desk (чек вложения)
  #[test]
  fn nestedLines() -> ()
  {
    let inner: Line = Line::newEmpty();
    let innerArc: Arc<RwLock<Line>> = Arc::new(RwLock::new(inner));
    let outer: Line = Line {
      tokens: None,
      lines: Some(vec![innerArc])
    };

    //
    assert!(outer.lines.is_some(), "lines must be Some");
    assert_eq!(outer.lines.as_ref().unwrap().len(), 1, "The length of lines must be 1");
  }
  
  // ===============================================================================================
}

// =================================================================================================