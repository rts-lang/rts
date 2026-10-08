use std::collections::HashSet;
use std::sync::{RwLock, RwLockReadGuard};
use std::sync::Arc;
use wasm_bindgen::prelude::wasm_bindgen;
use serde::Serialize;
use serde_json::to_string;
use crate::tokenizer::tokenizer::readTokensSimple;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Множество встроенных процедур и функций
///
/// todo Должно автоматически собираться из парсера
fn builtins() -> HashSet<&'static str>
{
  HashSet::from([
    "println", "print", "go", "exit",
    "type", "stype", "utype", "mut", "len"
  ])
}

// =================================================================================================

// todo issue #67 (возможно не все убирать)
/// Выходной токен (плоский формат — tools/syntax-analyzer.ts)
#[derive(Serialize, Clone)]
pub struct AnalyzeToken
{
  // todo desc
  pub start: usize,
  
  // todo desc
  pub end: usize,
  
  // todo desc
  pub kind: String
}

// todo issue #67 (возможно не все убирать)
/// Выходная линия (плоский формат — tools/syntax-analyzer.ts)
#[derive(Serialize)]
pub struct AnalyzedLine
{
  // todo desc
  pub tokens: Vec<AnalyzeToken>
}

// =================================================================================================

// todo issue #67
// todo desc
/// Плоский AST JSON — API для tools/syntax-analyzer.ts (форму не менять)
#[wasm_bindgen]
pub fn analyzeLines(code: &str) -> String
{
  let mut buffer: Vec<u8> = code.as_bytes().to_vec();
  let lines: Vec< Arc<RwLock<Line>> > = readTokensSimple(&mut buffer);
  let mut result: Vec<AnalyzedLine> = Vec::new();
  collectLines(&lines, &mut result);
  to_string(&result).unwrap_or_else(|_| "[]".to_string())
}

// todo desc
fn collectLines(lines: &[Arc<RwLock<Line>>], out: &mut Vec<AnalyzedLine>) -> ()
{
  for linLink in lines
  {
    let line: RwLockReadGuard<Line> = linLink.read().unwrap();
    let mut tokens: Vec<AnalyzeToken> = Vec::new();
    if let Some(lineTokens) = &line.tokens {
      flattenTokensTo(lineTokens, &mut tokens);
    }
    out.push(AnalyzedLine { tokens });

    // recursively process nested lines (indented blocks)
    if let Some(nested) = &line.lines {
      collectLines(nested, out);
    }
  }
}

// todo desc
fn flattenTokensTo(tokens: &[Token], out: &mut Vec<AnalyzeToken>) -> ()
{
  let builtinsSet: HashSet<&'static str> = builtins();
  for token in tokens
  {
    let mut kind: String = token.getDataType().to_string();
    if token.getDataType() == &TokenType::Word
    {
      if let Some(data) = token.getData().toString()
      {
        if builtinsSet.contains(data.as_str())
        {
          kind = String::from("Builtin");
        }
      }
    }
    
    out.push(AnalyzeToken {
      start: token.start,
      end: token.end,
      kind,
    });
    
    if let Some(nestedLines) = &token.lines
    {
      for lineLink in nestedLines
      {
        let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
        if let Some(toks) = &line.tokens
        {
          flattenTokensTo(toks, out);
        }
      }
    }
    //
  }
}

// =================================================================================================

/// Токен с вложениями — для tools/ast.ts (дерево как в старом debug)
///
/// todo rewrite desc + desc для параметров
#[derive(Serialize, Clone)]
pub struct TreeToken
{
  /// todo desc
  pub start: usize,

  /// todo desc
  pub end: usize,

  /// todo desc
  pub kind: String,

  /// todo desc
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<String>,

  /// todo desc
  #[serde(skip_serializing_if = "Option::is_none")]
  pub primitive: Option<bool>,

  /// todo desc
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lines: Option<Vec<TreeLine>>
}

/// Линия с вложениями — для tools/ast.ts
///
/// todo rewrite desc + desc для параметров
#[derive(Serialize, Clone)]
pub struct TreeLine
{
  /// todo desc
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tokens: Option<Vec<TreeToken>>,
  
  /// todo desc
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lines: Option<Vec<TreeLine>>
}

/// Дерево AST JSON — API для tools/ast.ts
///
/// todo rewrite desc
#[wasm_bindgen]
pub fn analyzeLinesTree(code: &str) -> String
{
  let mut buffer: Vec<u8> = code.as_bytes().to_vec();
  let lines: Vec< Arc<RwLock<Line>> > = readTokensSimple(&mut buffer);
  let result: Vec<TreeLine> = convertTreeLines(&lines);
  to_string(&result).unwrap_or_else(|_| "[]".to_string())
}

/// todo desc + внутренние comments
fn convertTreeLines(lines: &[Arc<RwLock<Line>>]) -> Vec<TreeLine>
{
  let mut out: Vec<TreeLine> = Vec::new();
  for link in lines
  {
    let line: RwLockReadGuard<Line> = link.read().unwrap();
    let tokens: Option<Vec<TreeToken>> = line.tokens.as_ref().map(|t| convertTreeTokens(t));
    let nested: Option<Vec<TreeLine>> = line.lines.as_ref().map(|n| convertTreeLines(n));
    out.push(TreeLine { tokens, lines: nested });
  }
  out
}

/// todo desc + внутренние comments
fn convertTreeTokens(tokens: &[Token]) -> Vec<TreeToken>
{
  let builtinsSet: HashSet<&'static str> = builtins();
  let mut out: Vec<TreeToken> = Vec::new();
  for token in tokens
  {
    let dataType: &TokenType = token.getDataType();
    let mut kind: String = dataType.to_string();
    let dataOpt: Option<String> = token.getData().toString();

    if dataType == &TokenType::Word
    {
      if let Some(ref data) = dataOpt
      {
        if builtinsSet.contains(data.as_str())
        {
          kind = String::from("Builtin");
        }
      }
    }

    let isComment: bool = kind == "Comment";
    let hasUsefulData: bool = dataOpt.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
    let primitive: Option<bool> = if isComment || !hasUsefulData { Some(true) } else { None };
    let data: Option<String> = if isComment || !hasUsefulData { None } else { dataOpt };
    let nestedLines: Option<Vec<TreeLine>> = token.lines.as_ref().map(|n| convertTreeLines(n));

    out.push(TreeToken {
      start: token.start,
      end: token.end,
      kind,
      data,
      primitive,
      lines: nestedLines
    });
  }
  out
}

// =================================================================================================
