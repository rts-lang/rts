use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Основная функция, которая вкладывает токены в скобки `() [] {}` от начальной скобки
/// до закрывающей; Её особенность в рекурсивном вызове себя для дочерних токенов
pub fn bracketNesting(tokens: &mut Vec<Token>, beginType: &TokenType, endType: &TokenType) -> ()
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
          #[cfg(not(feature = "analyzer"))]
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
        #[cfg(not(feature = "analyzer"))]
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
      _ => match isReadData
      { // Чтение данных в буфер
        false => {}
        true => readData.insert(0, tokens.remove(i))
      }
    }
    //
  }
}

// =================================================================================================