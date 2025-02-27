/* /tokenizer/line
  A line is a structure made up of consecutive Tokens
*/

use crate::tokenizer::token::*;

use std::sync::{Arc, RwLock};

// Line
#[derive(Clone)]
pub struct Line {
  pub tokens: Vec<Token>,                        // list
                                                 // todo: Option<Vec< Arc<RwLock<Token>> >>
  pub indent: usize,                             // indentation
                                                 // todo: Option
  pub lines: Option< Vec< Arc<RwLock<Line>> > >, // child lines
  pub parent: Option< Arc<RwLock<Line>> >        // parent link
}
impl Line 
{
  pub fn newEmpty() -> Self 
  {
    Line 
    {
      tokens: Vec::new(),
      indent: 0,
       lines: None,
      parent: None
    }
  }
}