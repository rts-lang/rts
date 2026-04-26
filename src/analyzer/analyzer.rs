use std::collections::HashSet;
use std::sync::{RwLock, RwLockReadGuard};
use std::sync::Arc;
use lazy_static::lazy_static;
use wasm_bindgen::prelude::wasm_bindgen;
use serde::Serialize;
use serde_json::to_string;
use crate::tokenizer::line::Line;
use crate::tokenizer::token::{Token, TokenType};
use crate::tokenizer::tokenizer::readTokens;
// =================================================================================================

lazy_static!
{
  /// Множество встроенных процедур и функций
  ///
  /// todo Должно автоматически собираться из парсера
  static ref Builtins: HashSet<&'static str> = {
    HashSet::from([
      "println", "print", "clear", "go", "sleep", "exit",
      "type", "mut", "randUInt", "len", "input", "exec", "execs"
    ])
  };
}

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
  let mut buffer: Vec<u8> = code.as_bytes().to_vec();
  // Добавляем перевод строки, если его нет в конце
  if buffer.last() != Some(&b'\n') {
    buffer.push(b'\n');
  }
  
  //
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

/// Обход списка токенов, включая их внутренние линии
fn flattenTokens(tokens: &[Token], out: &mut Vec<AnalyzeToken>) 
{
  for token in tokens 
  {
    // Определяем kind, возможно заменяя Word на BuiltinProcedure
    let mut kind = token.getDataType().to_string();
    if token.getDataType() == &TokenType::Word 
    {
      if let Some(data) = token.getData().toString() 
      {
        if Builtins.contains(data.as_str()) 
        {
          kind = String::from("Builtin");
        }
        //
      }
    }
    
    // Добавляем сам токен
    #[cfg(feature = "analyzer")]
    out.push(AnalyzeToken {
      start: token.start,
      end: token.end,
      kind
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