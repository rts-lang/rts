/* /tokenizer/line
  A line is a structure made up of consecutive Tokens
*/

use crate::tokenizer::token::*;

use std::sync::{Arc, RwLock};

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