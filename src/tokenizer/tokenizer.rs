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
fn pushLineToken(token: &mut Token, lineTokens: &mut Vec<Token>, start: usize, end: usize) 
{
  token.start = start;
  token.end = end;
  lineTokens.push(token.clone());
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
            pushLineToken(&mut token, &mut lineTokens, start, index);
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
            pushLineToken(&mut token, &mut lineTokens, start, index);
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
            pushLineToken(&mut token, &mut lineTokens, start, index);
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
            let tokenType: TokenType = 
              match byte 
              {
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
              pushLineToken(&mut token, &mut lineTokens, startPos, index);
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
            pushLineToken(&mut token, &mut lineTokens, start, index);
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

#[cfg(test)]
mod testsReadTokens
{
  use std::sync::{Arc, RwLock, RwLockReadGuard};
  use crate::tokenizer::types::line::Line;
  use crate::tokenizer::types::token::Token;
  use crate::tokenizer::types::tokenType::TokenType;
  use super::readTokens;
  // ===============================================================================================

  /// todo desk
  #[test]
  fn emptyBuffer() -> ()
  {
    let buffer: Vec<u8> = vec![];
    let result: Vec<Arc<RwLock<Line>>> = readTokens(buffer, false);

    //
    assert_eq!(result.len(), 1, "Пустой буфер даёт 1 разделитель");

    //
    let guard: RwLockReadGuard<Line> = result[0].read().unwrap();
    assert!(guard.tokens.is_none(), "Токенов быть не должно");
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn autoNewline() -> ()
  {
    let buffer: Vec<u8> = b"a".to_vec();
    let result: Vec<Arc<RwLock<Line>>> = readTokens(buffer, false);

    //
    assert_eq!(result.len(), 1, "Автодобавление \\n не ломает структуру");

    //
    let guard: RwLockReadGuard<Line> = result[0].read().unwrap();
    let tokens: &Vec<Token> = guard.tokens.as_ref().expect("Ожидается токен");
    assert_eq!(tokens.len(), 1);
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn indentHierarchy() -> () 
  {
    let buffer: Vec<u8> = b"a\n  b\n    c\n".to_vec();
    let result: Vec<Arc<RwLock<Line>>> = readTokens(buffer, false);

    //
    #[cfg(not(feature = "analyzer"))]
    assert_eq!(result.len(), 1, "1 корень");
    #[cfg(feature = "analyzer")]
    assert_eq!(result.len(), 3, "3 корневые линии (вложения не выполняются)");

    //
    let rootGuard: RwLockReadGuard<Line> = result[0].read().unwrap();
    #[cfg(not(feature = "analyzer"))]
    let children: &Vec<Arc<RwLock<Line>>> = rootGuard.lines.as_ref().expect("Вложенные линии");
    #[cfg(not(feature = "analyzer"))]
    assert_eq!(children.len(), 1, "1 дочерняя линия");

    //
    #[cfg(not(feature = "analyzer"))]
    let childGuard: RwLockReadGuard<Line> = children[0].read().unwrap();
    #[cfg(not(feature = "analyzer"))]
    assert!(childGuard.lines.is_some(), "Уровень вложенности 2");

    //
    #[cfg(not(feature = "analyzer"))]
    let inner: RwLockReadGuard<Line> = childGuard.lines.as_ref().unwrap()[0].read().unwrap();
    #[cfg(not(feature = "analyzer"))]
    assert!(inner.parent.is_some(), "Ссылка .parent существует");
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn indentReset() -> () 
  {
    let buffer: Vec<u8> = b"a\n  b\nc\n  d\n".to_vec();
    let result: Vec<Arc<RwLock<Line>>> = readTokens(buffer, false);

    //
    #[cfg(not(feature = "analyzer"))]
    assert_eq!(result.len(), 2, "2 корня после сброса отступа");
    #[cfg(feature = "analyzer")]
    assert_eq!(result.len(), 4, "4 корневые линии (вложения не выполняются)");

    //
    #[cfg(not(feature = "analyzer"))] {
      let guard1: RwLockReadGuard<Line> = result[0].read().unwrap();
      let guard2: RwLockReadGuard<Line> = result[1].read().unwrap();
      assert_eq!(guard1.lines.as_ref().unwrap().len(), 1);
      assert_eq!(guard2.lines.as_ref().unwrap().len(), 1);
    }
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn bracketInLine() -> ()
  {
    let buffer: Vec<u8> = b"(x + y)\n".to_vec();
    let result: Vec<Arc<RwLock<Line>>> = readTokens(buffer, false);

    //
    let lineGuard: RwLockReadGuard<Line> = result[0].read().unwrap();
    let tokens: &Vec<Token> = lineGuard.tokens.as_ref().expect("Токены линии");

    //
    let tokenTypeStr: String = tokens[0].getDataType().to_string();
    assert_eq!(tokenTypeStr, TokenType::CircleBracketBegin.to_string(), "Ожидалась открывающая скобка");

    //
    let innerLines: &Vec<Line> = tokens[0].lines.as_ref().expect("Вложение скобок");
    assert_eq!(innerLines.len(), 1);

    //
    let innerTokens: &Vec<Token> = innerLines[0].tokens.as_ref().expect("Токены внутри");
    assert_eq!(innerTokens.len(), 3);

    //
    let firstToken: String = innerTokens[0].getData().toString().unwrap_or_default();
    assert_eq!(firstToken, "x");
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn commentRemoval() -> ()
  {
    // todo Хороший пример, тут табуляция ниже станет и `= 20` видно будет.
    //  Также вроде как есть закрытие комментариев? что-то вроде ;
    /*
# comment
 comment
   comment
a -> UInt
  println(10) # comment
   comment
      comment
    comment
	= 20

println(a())
    */
    
    //let buffer: Vec<u8> =
    //let result: Vec<Arc<RwLock<Line>>> = readTokens(buffer, false);

    #[cfg(not(feature = "analyzer"))]
    {
      //
    }

    #[cfg(feature = "analyzer")]
    {
      // Analyzer сохраняет комментарии
    }
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn fullCommentLine() -> ()
  {
    let buffer: Vec<u8> = b"# only comment\n".to_vec();
    let result: Vec<Arc<RwLock<Line>>> = readTokens(buffer, false);

    //
    #[cfg(feature = "analyzer")]
    assert_eq!(result.len(), 1, "1 разделитель");

    //
    #[cfg(not(feature = "analyzer"))]
    assert_eq!(result.len(), 0, "Пустой результат");
    #[cfg(feature = "analyzer")]
    {
      let guard: RwLockReadGuard<Line> = result[0].read().unwrap();
      assert!(guard.tokens.is_some(), "Линия сохранена");
    }
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn complexBlock() -> ()
  {
    let buffer: Vec<u8> = b"a\n  10\ntype(a)\n# test comment\nmut(a)".to_vec();
    let result: Vec<Arc<RwLock<Line>>> = readTokens(buffer, false);

    //
    #[cfg(not(feature = "analyzer"))]
    assert_eq!(result.len(), 3); // a, type(a), mut(a)
    #[cfg(feature = "analyzer")]
    assert_eq!(result.len(), 5); // a, 10, type(a), # comment, mut(a)

    //
    #[cfg(not(feature = "analyzer"))]
    {
      //
      let rootGuard: RwLockReadGuard<Line> = result[0].read().unwrap();
      let body: &Vec<Arc<RwLock<Line>>> = rootGuard.lines.as_ref().expect("Тело блока");
      assert_eq!(body.len(), 1, "Комментарий удалён, 1 линия"); // 10

      //
      let firstGuard: RwLockReadGuard<Line> = body[0].read().unwrap();
      let firstTokens: &Vec<Token> = firstGuard.tokens.as_ref().expect("Токены линии");
      assert_eq!(firstTokens[0].getDataType().to_string(), TokenType::UInt.to_string());
    }

    //
    #[cfg(feature = "analyzer")]
    {
      // При анализаторе линии плоские (нет вложенности)
      // Вторая корневая линия (индекс 1) — это "10"
      let secondGuard: RwLockReadGuard<Line> = result[1].read().unwrap();
      let tokens: &Vec<Token> = secondGuard.tokens.as_ref().expect("Токены линии");
      assert_eq!(tokens[0].getDataType().to_string(), TokenType::UInt.to_string());

      // Проверяем остальные линии для уверенности
      let thirdGuard: RwLockReadGuard<Line> = result[2].read().unwrap();
      let thirdTokens: &Vec<Token> = thirdGuard.tokens.as_ref().unwrap();
      assert_eq!(thirdTokens[0].getDataType().to_string(), TokenType::Word.to_string());
      assert_eq!(thirdTokens[0].getData().toString().unwrap(), "type");

      let fourthGuard: RwLockReadGuard<Line> = result[3].read().unwrap();
      let fourthTokens: &Vec<Token> = fourthGuard.tokens.as_ref().unwrap();
      assert_eq!(fourthTokens[0].getDataType().to_string(), TokenType::Comment.to_string());

      let fifthGuard: RwLockReadGuard<Line> = result[4].read().unwrap();
      let fifthTokens: &Vec<Token> = fifthGuard.tokens.as_ref().unwrap();
      assert_eq!(fifthTokens[0].getDataType().to_string(), TokenType::Word.to_string());
      assert_eq!(fifthTokens[0].getData().toString().unwrap(), "mut");
    }
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn nestedBracketsWithCommas() -> ()
  {
    let buffer: Vec<u8> = b"((a), (b))\n".to_vec();
    let result: Vec<Arc<RwLock<Line>>> = readTokens(buffer, false);

    //
    let lineGuard: RwLockReadGuard<Line> = result[0].read().unwrap();
    let tokens: &Vec<Token> = lineGuard.tokens.as_ref().expect("Токены линии");

    //
    assert_eq!(tokens[0].getDataType().to_string(), TokenType::CircleBracketBegin.to_string());

    //
    let innerLines: &Vec<Line> = tokens[0].lines.as_ref().expect("Вложенные линии");
    assert_eq!(innerLines.len(), 2, "Две линии через запятую");
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn semicolonEndline() -> () {
    let buffer: Vec<u8> = b"x; y;".to_vec();
    let result: Vec<Arc<RwLock<Line>>> = readTokens(buffer, false);

    #[cfg(not(feature = "analyzer"))]
    assert_eq!(result.len(), 2, "2 линии через ;");
    #[cfg(feature = "analyzer")]
    assert_eq!(result.len(), 3, "3 линии (последняя пустая из-за завершающего \\n)");
  }

  // ===============================================================================================
}

// =================================================================================================