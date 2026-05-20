use std::{
  sync::{Arc, RwLock}
};
#[cfg(all(not(target_family = "wasm"), not(test)))]
use std::time::{Instant, Duration};
#[cfg(not(target_family = "wasm"))]
use crate::logger::logger::{log, logSeparator};
#[cfg(not(feature = "analyzer"))]
use crate::tokenizer::read::primitives::comments::{deleteComment};
#[cfg(feature = "analyzer")]
use crate::tokenizer::read::primitives::comments::{deleteComments}; 
use crate::tokenizer::read::primitives::numbers::{getNumber, isDigit};
use crate::tokenizer::read::primitives::operators::{getOperator, isSingleChar};
use crate::tokenizer::read::primitives::quotes::getQuotes;
use crate::tokenizer::read::primitives::words::{getWord, isLetter};
use crate::tokenizer::read::nesting::brackets::{bracketNesting};
use crate::tokenizer::read::nesting::comments::deleteNestedComment;
#[cfg(not(feature = "analyzer"))]
use crate::tokenizer::read::nesting::lines::lineNesting;
#[cfg(not(target_family = "wasm"))]
#[cfg(not(test))]
use crate::tokenizer::tools::output::outputLines;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Вспомогательный макрос для добавления токенов start/end
#[cfg(feature = "analyzer")]
macro_rules! pushLineToken 
{
  ($token:expr, $lineTokens:expr, $start:expr, $end:expr) => {
    {
      $token.start = $start;
      $token.end = $end;
      $lineTokens.push($token);
    }
  };
}

/// Основная функция для чтения токенов и получения чистых линий из них;
/// Токены в этот момент не только сгруппированы в линии, но и имеют
/// предварительные базовые типы данных
pub fn readTokens(buffer: Vec<u8>, debugMode: bool) -> Vec< Arc<RwLock<Line>> >
{
  // Требуем обязательно \n в конце для правильного чтения;
  // Получаем buffer без mut.
  let buffer: Vec<u8> = 
    if buffer.last() == Some(&b'\n') 
    { // Если есть, значит оставляем старый
      buffer
    } else 
    { // Если нет, получаем новый
      let mut newBuffer: Vec<u8> = buffer.clone();
      newBuffer.push(b'\n');
      newBuffer
    };
  
  //
  #[cfg(not(target_family = "wasm"))]
  #[cfg(not(test))]
  match debugMode
  {
    true =>
    {
      logSeparator("AST");
      log("ok","+Generation");
      println!("     ┃");
    }
    false => {}
  }
  #[cfg(not(target_family = "wasm"))]
  #[cfg(not(test))]
  let startTime: Instant = Instant::now(); // Замеряем текущее время для debug

  let mut      index: usize = 0;               // Основной индекс чтения
  let   bufferLength: usize = buffer.len();    // Размер буфера байтов
  let mut lineIndent: usize = 0;               // Текущий отступ линии
  let mut lineTokens: Vec<Token> = Vec::new(); // Прочитанные токены текущей линии

  let mut linesLinks:     Vec< Arc<RwLock<Line>> > = Vec::new(); // Ссылки на готовые линии
  let mut readLineIndent: bool                     = true;       // Флаг на проверку есть ли indent сейчас

  let mut byte: u8;
  while index < bufferLength
  { // Читаем байты
    byte = buffer[index]; // Текущий байт

    // Проверяем отступы, они могут быть указаны пробелами,
    // либо readLineIndent будет true после конца строки предыдущей линии
    match byte == b' ' && readLineIndent
    {
      true =>
      {
        index += 1;
        lineIndent += 1;
      }
      false =>
      {
        #[cfg(feature = "analyzer")]
        let start: usize = index; // Начало токена
        readLineIndent = false;
        
        // Смотрим, является ли это endline
        if byte == b'\n' || byte == b';'
        { // Если это действительно конец строки,
          // то вкладываем возможные скобки
          bracketNesting(
            &mut lineTokens,
            &TokenType::CircleBracketBegin,
            &TokenType::CircleBracketEnd
          );
          /* todo Вырезано, пока не используется
          bracketNesting(
            &mut lineTokens,
            &TokenType::SquareBracketBegin,
            &TokenType::SquareBracketEnd
          );
          */
          // todo FigureBracketBegin и FigureBracketEnd ?
          // это остаётся всё ещё здесь только потому,
          // что может быть нужным для реализации использования
          // подобных структур:
          /*
            for x(args: <Token>) -> None
              args[0]
              ? args[1]
                {}
                args[2]
                go(1)

            for i = 0, i < 10, i++
              println(10)
          */
          // здесь наглядно видно, что for функция будет запущена
          // только когда дойдёт до самого конца вложения,
          // после чего {} позволит запустить всё вложение.
          // а при необходимости мы бы могли обращаться к вложению,
          // например: {}.0 или {}[0] ...
          // поэтому эта тема требует отдельных тестов.
          /*
          bracketNesting(
            &mut lineTokens,
            &TokenType::FigureBracketBegin,
            &TokenType::FigureBracketEnd
          );
          */

          // Добавляем новую линию и пушим ссылку на неё
          let lineTokens: Vec<Token> = std::mem::take(&mut lineTokens); // Пустой вектор для следующей
          linesLinks.push(
            Arc::new(RwLock::new(
              Line
              {
                tokens:
                  match lineTokens.is_empty()
                  { // Забираем все токены в линию
                    true => None,
                    false => Some(lineTokens)
                  },
                indent: Some(lineIndent),
                lines:  None, // В данный момент у неё нет вложенных линий, будет чуть ниже
                parent: None  // Также у неё нет родителя, это тоже будет ниже при вложении
              }
            ))
          );
          lineIndent = 0;

          readLineIndent = true; // Это был конец строки
          index += 1;
        } else
        if byte == b'#'
        { // Ставим метку на комментарий в линии, по ним потом будут удалены линии
          #[cfg(feature = "analyzer")]
          deleteComments(&buffer, &mut index, &bufferLength, &lineIndent); // Пропускает комментарии
          #[cfg(not(feature = "analyzer"))]
          deleteComment(&buffer, &mut index, &bufferLength); // Пропускает комментарий
          //
          #[cfg(feature = "analyzer")]
          {
            let mut token: Token = Token::newEmpty(TokenType::Comment);
            pushLineToken!(token, lineTokens, start, index);
          }
          #[cfg(not(feature = "analyzer"))]
          {
            let token: Token = Token::newEmpty(TokenType::Comment);
            lineTokens.push(token);
          }
        } else
        if isDigit(&byte) || (byte == b'-' && index+1 < bufferLength && isDigit(&buffer[index+1]))
        { // Получаем все возможные численные примитивные типы данных
          #[cfg(feature = "analyzer")]
          {
            let mut token: Token = getNumber(&buffer, &mut index, &bufferLength);
            pushLineToken!(token, lineTokens, start, index);
          }
          #[cfg(not(feature = "analyzer"))]
          {
            let token: Token = getNumber(&buffer, &mut index, &bufferLength);
            lineTokens.push(token);
          }
        } else
        if isLetter(&byte)
        { // Получаем все возможные и зарезервированные слова
          //
          #[cfg(feature = "analyzer")]
          {
            let mut token: Token = getWord(&buffer, &mut index, &bufferLength);
            pushLineToken!(token, lineTokens, start, index);
          }
          #[cfg(not(feature = "analyzer"))]
          {
            let token: Token = getWord(&buffer, &mut index, &bufferLength);
            lineTokens.push(token);
          }
        } else
        if matches!(byte, b'\'' | b'"' | b'`') {
          // Проверяем, есть ли перед кавычкой токен `f`
          let isFormatted: bool = !lineTokens.is_empty()
            && lineTokens.last().unwrap().getDataType() == &TokenType::Word
            && lineTokens.last().unwrap().getData().toString().unwrap_or_default() == "f";

          let startPos: usize = index; // Начало кавычки (для обычного токена)

          if isFormatted 
          {
            // Удаляем токен `f`
            let fToken: Token = lineTokens.pop().unwrap();
            #[cfg(feature = "analyzer")]
            let startF: usize = fToken.start;

            let mut token: Token = getQuotes(&buffer, &mut index, true); // formatted = true

            // Устанавливаем тип (FormattedChar / FormattedString / FormattedRawString)
            let tokenType = match byte {
              b'\'' => TokenType::FormattedChar,
              b'"' => TokenType::FormattedString,
              b'`' => TokenType::FormattedRawString,
              _ => unreachable!(),
            };
            token.setDataType(tokenType);

            #[cfg(feature = "analyzer")]
            {
              token.start = startF;
              token.end = index;
            }
            lineTokens.push(token);
          } else 
          {
            let mut token: Token = getQuotes(&buffer, &mut index, false);
            let tokenType: TokenType = *token.getDataType();
            if tokenType != TokenType::None {
              #[cfg(feature = "analyzer")]
              pushLineToken!(token, lineTokens, startPos, index);
              #[cfg(not(feature = "analyzer"))]
              lineTokens.push(token);
            } else {
              index += 1;
            }
          }
        } else
        // Получаем возможные двойные и одиночные символы
        if isSingleChar(&byte)
        {
          //
          #[cfg(feature = "analyzer")]
          {
            let mut token: Token = getOperator(&buffer, &mut index, &bufferLength);
            pushLineToken!(token, lineTokens, start, index);
          }
          #[cfg(not(feature = "analyzer"))]
          {
            let token: Token = getOperator(&buffer, &mut index, &bufferLength);
            lineTokens.push(token);
          }
        } else
        { // Если мы ничего не нашли из возможного, значит этого нет в синтаксисе;
          // Поэтому просто идём дальше
          index += 1;
        }
      }
      //
    }
  }

  // Вкладываем линии
  #[cfg(not(feature = "analyzer"))]
  lineNesting(&mut linesLinks);
  // Удаляем возможные вложенные комментарии по меткам
  deleteNestedComment(&mut linesLinks, 0);

  // debug output and return
  #[cfg(not(target_family = "wasm"))]
  #[cfg(not(test))]
  match debugMode
  { false => {} true =>
  {
    let endTime:  Instant  = Instant::now();    // Получаем текущее время
    let duration: Duration = endTime-startTime; // Получаем сколько всего прошло
    outputLines(&linesLinks,&2); // Выводим полученное AST дерево из линий
    //
    println!("     ┃");
    log("ok",&format!("xDuration: {:?}",duration));
  }}
  // Возвращаем готовые ссылки на линии
  linesLinks
}

// =================================================================================================