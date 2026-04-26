use std::sync::{RwLock, RwLockReadGuard};
use std::sync::Arc;
use wasm_bindgen::prelude::wasm_bindgen;
use serde::Serialize;
use serde_json::to_string;
use crate::tokenizer::line::Line;
use crate::tokenizer::token::Token;
use crate::tokenizer::tokenizer::readTokens;
// =================================================================================================

/// Выходная сериализуемая структура
#[derive(Serialize, Clone)]
pub struct AnalyzeToken 
{
  pub start: usize,
  pub end: usize,
  pub kind: String,
}

/// Плоский список токенов с абсолютными позициями.
/// Используется для подсветки, LSP, WASM, IDE ...
#[wasm_bindgen]
pub fn tokenize(code: &str) -> String
{
  let buffer: Vec<u8> = code.as_bytes().to_vec();
  let lines: Vec< Arc<RwLock<Line>> > = readTokens(buffer, false);
  let mut result: Vec<AnalyzeToken> = Vec::new();
  flattenLines(&lines, &mut result);
  to_string(&result).unwrap_or_else(|_| "[]".to_string())
}

/// Рекурсивно собираем все токены из дерева линий (с поддержкой вложенных f‑строк)
fn flattenLines(lines: &[Arc<RwLock<Line>>], out: &mut Vec<AnalyzeToken>) 
{
  for linLink in lines 
  {
    let line: RwLockReadGuard<Line> = linLink.read().unwrap();

    // токены текущей линии
    if let Some(tokens) = &line.tokens {
      flattenTokens(tokens, out);
    }

    // вложенные линии (например, тело функции, блоки)
    if let Some(nested) = &line.lines {
      flattenLines(nested, out);
    }
  }
  //
}

/// Обход списка токенов, включая их внутренние линии (f‑строки)
fn flattenTokens(tokens: &[Token], out: &mut Vec<AnalyzeToken>) 
{
  for token in tokens 
  {
    // Добавляем сам токен
    #[cfg(feature = "analyzer")]
    out.push(AnalyzeToken {
      start: token.start,
      end: token.end,
      kind: token.getDataType().to_string(),
    });

    // Если у токена есть вложенные линии
    if let Some(lines) = &token.lines 
    {
      for line in lines 
      {
        // У каждой такой Line свои токены
        if let Some(tokens) = &line.tokens 
        {
          flattenTokens(tokens, out);
        }
        // Если внутри ещё есть вложенные линии
        if let Some(nested) = &line.lines {
          flattenLines(nested, out);
        }
      }
      //
    }
  }
  //
}

// =================================================================================================