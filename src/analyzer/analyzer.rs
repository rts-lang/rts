
use crate::tokenizer::token::TokenType;
use std::sync::RwLock;
use std::sync::Arc;
use crate::tokenizer::line::Line;
use crate::tokenizer::tokenizer::readTokens;

#[derive(Clone)]
pub struct AnalyzeToken {
  pub start: usize,
  pub end: usize,
  pub kind: TokenType,
}

/// Плоский список токенов с абсолютными позициями.
/// Используется для подсветки, LSP, WASM, IDE ...
pub fn tokenize(code: &str) -> Vec<AnalyzeToken> {
  let buffer = code.as_bytes().to_vec();
  // false = без debug-вывода в консоль
  let lines = readTokens(buffer, false);

  let mut result = Vec::new();
  flatten_lines(&lines, &mut result, 0);
  result
}

/// Рекурсивно разворачиваем дерево линий в плоский список
fn flatten_lines(
  lines: &[Arc<RwLock<Line>>],
  out: &mut Vec<AnalyzeToken>,
  offset: usize,
) {
  for line_link in lines {
    let line = line_link.read().unwrap();

    // Добавляем токены текущей линии
    if let Some(tokens) = &line.tokens {
      for token in tokens.iter() {
        // Здесь важно: текущий токенайзер не хранит абсолютные позиции.
        // Ниже покажу минимальную правку в tokenizer.rs, чтобы это работало.
        // Пока используем заглушку: позиции нужно будет пробросить из readTokens.
      }
    }

    // Рекурсия по вложенным линиям
    if let Some(nested) = &line.lines {
      // offset += line_length + 1; // упрощённо
      flatten_lines(nested, out, offset);
    }
  }
}