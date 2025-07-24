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

/// Проверяет buffer по index и так пропускаем возможные комментарии;
/// Потом они будут удалены по меткам
fn deleteComment(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> ()
{
  *index += 1;
  while *index < *bufferLength && buffer[*index] != b'\n' 
  {
    *index += 1;
  }
}

/// Проверяет что байт является одиночным знаком доступным для синтаксиса
fn isSingleChar(c: &u8) -> bool 
{
  matches!(*c, 
    b'+' | b'-' | b'*' | b'/' | b'=' | b'%' | b'^' |
    b'>' | b'<' | b'?' | b'!' | b'&' | b'|' | 
    b'(' | b')' | b'{' | b'}' | b'[' | b']' | 
    b':' | b',' | b'.' | b'~'
  )
}

/// Проверяет что байт является числом
fn isDigit(c: &u8) -> bool 
{
  *c >= b'0' && *c <= b'9'
}
/// Проверяет buffer по index и так находит возможные примитивные числовые типы данных;
/// `UInt, Int, UFloat, Float, Rational, Complex`
///
/// todo: Ввести Complex числа;
///
/// todo: Ввести работу float с .1 или . как 0.0
fn getNumber(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> Token 
{
  let mut savedIndex: usize = *index; // index buffer
  let mut result: String = String::from(buffer[savedIndex] as char);
  savedIndex += 1;

  let mut      dot: bool = false; // dot check
  let mut negative: bool = false; // negative check
  let mut rational: bool = false; // rational check

  let mut byte1: u8; // Текущий символ
  let mut byte2: u8; // Следующий символ
  while savedIndex < *bufferLength 
  {
    byte1 = buffer[savedIndex]; // Значение текущего символа
    byte2 =                     // Значение следующего символа
      match savedIndex+1 < *bufferLength 
      {
        true  => { buffer[savedIndex+1] }  
        false => { b'\0' }
      };

    // todo: use match case
    if !negative && buffer[*index] == b'-' 
    { // Int/Float
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
      { false => {}
        true => { break; }
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
    } else { break; }
  }

  *index = savedIndex;
  // next return
  match (rational, dot, negative) 
  { // rational, dot, negative
    (true, _, _)     => Token::new( TokenType::Rational, result ),
    (_, true, true)  => Token::new( TokenType::Float,    result ),
    (_, true, false) => Token::new( TokenType::UFloat,   result ),
    (_, false, true) => Token::new( TokenType::Int,      result ),
    _                => Token::new( TokenType::UInt,     result ),
  }
}

/// Проверяет что байт является буквой a-z A-Z
fn isLetter(c: &u8) -> bool 
{
  (c|32)>=b'a'&&(c|32)<=b'z'
}
/// Проверяет buffer по index и так находит возможные слова;
/// Из них также выделяет сразу определяемые зарезервированные
fn getWord(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> Token 
{
  let mut savedIndex: usize = *index; // index buffer
  let mut result: String = String::from(buffer[savedIndex] as char);
  savedIndex += 1;
  let mut isLink: bool = false;

  let mut byte1: u8; // Текущий символ
  while savedIndex < *bufferLength 
  {
    byte1 = buffer[savedIndex]; // Значение текущего символа

    // todo: use match case
    if (isDigit(&byte1) || byte1 == b'.') || // Либо число, либо . как ссылка
       (isLink && byte1 == b'[' || byte1 == b']') // В случае ссылки мы можем читать динамические []
    {
      result.push(byte1 as char);
      savedIndex += 1;
      match byte1 == b'.' 
      { false => {} true =>
      { // Только если есть . то мы знаем что это ссылка
        isLink = true;
      }}
    } else
    {
      match isLetter(&byte1)
      { false => { break; } true =>
      {
        result.push(byte1 as char);
        savedIndex += 1;
      }}
      //
    }
  }

  *index = savedIndex;
  // next return
  match isLink 
  {
    true => Token::new( TokenType::Link, result.clone() ),
    false =>
    {
      match result.as_str()
      {
        "true"     => Token::newEmpty( TokenType::Bool ),
        "false"    => Token::newEmpty( TokenType::Bool ),
        //
        "UInt"     => Token::newEmpty( TokenType::UInt ),
        "Int"      => Token::newEmpty( TokenType::Int ),
        "UFloat"   => Token::newEmpty( TokenType::UFloat ),
        "Float"    => Token::newEmpty( TokenType::Float ),
        //
        "String"   => Token::newEmpty( TokenType::String ),
        "Char"     => Token::newEmpty( TokenType::Char ),
        //
        "None"     => Token::newEmpty(TokenType::None),
        //
        _          => Token::new( TokenType::Word, result ),
      }
    }
  }
}

/// Проверяет buffer по index и так находит возможные
/// Char, String, RawString
fn getQuotes(buffer: &[u8], index: &mut usize) -> Token 
{
  let byte1: u8 = buffer[*index]; // Начальный символ кавычки
  let mut result: String = String::new();

  *index += 1;

  let length: usize = buffer.len();
  let mut byte2: u8;

  let mut backslashCount: usize;
  let mut i: usize;
  while *index < length 
  {
    byte2 = buffer[*index]; // Текущий байт
    // Ошибка: конец строки внутри кавычек
    match byte2 
    {
      // Возврат строки не возможен, поскольку она может выйти за скобки и т.п. 
      // если он достиг конца строки уже;
      b'\n' => { return Token::newEmpty(TokenType::None); }
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
          1 => result.push(byte2 as char), // Экранированная кавычка
          _ => 
          {
            *index += 1; // Завершение строки
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
        1 => Token::new(TokenType::Char, result),
        _ => Token::newEmpty(TokenType::None)
      } 
    }
    b'"' => Token::new(TokenType::String, result),
    b'`' => Token::new(TokenType::RawString, result),
    _ => Token::newEmpty(TokenType::None),
  }
}

/// Проверяет buffer по index и так находит возможные двойные и одиночные операторы
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
        b'=' => { increment(2); Token::newEmpty(TokenType::PlusEquals) }
        b'+' => { increment(2); Token::newEmpty(TokenType::UnaryPlus) }
        _    => { increment(1); Token::newEmpty(TokenType::Plus) }
      }
    }
    b'-' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::MinusEquals) }
        b'-' => { increment(2); Token::newEmpty(TokenType::UnaryMinus) }
        b'>' => { increment(2); Token::newEmpty(TokenType::Pointer) }
        _    => { increment(1); Token::newEmpty(TokenType::Minus) }
      }
    }
    b'*' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::MultiplyEquals) }
        b'*' => { increment(2); Token::newEmpty(TokenType::UnaryMultiply) }
        _    => { increment(1); Token::newEmpty(TokenType::Multiply) }
      }
    }
    b'/' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::DivideEquals) }
        b'/' => { increment(2); Token::newEmpty(TokenType::UnaryDivide) }
        _    => { increment(1); Token::newEmpty(TokenType::Divide) }
      }
    }
    b'%' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::Modulo) } // todo: add new type in Token
        b'%' => { increment(2); Token::newEmpty(TokenType::Modulo) } // todo: add new type in Token
        _    => { increment(1); Token::newEmpty(TokenType::Modulo) }
      }
    }
    b'^' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::Exponent) } // todo: add new type in Token
        b'^' => { increment(2); Token::newEmpty(TokenType::Exponent) } // todo: add new type in Token
        _    => { increment(1); Token::newEmpty(TokenType::Disjoint) }
      }
    }
    b'>' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::GreaterThanOrEquals) }
        _    => { increment(1); Token::newEmpty(TokenType::GreaterThan) }
      }
    }
    b'<' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::LessThanOrEquals) }
        _    => { increment(1); Token::newEmpty(TokenType::LessThan) }
      }
    }
    b'!' => 
    {
      match nextByte 
      {
        b'=' => { increment(2); Token::newEmpty(TokenType::NotEquals) }
        _    => { increment(1); Token::newEmpty(TokenType::Exclusion) }
      }
    }
    b'~' =>
    {
      match nextByte
      {
        b'~' => { increment(2); Token::newEmpty(TokenType::DoubleTilde) }
        _    => { increment(1); Token::newEmpty(TokenType::Tilde) }
      }
    }
    b'&' => { increment(1); Token::newEmpty(TokenType::Joint) }
    b'|' => { increment(1); Token::newEmpty(TokenType::Inclusion) }
    b'=' => { increment(1); Token::newEmpty(TokenType::Equals) }
    // brackets
    b'(' => { increment(1); Token::newEmpty(TokenType::CircleBracketBegin) }
    b')' => { increment(1); Token::newEmpty(TokenType::CircleBracketEnd) }
    b'{' => { increment(1); Token::newEmpty(TokenType::FigureBracketBegin) }
    b'}' => { increment(1); Token::newEmpty(TokenType::FigureBracketEnd) }
    b'[' => { increment(1); Token::newEmpty(TokenType::SquareBracketBegin) }
    b']' => { increment(1); Token::newEmpty(TokenType::SquareBracketEnd) }
    // other
    b';' => { increment(1); Token::newEmpty(TokenType::Endline) }
    b':' => { increment(1); Token::newEmpty(TokenType::Colon) }
    b',' => { increment(1); Token::newEmpty(TokenType::Comma) }
    b'.' => { increment(1); Token::newEmpty(TokenType::Dot) }
    b'?' => { increment(1); Token::newEmpty(TokenType::Question) }
    _ => Token::newEmpty(TokenType::None)
  }
}

/// Основная функция, которая вкладывает токены в скобки `() [] {}` от начальной скобки
/// до закрывающей; Её особенность в рекурсивном вызове себя для дочерних токенов
fn bracketNesting(tokens: &mut Vec<Token>, beginType: &TokenType, endType: &TokenType) -> ()
{
  /* todo Эта часть помогала пройти вложения, чтобы () [] {} видно было друг в друге
  for token in tokens.iter_mut()
  { // Чтение токенов
    match &mut token.tokens
    { None => {} Some(tokens) =>
    { // Рекурсия
      println!("!! {:?}",tokens);
      bracketNesting(tokens, beginType, endType);
    }}
  }
  */
  // Вкладывание
  blockNesting(tokens, beginType, endType);
}
/// Эта функция является дочерней bracketNesting;
/// Занимается вложением линий в токены;
/// От начальной скобки до закрывающей;
/// Делит токены через запятую.
///
/// todo может использовать split
fn blockNesting(tokens: &mut Vec<Token>, beginType: &TokenType, endType: &TokenType) -> ()
{
  let mut isReadData: bool = false; // Читаем данные в буфер?
  let mut readData: Vec<Token> = Vec::new(); // Буфер токенов
  let mut readDataLines: Vec<Line> = Vec::new(); // Линии из токенов

  let mut i: usize = tokens.len();
  while i > 0
  {
    i -= 1;
    match tokens[i].getDataType()
    {
      tokenType if tokenType == beginType =>
      { // Конец чтения
        readDataLines.insert(
          0,
          Line
          {
            tokens: Some( std::mem::take(&mut readData) ),
            indent: None,
            lines: None,
            parent: None,
          }
        );
        tokens[i].lines = Some( std::mem::take(&mut readDataLines) );
        return;
      }
      tokenType if tokenType == endType =>
      { // Начало чтения
        if !isReadData
        {
          tokens.remove(i);
          isReadData = true;
        } else
        {
          // Вложенный блок
          let before: usize = tokens.len();
          blockNesting(tokens, beginType, endType);
          // Сдвиг текущего списка
          let removed: usize = before - tokens.len();
          i = i - removed;
          //
          readData.insert(0, tokens.remove(i));
        }
      }
      TokenType::Comma =>
      { // Разделение буфера на линии
        tokens.remove(i);
        readDataLines.insert(
        0,
        Line
          {
            tokens: Some( std::mem::take(&mut readData) ),
            indent: None,
            lines: None,
            parent: None,
          }
        );
      }
      _ => match  isReadData
      { // Чтение данных в буфер
        false => {}
        true => readData.insert(0, tokens.remove(i))
      }
    }
  }
}

/// Проверка на вхождение в срез
macro_rules! matchesIn 
{
  ($value:expr, $slice:expr) => 
  {
    $slice.contains(&$value)
  };
}

/// Разделяет токены по типу токена-разделителя
pub fn splitByType(tokens: Vec<Token>, separatorTypes: &[TokenType]) -> Vec<Line>
{
  let mut lines: Vec<Line> = Vec::new();
  let mut buffer: Vec<Token> = Vec::new();

  for token in tokens.into_iter()
  {
    if matchesIn!(token.getDataType(), separatorTypes) 
    {
      lines.push(Line
      {
        tokens: Some(buffer),
        indent: None,
        lines: None,
        parent: None,
      });
      buffer = Vec::new();
    }
    else
    {
      buffer.push(token);
    }
  }

  if !buffer.is_empty()
  {
    lines.push(Line
    {
      tokens: Some(buffer),
      indent: None,
      lines: None,
      parent: None,
    });
  }

  lines
}

/// Вкладывает линии токенов друг в друга
fn lineNesting(linesLinks: &mut Vec< Arc<RwLock<Line>> >) -> ()
{
  let mut index:     usize = 0;                // current line index
  let mut nextIndex: usize = 1;                // next    line index
  let mut length:    usize = linesLinks.len(); // all lines links length

  let mut compare: bool;
  while index < length && nextIndex < length
  { // Если мы не дошли до конца, то читаем линии
    compare =
    { // Только в Tokenizer мы уверены, что существует indent
      let currentIndent: usize = linesLinks[index].read().unwrap().indent.unwrap();
      let nextIndent:    usize = linesLinks[nextIndex].read().unwrap().indent.unwrap();
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
          { // Если вложения уже были, то просто делаем push
            lineLines.push(nestingLineLink); // nesting
            lineNesting(lineLines);          // cycle
          },
          None =>
          { // Если вложения не было до этого, то создаём
            currentLine.lines = Some(vec![nestingLineLink]);  // nesting
            lineNesting(currentLine.lines.as_mut().unwrap()); // cycle
          }
        }
      }
      false =>
      {
        index += 1;
        nextIndex = index+1;
      }
    }
  }
}

/// Удаляет возможные вложенные комментарии по меткам;
/// Это такие комментарии, которые имеют вложения.
///
/// Кроме того, создаёт линии разделители (separator).
fn deleteNestedComment(linesLinks: &mut Vec< Arc<RwLock<Line>> >, mut index: usize) -> ()
{
  let mut linesLinksLength: usize = linesLinks.len(); // Количество ссылок строк
  let mut lastTokenIndex:   usize; // Это указатель на метку где TokenType::Comment
  // Это может быть либо последний токен, либо первый токен в большом комментарии;

  let mut deleteLine: bool;
  let mut line: RwLockWriteGuard<'_, Line>;

  while index < linesLinksLength
  {
    deleteLine = false; // Состояние удаления текущей линии
    'exit:
    { // Прерывание чтобы не нарушать мутабельность
      line = linesLinks[index].write().unwrap();

      match &mut line.lines
      { None => {} Some(lineLines) =>
      { // Рекурсивно обрабатываем вложенные линии
        deleteNestedComment(lineLines, 0);
      }}

      // Логика для разделителей
      match line.tokens.is_none()
      { false => {} true =>
      { // Пропускаем разделители, они нужны для синтаксиса
        // Если разделитель имеет вложения
        match &line.lines
        { None => {} Some(_) =>
        { // Выходим из прерывания, т.к это безымянный блок
          break 'exit;
        }}

        // Проверяем на скопление разделителей
        match index+1 < linesLinksLength
        { false => {} true =>
        { // Если есть линия ниже, то мы можем предполагать, что
          // Она может быть тоже разделителем;
          match
            linesLinks[index+1].write().unwrap()
             .tokens.is_none()
          { // Если токенов в следующей линии не было, значит точно separator;
            // Повторение подобных условий оставит 1 separator линию по итогу;
            false => {}
            true  => deleteLine = true
          }
        }}

        // Обычный разделитель
        break 'exit; // Выходим из прерывания
      }}

      // Логика для комментариев
      match line.tokens
      { None => {} Some(ref mut tokens) =>
      {
        lastTokenIndex = tokens.len() -1;
        match tokens.get(lastTokenIndex)
        { None => {} Some(token) =>
        {

          match *token.getDataType() == TokenType::Comment
          { false => {} true =>
          { // Удаляем комментарии

            tokens.remove(lastTokenIndex);
            // Проверяем если есть вложенные линии,
            // что комментарий не удалится весь
            // и продолжается на вложенные линии
            match &line.lines
            { None => {}, Some(_) =>
            {
              line.lines = None
            }}

            // Переходим к удалению пустой линии
            match &line.tokens
            {
              Some(tokens) =>
              { // Пустой массив
                match tokens.is_empty()
                { false => {} true =>
                {
                  deleteLine = true; // Линия была удалена
                  break 'exit;       // Выходим из прерывания
                }}
              }
              None =>
              { // Просто пустой
                deleteLine = true; // Линия была удалена
                break 'exit;       // Выходим из прерывания
              }
            }
          }}
          //
        }}
        //
      }}
      //
    }
    // Когда линия удалена в прерывании,
    // её можно спокойно удалить
    match deleteLine
    { false => {} true =>
    {
      drop(line);
      linesLinks.remove(index);
      linesLinksLength -= 1;
      continue;
    }}
    // Продолжаем чтение
    index += 1;
  }
}

/// Выводит токен, его тип данных
pub fn outputTokens(tokens: &Vec<Token>, lineIndent: &usize, indent: &usize) -> ()
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
    c =
      match i == tokenCount
      {
        true  => { 'X' }
        false => { '┃' }
      };

    tokenType = token.getDataType(); // Тип токена
    match token.getData().toString()
    {
      Some(tokenData) =>
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
        }
      }
      _ =>
      { // Если это токен только с типом, то выводим тип как символ
        match token.isPrimitive()
        {
          true =>
            log("parserToken",&format!(
              "{}\\b{}\\c{}|{}",
              lineIndentString,
              c,
              identString,
              tokenType.to_string()
            )),
          false =>
            formatPrint(&format!(
              "{}\\b{}\\c{}{}\n",
              lineIndentString,
              c,
              identString,
              tokenType.to_string()
            ))
        }
      }
    }

    match &token.lines
    { None => {} Some(lines) =>
    { // Если есть вложения у токена, то рекурсивно обрабатываем их
      for (i, line) in lines.iter().enumerate()
      {
        outputTokens(&line.tokens.clone().unwrap_or_default(), lineIndent, &(indent+1));
        match i != lines.len()-1
        { false => {}
          true => log("parserToken", &format!("{}\\b┃\\c", lineIndentString))
        }
      }
    }}
  }
}
/// Выводит информацию о линии, а также токены линии
pub fn outputLines(linesLinks: &Vec< Arc<RwLock<Line>> >, indent: &usize) -> ()
{
  let identStr1: String = " ".repeat(indent*2);   // Это отступ для главной строки
  let identStr2: String = format!("{} ", identStr1); // Это для дочерних токенов

  let mut line: RwLockReadGuard<'_, Line>;
  for (i, lineLink) in linesLinks.iter().enumerate()
  { // Проходи по линиям через чтение
    line = lineLink.read().unwrap();
    log("parserBegin", &format!("{} {}",identStr1,i));

    match &line.tokens
    {
      None =>
      { // Заголовок для разделителей
        formatPrint(&format!("{}\\b┗ \\fg(#90df91)Separator\\c\n",identStr2));
      }
      Some(tokens) =>
      { // Заголовок для начала вложенных токенов
        formatPrint(&format!("{}\\b┣ \\fg(#90df91)Tokens\\c\n",identStr2));
        // todo плохо используются tokens
        outputTokens(tokens, &indent, &1); // выводим вложенные токены
      }
    }

    match &line.lines
    { None => {} Some(lineLines) =>
    { // Заголовок для начала вложенных линий
      formatPrint(&format!("{}\\b┗ \\fg(#90df91)Lines\\c\n",identStr2));
      outputLines(lineLines, &(indent+1)); // выводим вложенные линии
    }}
  }
  //
}

/// Основная функция для чтения токенов и получения чистых линий из них;
/// Токены в этот момент не только сгруппированы в линии, но и имеют
/// предварительные базовые типы данных
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

  let mut      index: usize = 0;               // Основной индекс чтения
  let   bufferLength: usize = buffer.len();    // Размер буфера байтов
  let mut lineIndent: usize = 0;               // Текущий отступ линии
  let mut lineTokens: Vec<Token> = Vec::new(); // Прочитанные токены текущей линии

  let startTime: Instant = Instant::now(); // Замеряем текущее время для debug

  let mut linesLinks:     Vec< Arc<RwLock<Line>> > = Vec::new(); // Ссылки на готовые линии
  let mut readLineIndent: bool                     = true;       // Флаг на проверку есть ли indent сейчас

  let mut byte: u8;
  while index < bufferLength
  { // Читаем байты
    byte = buffer[index]; // текущий байт

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
        readLineIndent = false;
        // Смотрим является ли это endline
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
          deleteComment(&buffer, &mut index, &bufferLength); // Пропускает комментарий
          lineTokens.push( Token::newEmpty(TokenType::Comment) );
        } else
        if isDigit(&byte) || (byte == b'-' && index+1 < bufferLength && isDigit(&buffer[index+1]))
        { // Получаем все возможные численные примитивные типы данных
          lineTokens.push( getNumber(&buffer, &mut index, &bufferLength) );
        } else
        if isLetter(&byte)
        { // Получаем все возможные и зарезервированные слова
          lineTokens.push( getWord(&buffer, &mut index, &bufferLength) );
        } else
        if matches!(byte, b'\'' | b'"' | b'`')
        { // Получаем Char, String, RawString
          let mut token: Token = getQuotes(&buffer, &mut index);
          let tokenType: TokenType = token.getDataType().clone();
          match tokenType
          {
            tokenType if tokenType != TokenType::None =>
            { // if formatted quotes
              let lineTokensLength: usize = lineTokens.len();
              match lineTokensLength
              {
                lineTokensLength if lineTokensLength > 0 =>
                {
                  let backToken: &Token = &lineTokens[lineTokensLength-1];
                  // todo if -> match
                  if *backToken.getDataType() == TokenType::Word &&
                     backToken.getData().toString().unwrap_or_default() == "f"
                  {
                    match tokenType
                    {
                      TokenType::RawString =>
                      {
                       token.setDataType(TokenType::FormattedRawString);
                      }
                      TokenType::String =>
                      {
                        token.setDataType(TokenType::FormattedString);
                      }
                      TokenType::Char =>
                      {
                        token.setDataType(TokenType::FormattedChar);
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
        // Получаем возможные двойные и одиночные символы
        if isSingleChar(&byte)
        {
          let token: Token = getOperator(&buffer, &mut index, &bufferLength);
          lineTokens.push(token);
        } else
        { // Если мы ничего не нашли из возможного, значит этого нет в синтаксисе;
          // Поэтому просто идём дальше
          index += 1;
        }
      }
    }
  }

  // Вкладываем линии
  lineNesting(&mut linesLinks);
  // Удаляем возможные вложенные комментарии по меткам
  deleteNestedComment(&mut linesLinks, 0);

  // debug output and return
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