/* /tokenizer
*/

pub mod token;
pub mod line;

use crate::{
  logger::*,
  tokenizer::token::*,
  tokenizer::line::*
};

use std::{
  time::{Instant,Duration},
  sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}
};

// проверяет buffer по index и так пропускае возможные комментарии
// потом они будут удалены по меткам
fn deleteComment(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> ()
{
  *index += 1;
  while *index < *bufferLength && buffer[*index] != b'\n' 
  {
    *index += 1;
  }
}

// проверяет что байт является одиночным знаком;
// доступным для синтаксиса
fn isSingleChar(c: &u8) -> bool 
{
  matches!(*c, 
    b'+' | b'-' | b'*' | b'/' | b'=' | b'%' | b'^' |
    b'>' | b'<' | b'?' | b'!' | b'&' | b'|' | 
    b'(' | b')' | b'{' | b'}' | b'[' | b']' | 
    b':' | b',' | b'.' | b'~'
  )
}

// проверяет что байт является числом
fn isDigit(c: &u8) -> bool 
{
  *c >= b'0' && *c <= b'9'
}
// проверяет buffer по index и так находит возможные 
// примитивные численные типы данных;
// e: UInt, Int, UFloat, Float, Rational, Complex
// todo: Ввести Complex числа;
// todo: Ввести работу float с .1 или . как 0.0
fn getNumber(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> Token 
{
  let mut savedIndex: usize = *index; // index buffer
  let mut result: String = String::from(buffer[savedIndex] as char);
  savedIndex += 1;

  let mut      dot: bool = false; // dot check
  let mut negative: bool = false; // negative check
  let mut rational: bool = false; // reational check

  let mut byte1: u8; // текущий символ
  let mut byte2: u8; // следующий символ
  while savedIndex < *bufferLength 
  {
    byte1 = buffer[savedIndex]; // значение текущего символа
    byte2 =                     // значение следующего символа
      match savedIndex+1 < *bufferLength 
      {
        true  => { buffer[savedIndex+1] }  
        false => { b'\0' }
      };

    // todo: use match case
    if !negative && buffer[*index] == b'-' 
    { // Int/Float flag
      result.push(byte1 as char);
      negative = true;
      savedIndex += 1;
    } else
    if isDigit(&byte1) 
    { // UInt
      result.push(byte1 as char);
      savedIndex += 1;
    } else 
    if byte1 == b'.' && !dot && isDigit(&byte2) &&
       savedIndex > 1 && buffer[*index-1] != b'.' // fixed for a.0.1
    { // UFloat
      match rational 
      {
        true => { break; }
        false => {}
      }
      dot = true;
      result.push(byte1 as char);
      savedIndex += 1;
    } else
    if byte1 == b'/' && byte2 == b'/' && !dot && 
       (savedIndex+2 < *bufferLength && isDigit(&buffer[savedIndex+2])) 
    { // Rational
      rational = true;
      result.push_str("//");
      savedIndex += 2;
    } else 
    {
      break;
    }
  }

  *index = savedIndex;
  // next return
  match (rational, dot, negative) 
  { //   rational,  dot,  negative
    (true, _, _)     => Token::new( Some(TokenType::Rational), Some(result) ),
    (_, true, true)  => Token::new( Some(TokenType::Float),    Some(result) ),
    (_, true, false) => Token::new( Some(TokenType::UFloat),   Some(result) ),
    (_, false, true) => Token::new( Some(TokenType::Int),      Some(result) ),
    _                => Token::new( Some(TokenType::UInt),     Some(result) ),
  }
}

// проверяет что байт является буквой a-z A-Z
fn isLetter(c: &u8) -> bool 
{
  (c|32)>=b'a'&&(c|32)<=b'z'
}
// проверяет buffer по index и так находит возможные слова;
// из них также выделяет сразу определяемые зарезервированные
fn getWord(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> Token 
{
  let mut savedIndex: usize = *index; // index buffer
  let mut result: String = String::from(buffer[savedIndex] as char);
  savedIndex += 1;
  let mut isLink: bool = false;

  let mut byte1: u8; // текущий символ
  while savedIndex < *bufferLength 
  {
    byte1 = buffer[savedIndex]; // значение текущего символа

    // todo: use match case
    if (isDigit(&byte1) || byte1 == b'.') || // Либо число, либо . как ссылка
       (isLink && byte1 == b'[' || byte1 == b']') // В случае ссылки мы можем читать динамические []
    {
      result.push(byte1 as char);
      savedIndex += 1;
      match byte1 == b'.' 
      { // только если есть . то мы знаем что это ссылка
        true => { isLink = true; }
        false => {}
      } 
    } else 
    if isLetter(&byte1)
    {
      result.push(byte1 as char);
      savedIndex += 1;
    } else 
    {
      break;
    }
  }

  *index = savedIndex;
  // next return
  match isLink 
  {
    true => {
      Token::new( Some(TokenType::Link), Some(result.clone()) )
    }  
    false => {
      match result.as_str()
      {
        "true"     => Token::new( Some(TokenType::Bool), Some(String::from("1")) ),
        "false"    => Token::new( Some(TokenType::Bool), Some(String::from("0")) ),
        _          => Token::new( Some(TokenType::Word), Some(result) ),
      }
    }
  }
}

// проверяет buffer по index и так находит возможные 
// Char, String, RawString
fn getQuotes(buffer: &[u8], index: &mut usize) -> Token 
{
  let byte1: u8 = buffer[*index]; // начальный символ кавычки
  let mut result: String = String::new();

  *index += 1;

  let length: usize = buffer.len();
  let mut byte2: u8;

  let mut backslashCount: usize;
  let mut i: usize;
  while *index < length 
  {
    byte2 = buffer[*index]; // текущий байт
    // Ошибка: конец строки внутри кавычек
    match byte2 
    {
      // Возврат строки не возможен, поскольку она может выйти за скобки и т.п. 
      // если он достиг конца строки уже;
      b'\n' => { return Token::newEmpty(None); }
      // Если мы нашли символ похожий на первый, значит закрываем,
      // но возможно это экранированная кавычка, и не закрываем.
      byte if byte == byte1 =>
      { // Проверка обратных слэшей перед закрывающей кавычкой
        backslashCount = 0;
        i = *index-1;

        while i > 0 && buffer[i] == b'\\' 
        {
          backslashCount += 1;
          i -= 1;
        }

        // Нечетное количество обратных слэшей — кавычка экранирована
        match backslashCount%2 
        {
          1 => result.push(byte2 as char), // экранированная кавычка
          _ => 
          {
            *index += 1; // завершение строки
            break;
          }
        }
      }
      // Все иные символы, входящие между кавычек;
      _ => { result.push(byte2 as char); }
    }

    *index += 1;
  }

  // Проверяем тип кавычки и возвращаем соответствующий токен
  match byte1 
  {
    b'\'' => 
    { // Одинарные кавычки должны содержать только один символ
      match result.len() 
      {
        1 => { Token::new(Some(TokenType::Char), Some(result)) }
        _ => { Token::newEmpty(None) }
      } 
    }
    b'"' => Token::new(Some(TokenType::String), Some(result)),
    b'`' => Token::new(Some(TokenType::RawString), Some(result)),
    _ => Token::newEmpty(None),
  }
}

// проверяет buffer по index и так находит возможные 
// двойные и одиночные операторы
fn getOperator(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> Token 
{
  let currentByte: u8 = buffer[*index]; // current byte
  let nextByte: u8 =                    // next byte or \0
    match *index+1 < *bufferLength 
    {
      true  => { buffer[*index+1] } 
      false => { b'\0'}
    };

  let mut increment = |count: usize| 
  { // index increment for single & duble operators
    *index += count;
  };

  match currentByte 
  {
    b'+' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty( Some(TokenType::PlusEquals) ) }
        b'+' => { increment(2); Token::newEmpty( Some(TokenType::UnaryPlus) ) }
        _    => { increment(1); Token::newEmpty( Some(TokenType::Plus) ) }
      }
    }
    b'-' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty( Some(TokenType::MinusEquals) ) }
        b'-' => { increment(2); Token::newEmpty( Some(TokenType::UnaryMinus) ) }
        b'>' => { increment(2); Token::newEmpty( Some(TokenType::Pointer) ) }
        _ =>    { increment(1); Token::newEmpty( Some(TokenType::Minus) ) }
      }
    }
    b'*' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty( Some(TokenType::MultiplyEquals) ) }
        b'*' => { increment(2); Token::newEmpty( Some(TokenType::UnaryMultiply) ) }
        _ =>    { increment(1); Token::newEmpty( Some(TokenType::Multiply) ) }
      }
    }
    b'/' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty( Some(TokenType::DivideEquals) ) }
        b'/' => { increment(2); Token::newEmpty( Some(TokenType::UnaryDivide) ) }
        _ =>    { increment(1); Token::newEmpty( Some(TokenType::Divide) ) }
      }
    }
    b'%' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty( Some(TokenType::Modulo) ) } // todo: add new type in Token
        b'%' => { increment(2); Token::newEmpty( Some(TokenType::Modulo) ) } // todo: add new type in Token
        _ =>    { increment(1); Token::newEmpty( Some(TokenType::Modulo) ) }
      }
    }
    b'^' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty( Some(TokenType::Exponent) ) } // todo: add new type in Token
        b'^' => { increment(2); Token::newEmpty( Some(TokenType::Exponent) ) } // todo: add new type in Token
        _ =>    { increment(1); Token::newEmpty( Some(TokenType::Disjoint) ) }
      }
    }
    b'>' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty( Some(TokenType::GreaterThanOrEquals) ) }
        _ =>    { increment(1); Token::newEmpty( Some(TokenType::GreaterThan) ) }
      }
    }
    b'<' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty( Some(TokenType::LessThanOrEquals) ) }
        _ =>    { increment(1); Token::newEmpty( Some(TokenType::LessThan) ) }
      }
    }
    b'!' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty( Some(TokenType::NotEquals) ) }
        _ =>    { increment(1); Token::newEmpty( Some(TokenType::Exclusion) ) }
      }
    }
    b'&' => { increment(1); Token::newEmpty( Some(TokenType::Joint) ) }
    b'|' => { increment(1); Token::newEmpty( Some(TokenType::Inclusion) ) }
    b'=' => { increment(1); Token::newEmpty( Some(TokenType::Equals) ) }
    // brackets
    b'(' => { increment(1); Token::newEmpty( Some(TokenType::CircleBracketBegin) ) }
    b')' => { increment(1); Token::newEmpty( Some(TokenType::CircleBracketEnd) ) }
    b'{' => { increment(1); Token::newEmpty( Some(TokenType::FigureBracketBegin) ) }
    b'}' => { increment(1); Token::newEmpty( Some(TokenType::FigureBracketEnd) ) }
    b'[' => { increment(1); Token::newEmpty( Some(TokenType::SquareBracketBegin) ) }
    b']' => { increment(1); Token::newEmpty( Some(TokenType::SquareBracketEnd) ) }
    // other
    b';' => { increment(1); Token::newEmpty( Some(TokenType::Endline) ) }
    b':' => { increment(1); Token::newEmpty( Some(TokenType::Colon) ) }
    b',' => { increment(1); Token::newEmpty( Some(TokenType::Comma) ) }
    b'.' => { increment(1); Token::newEmpty( Some(TokenType::Dot) ) }
    b'?' => { increment(1); Token::newEmpty( Some(TokenType::Question) ) }
    b'~' => { increment(1); Token::newEmpty( Some(TokenType::Tilde) ) }
    _ => Token::newEmpty( None ),
  }
}

// основная функция, которая вкладывает токены в скобки
// e: () [] {}
// от начальной скобки до закрывающей
// её особенность в рекурсивном вызове себя для дочерних токенов
fn bracketNesting(tokens: &mut Vec<Token>, beginType: &TokenType, endType: &TokenType) -> ()
{
  for token in tokens.iter_mut() 
  { // чтение токенов
    match &mut token.tokens {
      Some(tokens) => {
        // рекурсия
        bracketNesting(tokens, beginType, endType);
      }
      None => {} // ничего не делаем
    }
  }
  // вкладывание
  blockNesting(tokens, beginType, endType);
}
// эта функция является дочерней bracketNesting 
// и занимается только самим вложением токенов 
// от начальной скобки до закрывающей
fn blockNesting(tokens: &mut Vec<Token>, beginType: &TokenType, endType: &TokenType) -> ()
{
  let mut brackets: Vec::<usize> = Vec::new();   // nested brackets
  let mut tokensLength: usize    = tokens.len(); // tokens length

  let mut i: usize = 0; // index buffer
  while i < tokensLength 
  { // read tokens
    match &tokens[i].getDataType().unwrap_or_default() 
    { // compare type
      tokenType if tokenType == beginType =>
      { // if this is the first token
        brackets.push(i);
      } 
      tokenType if tokenType == endType =>
      { // if this is the last token
        if let Some(lastBracket) = brackets.pop() 
        { // then delete last bracket
          if !brackets.is_empty() 
          { // add new nesting
            let savedToken: Token = tokens[lastBracket].clone(); // last token (bracket)
            if let Some(token) = tokens.get_mut(brackets[brackets.len()-1]) 
            {
              match &mut token.tokens 
              {
                Some(tokenTokens) =>
                { // contains tokens 
                  tokenTokens.push(savedToken.clone());
                } 
                _ => 
                { // no tokens
                  token.tokens = Some( vec![savedToken.clone()] );
                }
              }
            }

            // remove last token (bracket)
            tokens.remove(lastBracket);
            tokensLength -= 1;

            match lastBracket < i 
            { 
              true  => { i -= 1; }
              false => {}
            }
          }
        }

        // remove begin token (bracket)
        tokens.remove(i);
        tokensLength -= 1;
        continue;
      } 
      _ => if !brackets.is_empty() 
      { // nesting tokens to bracket begin
        let savedToken: Token = tokens.remove(i);
        match &mut tokens.get_mut(brackets[brackets.len()-1]) 
        {
          Some(token) => 
          {
            match &mut token.tokens 
            {
              Some(tokenTokens) => 
              { // contains tokens 
                tokenTokens.push(savedToken.clone());
              }
              None => 
              { // no tokens
                token.tokens = Some( vec![savedToken.clone()] );
              }
            }
          }
          None => {}
        }

        // go to next token
        tokensLength -= 1;
        continue;
      }
    }
    i += 1; // continue
  }
}
// вкладывает линии токенов друг в друга
fn lineNesting(linesLinks: &mut Vec< Arc<RwLock<Line>> >) -> ()
{
  let mut index:     usize = 0;                // current line index
  let mut nextIndex: usize = 1;                // next    line index
  let mut length:    usize = linesLinks.len(); // all lines links length

  let mut compare: bool;
  while index < length && nextIndex < length 
  { // если мы не дошли до конца, то читаем линии
    compare = 
    {
      let currentIndent: usize = linesLinks[index].read().unwrap().indent;
      let nextIndent:    usize = linesLinks[nextIndex].read().unwrap().indent;
      currentIndent < nextIndent
    };
    match compare 
    { // compare current indent < next indent
      true => 
      {
        // get next line and remove
        let nestingLineLink: Arc<RwLock<Line>> = linesLinks.remove(nextIndex);
        length -= 1;
        { // set parent line link
          nestingLineLink.write().unwrap()
            .parent = Some( linesLinks[index].clone() );
        }
        // push nesting
        let mut currentLine: RwLockWriteGuard<'_, Line> = linesLinks[index].write().unwrap();
        match &mut currentLine.lines 
        {
          Some(lineLines) => 
          { // если вложения уже были, то просто делаем push
            lineLines.push(nestingLineLink); // nesting
            lineNesting(lineLines);          // cycle
          },
          None => 
          { // если вложения не было до этого, то создаём
            currentLine.lines = Some(vec![nestingLineLink]);  // nesting
            lineNesting(currentLine.lines.as_mut().unwrap()); // cycle
          }
        }
      }
      false => {
        index += 1;
        nextIndex = index+1;
      }
    } 
  }
}

// удаляет возможные вложенные комментарии по меткам;
// это такие комментарии, которые имеют вложения
// todo: не удаляет комментарии во вложенных блоках
fn deleteNestedComment(linesLinks: &mut Vec< Arc<RwLock<Line>> >, mut index: usize) -> ()
{
  let mut linesLinksLength: usize = linesLinks.len(); // количество ссылок строк
  let mut lastTokenIndex:   usize; // это указатель на метку где TokenType::Comment
  // это может быть либо последний токен, либо первый токен в большом комментарии;

  let mut deleteLine: bool;
  let mut line: RwLockWriteGuard<'_, Line>;

  while index < linesLinksLength 
  {
    deleteLine = false; // состояние удаления текущей линии
    'exit: 
    { // прерывание чтобы не нарушать мутабельность
      line = linesLinks[index].write().unwrap();

      match &mut line.lines
      { // рекурсивно обрабатываем вложенные линии
        Some(lineLines) => { deleteNestedComment(lineLines, 0); }
        None => {}
      }
      
      match line.tokens.is_empty()
      { 
        true => 
        { // пропускаем разделители, они нужны для синтаксиса
          // если разделитель имеет вложения
          match &line.lines 
          { 
            Some(_) => { break 'exit; } // выходим из прерывания
            None => {}
          } 

          // проверяем на скопление разделителей
          match index+1 < linesLinksLength 
          { // если есть линия ниже, то мы можем предполагать, что 
            // она может быть тоже разделителем;
            true => 
            {
              match linesLinks[index+1].write().unwrap()
                   .tokens.is_empty() 
               { // если токенов в следующей линии не было, значит точно separator;
                 // повторение подобных условий оставит 1 separator линию по итогу;
                  true  => { deleteLine = true; }
                  false => {}
               }
            }
            false => {}
          }

          // обычный разделитель
          break 'exit; // выходим из прерывания
        }
        false => {}
      }
      
      lastTokenIndex = line.tokens.len()-1;
      match line.tokens[lastTokenIndex].getDataType().unwrap_or_default()
      {
        TokenType::Comment =>  
        { // удаляем комментарии
          line.tokens.remove(lastTokenIndex);

          match
          { // проверяем если есть вложенные линии;
            // а также, что комментарий не удалится весь 
            // и продолжается на вложенные линии;
            match &line.lines 
            {
              Some(_) =>
              {
                match lastTokenIndex 
                {
                  0 => { false }
                  _ => { true }
                }
              }
              None => false,
            }
          }
          {
            true => { line.lines = None; }
            false => {}
          }

          match line.tokens.is_empty()
          { // переходим к удалению пустой линии
            true => 
            {
              deleteLine = true; // линия была удалена
              break 'exit;       // выходим из прерывания
            }
            false => {}
          }
        }
        _ => {}
      }
    }
    // когда линия удалена в прерывании, 
    // её можно спокойно удалить
    match deleteLine 
    {
      true => 
      {
        drop(line);
        linesLinks.remove(index);
        linesLinksLength -= 1;
        continue;
      }
      false => {}
    }
    // продолжаем чтение
    index += 1;
  }
}

// выводит токен, его тип данных
pub fn outputTokens(tokens: &Vec<Token>, lineIndent: &usize, indent: &usize) -> ()
{
  let lineIndentString: String = " ".repeat(lineIndent*2+1); // отступ для линии
  let identString:      String = " ".repeat(indent*2+1);     // отступ для вложения токенов

  let tokenCount: usize = tokens.len()-1;
  let mut c: char;

  let mut tokenType: TokenType;
  for (i, token) in tokens.iter().enumerate() 
  { // читаем все токены

    // слева помечаем что это за токен;
    // в случае с X это завершающий токен
    c = 
      match i == tokenCount 
      {
        true  => { 'X' }  
        false => { '┃' }
      };

    tokenType = token.getDataType().unwrap_or_default(); // тип токена
    match token.getData() 
    {
      Some(tokenData) => 
      { // если токен содержит данные
        match tokenType 
        { // проверяем что за токен
          TokenType::Char | TokenType::FormattedChar =>
          { // если токен это Char | FormattedChar
            log("parserToken",&format!(
              "{}{}{}\\fg(#f0f8ff)\\b'\\c{}\\c\\fg(#f0f8ff)\\b'\\c  |{}",
              lineIndentString,
              c,
              identString,
              tokenData,
              tokenType.to_string()
            ));
          }
          TokenType::String | TokenType::FormattedString =>
          { // если токен это String | FormattedString
            log("parserToken",&format!(
              "{}{}{}\\fg(#f0f8ff)\\b\"\\c{}\\c\\fg(#f0f8ff)\\b\"\\c  |{}",
              lineIndentString,
              c,
              identString,
              tokenData,
              tokenType.to_string()
            ));
          }
          TokenType::RawString | TokenType::FormattedRawString =>
          { // если токен это RawString | FormattedRawString
            log("parserToken",&format!(
              "{}{}{}\\fg(#f0f8ff)\\b`\\c{}\\c\\fg(#f0f8ff)\\b`\\c  |{}",
              lineIndentString,
              c,
              identString,
              tokenData,
              tokenType.to_string()
            ));
          }
          _ => 
          { // если это обычный токен
            log("parserToken",&format!(
              "{}{}{}{}  |{}",
              lineIndentString,
              c,
              identString,
              tokenData,
              tokenType.to_string()
            ));
          }
        }
      }
      _ => 
      { // если это токен только с типом, то выводим тип как символ
        formatPrint(&format!(
          "{}{}{}{}\n",
          lineIndentString,
          c,
          identString,
          tokenType.to_string()
        ));
      }
    } 

    // если есть вложения у токена, то просто рекурсивно обрабатываем их
    match &token.tokens
    {
      Some(tokens) => { outputTokens(tokens, lineIndent, &(indent+1)); }
      None => {}
    }
  }
}
// выводит информацию о линии;
// также токены линии
pub fn outputLines(linesLinks: &Vec< Arc<RwLock<Line>> >, indent: &usize) -> ()
{
  let identStr1: String = " ".repeat(indent*2);      // это отступ для главной строки
  let identStr2: String = format!("{} ", identStr1); // а это для дочерних токенов

  let mut line: RwLockReadGuard<'_, Line>;
  for (i, lineLink) in linesLinks.iter().enumerate() 
  { // проходи по линиям через чтение
    line = lineLink.read().unwrap();
    log("parserBegin", &format!("{} {}",identStr1,i));

    match (&line.tokens).len() 
    {
      0 => 
      { // заголовок для разделителей
        formatPrint(&format!("{}\\b┗ \\fg(#90df91)Separator\\c\n",identStr2));
      } 
      _ =>
      { // заголовок для начала вложенных токенов
        formatPrint(&format!("{}\\b┣ \\fg(#90df91)Tokens\\c\n",identStr2));
        outputTokens(&line.tokens, &indent, &1); // выводим вложенные токены
      }
    } 
    
    match &line.lines
    { // заголовок для начала вложенных линий
      Some(lineLines) => 
      {
        formatPrint(&format!("{}\\b┗ \\fg(#90df91)Lines\\c\n",identStr2));
        outputLines(lineLines, &(indent+1)); // выводим вложенные линии
      }
      None => {}
    }
  }
  //
}

// основная функция для чтения токенов и получения чистых линий из них;
// токены в этот момент не только сгруппированы в линии, но и имеют 
// предварительные базовые типы данных.
pub fn readTokens(buffer: Vec<u8>, debugMode: bool) -> Vec< Arc<RwLock<Line>> > 
{
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

  let mut      index: usize = 0;               // основной индекс чтения
  let   bufferLength: usize = buffer.len();    // размер буфера байтов
  let mut lineIndent: usize = 0;               // текущий отступ линии
  let mut lineTokens: Vec<Token> = Vec::new(); // прочитанные токены текущей линии

  let startTime: Instant = Instant::now(); // замеряем текущее время для debug

  let mut linesLinks:     Vec< Arc<RwLock<Line>> > = Vec::new(); // это ссылки на готовые линии 
  let mut readLineIndent: bool                     = true;       // флаг на проверку есть ли indent сейчас

  let mut byte: u8;
  while index < bufferLength 
  { // читаем байты
    byte = buffer[index]; // текущий байт

    // проверяем отступы, они могут быть указаны пробелами;
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
        readLineIndent = false;
        // смотрим является ли это endline
        if byte == b'\n' || byte == b';' 
        { // если это действительно конец строки,
          // то вкладываем возможные скобки
          bracketNesting(
            &mut lineTokens,
            &TokenType::CircleBracketBegin, 
            &TokenType::CircleBracketEnd
          );
          bracketNesting(
            &mut lineTokens,
            &TokenType::SquareBracketBegin, 
            &TokenType::SquareBracketEnd
          );
          // FigureBracketBegin и FigureBracketEnd
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

          // добавляем новую линию и пушим ссылку на неё
          linesLinks.push( 
            Arc::new(RwLock::new( 
              Line {
                tokens: std::mem::take(&mut lineTokens), // забираем все токены в линию, 
                                                         // оставляя пустой вектор для следующей
                indent: lineIndent,
                lines:  None, // в данный момент у неё нет вложенных линий, это будет чуть ниже
                parent: None  // также у неё нет родителя, это тоже будет ниже при вложении
              }
            ))
          );
          lineIndent = 0;

          readLineIndent = true; // это был конец строки
          index += 1;
        } else
        if byte == b'#' 
        { // ставим метку на комментарий в линии, по ним потом будут удалены линии
          deleteComment(&buffer, &mut index, &bufferLength); // пропускает комментарий
          lineTokens.push( Token::newEmpty( Some(TokenType::Comment) ) );
        } else
        if isDigit(&byte) || (byte == b'-' && index+1 < bufferLength && isDigit(&buffer[index+1])) 
        { // получаем все возможные численные примитивные типы данных
          lineTokens.push( getNumber(&buffer, &mut index, &bufferLength) );
        } else
        if isLetter(&byte) 
        { // получаем все возможные и зарезервированные слова
          lineTokens.push( getWord(&buffer, &mut index, &bufferLength) );
        } else
        if matches!(byte, b'\'' | b'"' | b'`') 
        { // получаем Char, String, RawString
          let mut token: Token = getQuotes(&buffer, &mut index);
          let tokenType: Option<TokenType> = token.getDataType();
          match tokenType 
          {
            tokenType if tokenType != None =>
            { // if formatted quotes
              let lineTokensLength: usize = lineTokens.len();
              match lineTokensLength 
              {
                lineTokensLength if lineTokensLength > 0 =>
                {
                  let backToken: &Token = &lineTokens[lineTokensLength-1];
                  if backToken.getDataType().unwrap_or_default() == TokenType::Word && 
                     backToken.getData().unwrap_or_default() == "f" 
                  {
                    match token.getDataType().unwrap_or_default()
                    {
                      TokenType::RawString =>
                      {
                       token.setDataType( Some(TokenType::FormattedRawString) ); 
                      }
                      TokenType::String =>
                      { 
                        token.setDataType( Some(TokenType::FormattedString) ); 
                      }
                      TokenType::Char =>
                      { 
                        token.setDataType( Some(TokenType::FormattedChar) ); 
                      }
                      _ => {}
                    }
                    lineTokens[lineTokensLength-1] = token; // replace the last token in place
                  } else 
                  { // basic quote
                    lineTokens.push(token);
                  }
                } 
                _ => { lineTokens.push(token); } // basic quote
              }
            }  
            _ => { index += 1; } // skip
          }
        } else
        // получаем возможные двойные и одиночные символы
        if isSingleChar(&byte) 
        {
          let token: Token = getOperator(&buffer, &mut index, &bufferLength);
          match token.getDataType()
          {
            None => { index += 1; } 
            _    => { lineTokens.push(token); }
          }
        } else 
        { // если мы ничего не нашли из возможного, значит этого нет в синтаксисе;
          // поэтому просто идём дальше
          index += 1;
        }
      }
    }
  }

  // вкладываем линии
  lineNesting(&mut linesLinks);
  // удаляем возможные вложенные комментарии по меткам
  deleteNestedComment(&mut linesLinks, 0);

  // debug output and return
  match debugMode 
  {
    true => 
    {
      let endTime:  Instant  = Instant::now();    // получаем текущее время
      let duration: Duration = endTime-startTime; // получаем сколько всего прошло
      outputLines(&linesLinks,&2); // выводим полученное AST дерево из линий
      //
      println!("     ┃");
      log("ok",&format!("xDuration: {:?}",duration));
    }
    false => {}
  }
  // возвращаем готовые ссылки на линии
  linesLinks
}