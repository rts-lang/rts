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
  let mut index: usize = tokens.len();
  blockNesting(tokens, beginType, endType, &mut index);
}

/// Эта функция является дочерней bracketNesting;
/// Занимается вложением линий в токены;
/// От начальной скобки до закрывающей;
/// Делит токены через запятую.
///
/// todo может использовать split
fn blockNesting(tokens: &mut Vec<Token>, beginType: &TokenType, endType: &TokenType, index: &mut usize) -> ()
{
  let mut isReadData: bool = false; // Читаем данные в буфер?
  let mut readData: Vec<Token> = Vec::new(); // Буфер токенов
  let mut readDataLines: Vec<Line> = Vec::new(); // Линии из токенов

  while *index > 0
  {
    *index -= 1;
    let i: usize = *index;

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
          *index += 1;
          blockNesting(tokens, beginType, endType, index);
          //
          readData.insert(0, tokens.remove(*index));
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
  use crate::tokenizer::read::nesting::brackets::bracketNesting;
  use crate::tokenizer::types::line::Line;
  use crate::tokenizer::types::token::Token;
  use crate::tokenizer::types::tokenType::TokenType;
  // ===============================================================================================

  /// Вспомогательная функция: 
  /// Генерация токенов
  fn createToken(tokenType: TokenType, data: &str) -> Token 
  {
    Token::new(tokenType, data.to_string())
  }

  /// Вспомогательная функция: 
  /// Рекурсивно разворачивает линии и вложенные токены в плоский список пар (тип, данные)
  fn flattenLines(lines: &[Line]) -> Vec<(TokenType, String)> 
  {
    let mut result: Vec<(TokenType, String)> = Vec::new();
    for line in lines 
    {
      if let Some(tokens) = &line.tokens 
      {
        for token in tokens 
        {
          result.push((
            *token.getDataType(), 
            token.getData().toString().unwrap_or_default())
          );
          if let Some(innerLines) = &token.lines {
            result.extend(flattenLines(innerLines));
          }
          //
        }
      }
      // НЕ обрабатываем line.lines, потому что в bracketNesting они не используются
    }
    result
  }

  /// Вспомогательная функция: 
  /// Табличная проверка токенов
  fn checkLines(lines: &[Line], expected: &[(TokenType, &str)]) -> ()
  {
    let flat: Vec<(TokenType, String)> = flattenLines(lines);
    assert_eq!(flat.len(), expected.len(), "Длина плоского списка токенов не совпадает с ожидаемой");
    
    //
    for (i, (actualType, actualData)) in flat.iter().enumerate()
    {
      let (expType, expData): &(TokenType, &str) = &expected[i];
      assert_eq!(actualType.to_string(), expType.to_string(), "Токен на позиции {}: несоответствие типа", i);
      assert_eq!(actualData, expData, "Токен на позиции {}: несоответствие данных", i);
    }
  }

  // ===============================================================================================

  /// Проверяет базовое вложение: `(a+b)`
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
    #[cfg(not(feature = "analyzer"))]
    assert_eq!(tokens.len(), 1, "Должен остаться только открывающий токен (контейнер)");
    #[cfg(feature = "analyzer")]
    assert_eq!(tokens.len(), 2, "Внешняя закрывающая скобка должна сохраниться в токене");
    
    //
    let tokenType: String = tokens[0].getDataType().to_string();
    assert_eq!(tokenType, TokenType::CircleBracketBegin.to_string(), "Ожидалась открывающая скобка");

    //
    let lines: &Vec<Line> = tokens[0].lines.as_ref().expect("Ожидались вложенные линии");
    assert_eq!(lines.len(), 1, "Ожидалась 1 линия");

    // Табличная проверка содержимого
    checkLines(lines, &[
      (TokenType::Word, "a"),
      (TokenType::Plus, "+"),
      (TokenType::Word, "b"),
    ]);
  }

  // ===============================================================================================

  /// Проверяет вложенные скобки: `((x)(y))`
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
    #[cfg(not(feature = "analyzer"))]
    assert_eq!(tokens.len(), 1, "Должен остаться только корневой открывающий токен");
    #[cfg(feature = "analyzer")]
    assert!(tokens.len() > 1, "Маркеры закрывающих скобок должны остаться");
    
    //
    let lines: &Vec<Line> = tokens[0].lines.as_ref().expect("Ожидались вложенные линии");
    assert_eq!(lines.len(), 1, "Ожидалась 1 линия на верхнем уровне");

    // Табличная проверка содержимого
    #[cfg(not(feature = "analyzer"))]
    checkLines(lines, &[
      (TokenType::CircleBracketBegin, "("),
      (TokenType::Word, "x"),
      (TokenType::CircleBracketBegin, "("),
      (TokenType::Word, "y"),
    ]);
    
    #[cfg(feature = "analyzer")]
    {
      let flat: Vec<(TokenType, String)> = flattenLines(lines);
      assert!(flat.len() >= 2, "Ожидались токены включая сохраненные маркеры");
      assert_eq!(flat[0].0.to_string(), TokenType::CircleBracketBegin.to_string());
    }
  }

  // ===============================================================================================

  /// Проверяет разделение запятыми: `(a,b,c)`
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
    #[cfg(not(feature = "analyzer"))]
    assert_eq!(tokens.len(), 1, "Должен остаться только открывающий токен");
    #[cfg(feature = "analyzer")]
    assert_eq!(tokens.len(), 4, "Должны сохраниться запятые и закрывающая скобка");
    
    //
    let lines: &Vec<Line> = tokens[0].lines.as_ref().expect("Ожидались вложенные линии");
    assert_eq!(lines.len(), 3, "Ожидалось 3 линии из-за разделения запятыми");

    // Табличная проверка всех линий последовательно
    checkLines(lines, &[
      (TokenType::Word, "a"),
      (TokenType::Word, "b"),
      (TokenType::Word, "c"),
    ]);
  }

  // ===============================================================================================

  /// Проверяет пустые скобки: `()`
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
    #[cfg(not(feature = "analyzer"))]
    assert_eq!(tokens.len(), 1, "Должен остаться только открывающий токен");
    #[cfg(feature = "analyzer")]
    assert_eq!(tokens.len(), 2, "Открывающая и закрывающая скобки сохраняются");
    
    //
    let lines: &Vec<Line> = tokens[0].lines.as_ref().expect("Ожидались вложенные линии");
    assert_eq!(lines.len(), 1, "Ожидается 1 пустая линия");
    
    // Табличная проверка пустого содержимого
    checkLines(lines, &[]);
  }

  // ===============================================================================================
}

// =================================================================================================