use std::sync::{Arc, RwLock, RwLockReadGuard};
use crate::logger::logger::{formatPrint, log};
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// todo issue #57
/// Выводит токен, его тип данных
pub fn outputTokens(tokens: &[Token], lineIndent: usize, indent: &usize) -> ()
{
  let lineIndentString: String = " ".repeat(lineIndent*2+1); // Отступ для линии
  let identString:      String = " ".repeat(indent*2+1);     // Отступ для вложения токенов
  
  if tokens.len() == 0 { return; }
  let tokenCount: usize = tokens.len()-1;
  let mut c: char;

  let mut tokenType: &TokenType;
  for (i, token) in tokens.iter().enumerate()
  { // Читаем все токены

    // Слева помечаем что это за токен;
    // В случае с X это завершающий токен
    c = if i == tokenCount { 'X' } else { '┃' };

    tokenType = token.getDataType(); // Тип токена
    if let Some(tokenData) = token.getData().toString()
    { // Если токен содержит данные
      match *tokenType
      { // Проверяем что за токен
        TokenType::Char | TokenType::FormattedChar =>
        { // Если токен это Char | FormattedChar
          log("parserToken",&format!(
            "{}\\b{}\\c{}\\fg(#f0f8ff)\\b'\\c{}\\c\\fg(#f0f8ff)\\b'\\c  |{}",
            lineIndentString,
            c,
            identString,
            tokenData,
            tokenType.to_string()
          ));
        }
        TokenType::String | TokenType::FormattedString =>
        { // Если токен это String | FormattedString
          log("parserToken",&format!(
            "{}\\b{}\\c{}\\fg(#f0f8ff)\\b\"\\c{}\\c\\fg(#f0f8ff)\\b\"\\c  |{}",
            lineIndentString,
            c,
            identString,
            tokenData,
            tokenType.to_string()
          ));
        }
        TokenType::RawString | TokenType::FormattedRawString =>
        { // Если токен это RawString | FormattedRawString
          log("parserToken",&format!(
            "{}\\b{}\\c{}\\fg(#f0f8ff)\\b`\\c{}\\c\\fg(#f0f8ff)\\b`\\c  |{}",
            lineIndentString,
            c,
            identString,
            tokenData,
            tokenType.to_string()
          ));
        }
        _ =>
        { // Если это обычный токен
          log("parserToken",&format!(
            "{}\\b{}\\c{}{}  |{}",
            lineIndentString,
            c,
            identString,
            tokenData,
            tokenType.to_string()
          ));
        }
        //
      }
    } else
    { // Если это токен только с типом, то выводим тип как символ
      if token.isPrimitive()
      {
        log("parserToken", &format!(
          "{}\\b{}\\c{}|{}",
          lineIndentString,
          c,
          identString,
          tokenType.to_string()
        ));
      } else
      {
        formatPrint(&format!(
          "{}\\b{}\\c{}{}\n",
          lineIndentString,
          c,
          identString,
          tokenType.to_string()
        ));
      }
      //
    }

    if let Some(lines) = &token.lines
    { // Если есть вложения у токена, то рекурсивно обрабатываем их
      for (i, lineLink) in lines.iter().enumerate()
      {
        let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
        outputTokens(&line.tokens.clone().unwrap_or_default(), lineIndent, &(indent+1));
        
        if i != lines.len()-1 {
          log("parserToken", &format!("{}\\b┃\\c", lineIndentString));
        }
      }
    }
    //
  }
}
// todo issue #57
/// Выводит информацию о линии, а также токены линии
pub fn outputLines(linesLinks: &[ Arc<RwLock<Line>> ], indent: usize) -> ()
{
  let identStr1: String = " ".repeat(indent*2);   // Это отступ для главной строки
  let identStr2: String = format!("{} ", identStr1); // Это для дочерних токенов

  let mut line: RwLockReadGuard<Line>;
  for (i, lineLink) in linesLinks.iter().enumerate()
  { // Проходи по линиям через чтение
    line = lineLink.read().unwrap();
    log("parserBegin", &format!("{} {}",identStr1,i));

    if let Some(tokens) = &line.tokens
    { // Заголовок для начала вложенных токенов
      formatPrint(&format!("{}\\b┣ \\fg(#90df91)Tokens\\c\n",identStr2));
      // todo плохо используются tokens
      outputTokens(tokens, indent, &1); // выводим вложенные токены
    } else 
    { // Заголовок для разделителей
      formatPrint(&format!("{}\\b┗ \\fg(#90df91)Separator\\c\n",identStr2));
    }

    if let Some(lineLines) = &line.lines
    { // Заголовок для начала вложенных линий
      formatPrint(&format!("{}\\b┗ \\fg(#90df91)Lines\\c\n",identStr2));
      outputLines(lineLines, indent+1); // выводим вложенные линии
    }
  }
  //
}

// =================================================================================================