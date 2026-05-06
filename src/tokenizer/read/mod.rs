pub(super) mod numbers;
pub(super) mod words;
pub(super) mod operators;
pub(super) mod quotes;
pub(super) mod comments;

// =================================================================================================

#[cfg(test)]
mod tests
{
  use std::mem;
  use std::sync::{Arc, RwLock, RwLockWriteGuard};
  use crate::tokenizer::types::line::Line;
  use crate::tokenizer::types::token::Token;

  /// Получает bytes -> выдает token types для проверки в тестах
  pub fn getTokensFromBuffer(src: &str) -> Vec<Token>
  {
    let buffer: Vec<u8> = src.as_bytes().to_vec();
    let lines: Vec< Arc<RwLock<Line>> > = crate::tokenizer::tokenizer::readTokens(buffer, false);

    let mut types: Vec<Token> = Vec::new();
    for lineLink in lines
    {
      let mut line: RwLockWriteGuard<Line> = lineLink.write().unwrap();
      if let Some(tokens) = &mut line.tokens
      {
        let taken: Vec<Token> = mem::take(tokens); // Изымаем, т.к. нужно только нам
        for token in taken
        {
          types.push(token);
        }
        //
      }
    }

    types
  }
}

// =================================================================================================