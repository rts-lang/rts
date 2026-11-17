use crate::parser::structure::structure::Structure;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

/// Проверяет, является ли [`Token`] тегом `[ffi]`.
/// 
/// Токен должен иметь тип [`TokenType::SquareBracketBegin`] и
/// хранить одну вложенную линию с токеном `ffi` типа [`TokenType::Word`];
/// 
/// Возвращаем `true`, если это так.
//
// todo Когда появятся другие теги (`async`, `thread` и т.д.) — расширить.
pub fn isFfiTagToken(token: &Token) -> bool
{
  // todo desc
  if *token.getDataType() != TokenType::SquareBracketBegin {
    return false;
  }

  // todo desc
  let lines: &Vec< Arc<RwLock<Line>> > = if let Some(lines) =
    token.lines.as_ref() { lines }
    else { return false };

  // todo desc
  for lineLink in lines
  {
    // todo desc
    let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
    let tokens: &[Token] = if let Some(tokens) =
      line.tokens.as_ref() { tokens }
      else { continue };

    // todo desc
    for token in tokens
    {
      if *token.getDataType() == TokenType::Word &&
        token.getData().toString().unwrap_or_default() == "ffi"
      {
        return true;
      }
    }
  }
  
  //
  false
}

/// Проверяет, является ли предыдущая [`Line`] тегом `[ffi]`.
/// 
/// Используется для многострочной формы:
/// ```ignore
/// [ffi]
/// {
///   ...
/// }
/// ```
/// 
/// Здесь вторая [`Line`] сама по себе не знает о теге сверху.
pub fn isPrevLineFfiTag(parentLink: &Arc<RwLock<Structure>>, lineIndex: usize) -> bool
{
  // Если до этой `Line` нет других - то ничего не будет.
  if lineIndex == 0 {
    return false;
  }

  // Получаем предыдущую `Line`.
  let prevLineLinkOption: Option< Arc<RwLock<Line>> > = parentLink.read().unwrap()
    .lines.as_ref()
    .and_then(|lines| lines.get(lineIndex-1).cloned());
  
  let prevLineLink: Arc<RwLock<Line>> = match prevLineLinkOption {
    Some(line) => line,
    None => return false,
  };

  // Получаем токены предыдущей `Line`.
  let prevLine: RwLockReadGuard<Line> = prevLineLink.read().unwrap();
  let prevLineTokens: &[Token] = match prevLine.tokens.as_ref() {
    Some(tokens) => tokens,
    None => return false,
  };
  
  // Предыдущая `Line` должна иметь ровно один токен: `[ffi]`.
  if prevLineTokens.len() != 1 {
    return false;
  }
  
  // Проверяет, является ли токен тегом `[ffi]`.
  isFfiTagToken(&prevLineTokens[0])
}

// =================================================================================================