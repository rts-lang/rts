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

#[cfg(test)]
mod tests
{
  use crate::tokenizer::types::line::Line;
  use crate::tokenizer::types::token::Token;
  use crate::tokenizer::types::tokenType::TokenType;
  use super::bracketNesting;
  // ===============================================================================================

  /// Вспомогательная функция для генерации токенов
  fn createToken(tokenType: TokenType, data: &str) -> Token {
    Token::new(tokenType, data.to_string())
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn simpleBracket() -> ()
  {
    let mut tokens: Vec<Token> = 
      vec![
        createToken(TokenType::CircleBracketBegin, "("),
        createToken(TokenType::Word, "a"),
        createToken(TokenType::Plus, "+"),
        createToken(TokenType::Word, "b"),
        createToken(TokenType::CircleBracketEnd, ")"),
      ];

    //
    bracketNesting(&mut tokens, &TokenType::CircleBracketBegin, &TokenType::CircleBracketEnd);

    //
    assert_eq!(tokens.len(), 1, "Должен остаться только открывающий токен (контейнер)");
    
    //
    let tokenType: String = tokens[0].getDataType().to_string();
    assert_eq!(tokenType, TokenType::CircleBracketBegin.to_string(), "Ожидалась открывающая скобка");

    //
    let lines: &Vec<Line> = tokens[0].lines.as_ref().expect("Ожидались вложенные линии");
    assert_eq!(lines.len(), 1, "Ожидалась 1 линия");

    //
    let lineTokens: &Vec<Token> = lines[0].tokens.as_ref().expect("Ожидались токены в линии");
    assert_eq!(lineTokens.len(), 3, "Ожидалось 3 токена внутри: a, +, b");
    
    //
    let firstTokenData: String = lineTokens[0].getData().toString().unwrap_or_default();
    assert_eq!(firstTokenData, "a");
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn nestedBrackets() -> ()
  {
    let mut tokens: Vec<Token> = 
      vec![
        createToken(TokenType::CircleBracketBegin, "("),
        createToken(TokenType::CircleBracketBegin, "("),
        createToken(TokenType::Word, "x"),
        createToken(TokenType::CircleBracketEnd, ")"),
        createToken(TokenType::CircleBracketBegin, "("),
        createToken(TokenType::Word, "y"),
        createToken(TokenType::CircleBracketEnd, ")"),
        createToken(TokenType::CircleBracketEnd, ")"),
      ];

    //
    bracketNesting(&mut tokens, &TokenType::CircleBracketBegin, &TokenType::CircleBracketEnd);

    //
    assert_eq!(tokens.len(), 1, "Должен остаться только корневой открывающий токен");
    
    //
    let lines: &Vec<Line> = tokens[0].lines.as_ref().expect("Ожидались вложенные линии");
    assert_eq!(lines.len(), 1, "Ожидалась 1 линия на верхнем уровне");

    //
    let lineTokens: &Vec<Token> = lines[0].tokens.as_ref().expect("Ожидались токены в линии");
    assert_eq!(lineTokens.len(), 2, "Ожидалось 2 токена (вложенные скобки)");
    
    //
    let t1: String = lineTokens[0].getDataType().to_string();
    let t2: String = lineTokens[1].getDataType().to_string();
    let expected: String = TokenType::CircleBracketBegin.to_string();
    assert_eq!(t1, expected);
    assert_eq!(t2, expected);
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn commas() -> ()
  {
    let mut tokens: Vec<Token> = 
      vec![
        createToken(TokenType::CircleBracketBegin, "("),
        createToken(TokenType::Word, "a"),
        createToken(TokenType::Comma, ","),
        createToken(TokenType::Word, "b"),
        createToken(TokenType::Comma, ","),
        createToken(TokenType::Word, "c"),
        createToken(TokenType::CircleBracketEnd, ")"),
      ];

    //
    bracketNesting(&mut tokens, &TokenType::CircleBracketBegin, &TokenType::CircleBracketEnd);

    //
    assert_eq!(tokens.len(), 1, "Должен остаться только открывающий токен");
    
    //
    let lines: &Vec<Line> = tokens[0].lines.as_ref().expect("Ожидались вложенные линии");
    assert_eq!(lines.len(), 3, "Ожидалось 3 линии из-за разделения запятыми");

    //
    let expectedValues: [&str; 3] = ["a", "b", "c"];
    for (i, expectedVal) in expectedValues.iter().enumerate()
    {
      let lineTokens: &Vec<Token> = lines[i].tokens.as_ref().expect("Ожидались токены в линии");
      assert_eq!(lineTokens.len(), 1, "В каждой линии ожидается 1 токен");
      
      let tokenData: String = lineTokens[0].getData().toString().unwrap_or_default();
      assert_eq!(tokenData, *expectedVal);
    }
  }

  // ===============================================================================================
  
  /// todo desk
  #[test]
  fn emptyBrackets() -> ()
  {
    let mut tokens: Vec<Token> = 
      vec![
        createToken(TokenType::CircleBracketBegin, "("),
        createToken(TokenType::CircleBracketEnd, ")"),
      ];

    //
    bracketNesting(&mut tokens, &TokenType::CircleBracketBegin, &TokenType::CircleBracketEnd);

    //
    assert_eq!(tokens.len(), 1, "Должен остаться только открывающий токен");
    
    //
    let lines: &Vec<Line> = tokens[0].lines.as_ref().expect("Ожидались вложенные линии");
    assert_eq!(lines.len(), 1, "Ожидается 1 пустая линия");
    
    //
    let lineTokens: &Vec<Token> = lines[0].tokens.as_ref().expect("Ожидались токены");
    assert_eq!(lineTokens.len(), 0, "Линия должна быть пустой");
  }

  // ===============================================================================================
}

// =================================================================================================