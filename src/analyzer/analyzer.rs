use std::collections::HashSet;
use std::sync::{RwLock, RwLockReadGuard};
use std::sync::Arc;
use lazy_static::lazy_static;
use wasm_bindgen::prelude::wasm_bindgen;
use serde::Serialize;
use serde_json::to_string;
use crate::tokenizer::tokenizer::readTokens;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::{Token, TokenType};
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

/// Выходной токен
#[derive(Serialize, Clone)]
pub struct AnalyzeToken 
{
  pub start: usize,
  pub end: usize,
  pub kind: String,
}

/// Выходная линия
#[derive(Serialize)]
pub struct AnalyzedLine 
{
  pub indent: usize,
  pub tokens: Vec<AnalyzeToken>,
}

// =================================================================================================

#[wasm_bindgen]
pub fn analyzeLines(code: &str) -> String 
{
  let buffer: Vec<u8> = code.as_bytes().to_vec();
  let lines: Vec< Arc<RwLock<Line>> > = readTokens(buffer, false);
  let mut result: Vec<AnalyzedLine> = Vec::new();
  collectLines(&lines, &mut result);
  to_string(&result).unwrap_or_else(|_| "[]".to_string())
}

fn collectLines(lines: &[Arc<RwLock<Line>>], out: &mut Vec<AnalyzedLine>)
{
  for linLink in lines
  {
    let line: RwLockReadGuard<Line> = linLink.read().unwrap();
    let indent: usize = line.indent.unwrap_or(0);
    let mut tokens: Vec<AnalyzeToken> = Vec::new();
    if let Some(lineTokens) = &line.tokens {
      flattenTokensTo(lineTokens, &mut tokens);
    }
    out.push(AnalyzedLine { indent, tokens });

    // recursively process nested lines (indented blocks)
    if let Some(nested) = &line.lines {
      collectLines(nested, out);
    }
  }
}

fn flattenTokensTo(tokens: &[Token], out: &mut Vec<AnalyzeToken>) 
{
  for token in tokens 
  {
    let mut kind: String = token.getDataType().to_string();
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
    out.push(AnalyzeToken {
      start: token.start,
      end: token.end,
      kind,
    });
    if let Some(lines) = &token.lines 
    {
      for line in lines 
      {
        if let Some(toks) = &line.tokens 
        {
          flattenTokensTo(toks, out);
        }
      }
      //
    }
  }
  //
}

// =================================================================================================