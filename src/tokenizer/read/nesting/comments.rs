use std::sync::{Arc, RwLock, RwLockWriteGuard};
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Удаляет возможные вложенные комментарии по меткам;
/// Это такие комментарии, которые имеют вложения.
///
/// Кроме того, создаёт линии разделители (separator).
pub fn deleteNestedComment(linesLinks: &mut Vec< Arc<RwLock<Line>> >, mut index: usize) -> ()
{
  let mut linesLinksLength: usize = linesLinks.len(); // Количество ссылок строк
  let mut lastTokenIndex:   usize; // Это указатель на метку где TokenType::Comment
  // Это может быть либо последний токен, либо первый токен в большом комментарии;

  let mut deleteLine: bool;
  let mut line: RwLockWriteGuard<Line>;

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
      #[cfg(feature = "analyzer")]
      let hadNested: bool = line.lines.is_some();
      match line.tokens
      { None => {} Some(ref mut tokens) =>
      {
        lastTokenIndex = tokens.len() -1;
        match tokens.get(lastTokenIndex)
        { None => {} Some(token) =>
        {

          match *token.getDataType() == TokenType::Comment
          { false => {} true =>
          {
            #[cfg(feature = "analyzer")]
            {
              // Для анализатора НЕ объединяем токены в один.
              // Оставляем структуру линии нетронутой, чтобы подсветка синтаксиса работала корректно.
              if hadNested {
                line.lines = None;
              }
              break 'exit; // Прерываем блок, чтобы строка не удалилась и токены остались
            }
            #[cfg(not(feature = "analyzer"))]
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
            }
            //
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

// =================================================================================================

#[cfg(test)]
mod tests
{
  use std::sync::{Arc, RwLock, RwLockReadGuard};
  use crate::tokenizer::read::nesting::comments::deleteNestedComment;
  use crate::tokenizer::types::line::Line;
  use crate::tokenizer::types::token::Token;
  use crate::tokenizer::types::tokenType::TokenType;
  // ===============================================================================================

  /// Вспомогательная функция для создания линий
  fn createLine(tokens: Option<Vec<Token>>, lines: Option<Vec<Arc<RwLock<Line>>>>) -> Arc<RwLock<Line>>
  {
    Arc::new(RwLock::new(Line {
      tokens,
      indent: None,
      lines,
      parent: None
    }))
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn commentRemoval() -> ()
  {
    let mut lines: Vec<Arc<RwLock<Line>>> = vec![
      createLine(Some(vec![
        Token::newEmpty(TokenType::Word),
        Token::newEmpty(TokenType::Comment)
      ]), None)
    ];

    //
    deleteNestedComment(&mut lines, 0);

    //
    assert_eq!(lines.len(), 1, "Линия должна остаться");
    let line: RwLockReadGuard<Line> = lines[0].read().unwrap();
    let tokens: &Vec<Token> = line.tokens.as_ref().unwrap();

    //
    #[cfg(not(feature = "analyzer"))]
    assert_eq!(tokens.len(), 1, "Токен комментария должен быть удален");
    #[cfg(feature = "analyzer")]
    assert_eq!(tokens.len(), 2, "Токен комментария должен быть сохранен");

    //
    assert!(tokens[0].getDataType() == &TokenType::Word, "Должен остаться только Word");
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn nestedRemoval() -> ()
  {
    let childLine: Arc<RwLock<Line>> = 
      createLine(Some(vec![
        Token::newEmpty(TokenType::Word),
        Token::newEmpty(TokenType::Comment)
      ]), None);

    let mut lines: Vec< Arc<RwLock<Line>> > = vec![
      createLine(Some(vec![
        Token::newEmpty(TokenType::Word)
      ]), Some(vec![childLine]))
    ];

    //
    deleteNestedComment(&mut lines, 0);

    //
    assert_eq!(lines.len(), 1, "Корневая линия должна остаться");
    let line: RwLockReadGuard<Line> = lines[0].read().unwrap();
    let nestedLines: &Vec<Arc<RwLock<Line>>> = line.lines.as_ref().unwrap();
    assert_eq!(nestedLines.len(), 1, "Вложенная линия должна остаться");
    
    //
    let child: RwLockReadGuard<Line> = nestedLines[0].read().unwrap();
    let childTokens: &Vec<Token> = child.tokens.as_ref().unwrap();

    //
    #[cfg(not(feature = "analyzer"))]
    assert_eq!(childTokens.len(), 1, "Вложенный комментарий должен быть рекурсивно удален");
    #[cfg(feature = "analyzer")]
    assert_eq!(childTokens.len(), 2, "Вложенный комментарий должен быть сохранен");

    //
    assert!(childTokens[0].getDataType() == &TokenType::Word, "Ожидался только Word");
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn emptyLineRemoval() -> ()
  {
    let mut lines: Vec< Arc<RwLock<Line>> > = 
      vec![
        createLine(Some(vec![
          Token::newEmpty(TokenType::Word)
        ]), None),
        createLine(Some(vec![
          Token::newEmpty(TokenType::Comment)
        ]), None)
      ];

    //
    deleteNestedComment(&mut lines, 0);

    //
    #[cfg(not(feature = "analyzer"))]
    assert_eq!(lines.len(), 1, "Пустая линия с комментарием должна быть полностью удалена");
    #[cfg(feature = "analyzer")]
    assert_eq!(lines.len(), 2, "Линия с комментарием должна быть сохранена");

    //
    let line: RwLockReadGuard<Line> = lines[0].read().unwrap();
    assert!(line.tokens.as_ref().unwrap()[0].getDataType() == &TokenType::Word, "Ожидалась линия с Word");
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn separators() -> ()
  {
    let mut lines: Vec< Arc<RwLock<Line>> > = 
      vec![
        createLine(None, None), // separator
        createLine(None, None), // separator
        createLine(None, None), // separator
        createLine(Some(vec![Token::newEmpty(TokenType::Word)]), None)
      ];

    //
    deleteNestedComment(&mut lines, 0);

    //
    assert_eq!(lines.len(), 2, "Должен остаться 1 разделитель и 1 линия с токеном");
    let line0: RwLockReadGuard<Line> = lines[0].read().unwrap();
    assert!(line0.tokens.is_none(), "Первая линия должна быть разделителем");
    let line1: RwLockReadGuard<Line> = lines[1].read().unwrap();
    assert!(line1.tokens.is_some(), "Вторая линия должна содержать токены");
  }

  // ===============================================================================================
}

// =================================================================================================