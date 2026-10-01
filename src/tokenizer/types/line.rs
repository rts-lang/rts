use std::sync::{Arc, RwLock};
use crate::tokenizer::types::token::Token;
// =================================================================================================

/// Это последовательный набор токенов
#[derive(Clone)]
pub struct Line
{
  /// Список вложенных токенов
  pub tokens: Option< Vec<Token> >,
  /// Уровень отступа
  pub indent: Option<usize>, // todo Indent больше не нужен - убрать, но тесты надо переписать;
                             //  Также это потянет за собой parser - а там кучу мест.
                             //  (Кстати сейчас он же вообще не юзается?)
  /// Вложенные линии
  pub lines: Option< Vec< Arc<RwLock<Self>> > >,
  /// Ссылка на родителя
  pub parent: Option< Arc<RwLock<Self>> >
}
impl Line 
{
  pub const fn newEmpty() -> Self 
  {
    Self 
    {
      tokens: None,
      indent: None,
      lines: None,
      parent: None
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

  /// todo desk
  #[test]
  fn withTokens() -> ()
  {
    let token1: Token = Token::newEmpty(TokenType::Word);
    let token2: Token = Token::newEmpty(TokenType::UInt);
    let line: Line = Line {
      tokens: Some(vec![token1, token2]),
      indent: None,
      lines: None,
      parent: None
    };

    //
    assert!(line.tokens.is_some(), "tokens must be Some");
    assert_eq!(line.tokens.as_ref().unwrap().len(), 2, "The length of tokens must be 2");
    assert!(line.indent.is_none(), "indent must be None");
  }
  
  /// todo desk
  #[test]
  fn withIndent() -> ()
  {
    let token: Token = Token::newEmpty(TokenType::Bool);
    let line: Line = Line {
      tokens: Some(vec![token]),
      indent: Some(4),
      lines: None,
      parent: None
    };

    //
    assert_eq!(line.indent.unwrap(), 4, "indent must be 4");
    assert_eq!(line.tokens.as_ref().unwrap().len(), 1, "The length of tokens must be 1");
  }
  
  /// todo desk
  #[test]
  fn nestedLines() -> ()
  {
    let inner: Line = Line::newEmpty();
    let innerArc: Arc<RwLock<Line>> = Arc::new(RwLock::new(inner));
    let outer: Line = Line {
      tokens: None,
      indent: Some(0),
      lines: Some(vec![innerArc]),
      parent: None
    };

    //
    assert!(outer.lines.is_some(), "lines must be Some");
    assert_eq!(outer.lines.as_ref().unwrap().len(), 1, "The length of lines must be 1");
  }
  
  // ===============================================================================================
}

// =================================================================================================