use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Проверка на вхождение в срез
macro_rules! matchesIn
{
  ($value:expr, $slice:expr) => 
  {
    $slice.contains(&$value)
  };
}

/// Разделяет токены по типу токена-разделителя
pub fn splitByType(tokens: Vec<Token>, separatorTypes: &[TokenType]) -> Vec<Line>
{
  let mut lines: Vec<Line> = Vec::new();
  let mut buffer: Vec<Token> = Vec::new();

  for token in tokens.into_iter()
  {
    if matchesIn!(token.getDataType(), separatorTypes) 
    {
      lines.push(Line
      {
        tokens: Some(buffer),
        indent: None,
        lines: None,
        parent: None,
      });
      buffer = Vec::new();
    }
    else
    {
      buffer.push(token);
    }
  }

  if !buffer.is_empty()
  {
    lines.push(Line
    {
      tokens: Some(buffer),
      indent: None,
      lines: None,
      parent: None,
    });
  }

  lines
}

// =================================================================================================