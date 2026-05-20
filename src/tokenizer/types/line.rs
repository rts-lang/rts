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
  pub indent: Option<usize>,
  /// Вложенные линии
  pub lines: Option< Vec< Arc<RwLock<Line>> > >,
  /// Ссылка на родителя
  pub parent: Option< Arc<RwLock<Line>> >
}
impl Line 
{
  pub fn newEmpty() -> Self 
  {
    Line 
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
  use crate::tokenizer::types::line::Line;
  // ===============================================================================================
  
  /// todo desk
  #[test]
  fn newEmpty()
  {
    let line: Line = Line::newEmpty();
    assert!(line.tokens.is_none(), "tokens должны быть None");
    assert!(line.indent.is_none(), "indent должен быть None");
    assert!(line.lines.is_none(), "lines должны быть None");
    assert!(line.parent.is_none(), "parent должен быть None");
  }
  
  // ===============================================================================================
}

// =================================================================================================