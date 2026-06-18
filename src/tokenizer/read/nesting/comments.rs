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
      // todo Не удаляет Separator если они друг за другом -
      //  а могли бы быть просто 1 Separator
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

  /// Вспомогательная функция: 
  /// Создаёт линию `Arc<RwLock<Line>>` по описанию токенов и списку вложенных линий
  fn buildLine(tokensDesc: &[(TokenType, &str)], nestedLines: Vec< Arc<RwLock<Line>> >) -> Arc<RwLock<Line>> 
  {
    let tokens: Vec<Token> = tokensDesc.iter()
      .map(|(ty, data)| Token::new(ty.clone(), data.to_string()))
      .collect();
    //
    Arc::new(RwLock::new(Line {
      tokens: Some(tokens),
      indent: None,
      lines: if nestedLines.is_empty() { None } else { Some(nestedLines) },
      parent: None,
    }))
  }

  /// Вспомогательная функция: 
  /// Создаёт линию-разделитель (не содержит токенов и вложенных линий)
  fn buildSeparator() -> Arc<RwLock<Line>> 
  {
    Arc::new(RwLock::new(Line {
      tokens: None,
      indent: None,
      lines: None,
      parent: None,
    }))
  }

  /// Вспомогательная функция: 
  /// Преобразует список линий в плоский вектор пар (тип токена, данные), обходя вложенные линии
  fn flattenLines(lines: &[Arc<RwLock<Line>>]) -> Vec<(TokenType, String)> 
  {
    let mut result: Vec<(TokenType, String)> = Vec::new();
    for lineArc in lines 
    {
      let line: RwLockReadGuard<Line> = lineArc.read().unwrap();
      if let Some(tokens) = &line.tokens 
      {
        for token in tokens 
        {
          let data: String = token.getData().toString().unwrap_or_default();
          result.push((*token.getDataType(), data));
          // В comments.rs вложенные линии находятся в line.lines,
          // но они обрабатываются рекурсивно через вызов flatten_lines,
          // а не через token.lines.
          // Если у вас есть вложенные линии на уровне самой линии (line.lines),
          // их нужно обойти здесь:
          if let Some(innerLines) = &line.lines {
            result.extend(flattenLines(innerLines));
          }
          //
        }
      }
      // Если линия не имеет токенов (разделитель), мы её пропускаем в плоском списке
    }
    result
  }

  /// Вспомогательная функция: 
  /// Табличная проверка токенов
  fn checkLines(result: &[Arc<RwLock<Line>>], expected_tokens: &[(TokenType, &str)]) 
  {
    let flat: Vec<(TokenType, String)> = flattenLines(result);
    assert_eq!(flat.len(), expected_tokens.len());
    
    //
    for (i, (actualType, actualData)) in flat.iter().enumerate() 
    {
      let (exp_ty, exp_data) = &expected_tokens[i];
      assert_eq!(actualType.to_string(), exp_ty.to_string());
      assert_eq!(actualData, exp_data);
    }
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn commentRemoval() -> ()
  {
    let mut lines: Vec< Arc<RwLock<Line>> > = vec![
      buildLine(&[(TokenType::Word, ""), (TokenType::Comment, "")], vec![])
    ];

    //
    deleteNestedComment(&mut lines, 0);

    //
    #[cfg(not(feature = "analyzer"))]
    checkLines(&lines, &[(TokenType::Word, "")]);

    #[cfg(feature = "analyzer")]
    checkLines(&lines, &[(TokenType::Word, ""), (TokenType::Comment, "")]);
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn nestedRemoval() -> ()
  {
    let childLine: Arc<RwLock<Line>> =
      buildLine(&[(TokenType::Word, ""), (TokenType::Comment, "")], vec![]);

    let mut lines: Vec< Arc<RwLock<Line>> > = vec![
      buildLine(&[(TokenType::Word, "")], vec![childLine])
    ];

    //
    deleteNestedComment(&mut lines, 0);

    //
    #[cfg(not(feature = "analyzer"))]
    checkLines(&lines, &[(TokenType::Word, ""), (TokenType::Word, "")]);

    #[cfg(feature = "analyzer")]
    checkLines(&lines, &[(TokenType::Word, ""), (TokenType::Word, ""), (TokenType::Comment, "")]);
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn emptyLineRemoval() -> ()
  {
    let mut lines: Vec< Arc<RwLock<Line>> > = vec![
      buildLine(&[(TokenType::Word, "")], vec![]),
      buildLine(&[(TokenType::Comment, "")], vec![])
    ];

    //
    deleteNestedComment(&mut lines, 0);

    //
    #[cfg(not(feature = "analyzer"))]
    {
      assert_eq!(lines.len(), 1, "Пустая линия с комментарием должна быть полностью удалена");
      checkLines(&lines, &[(TokenType::Word, "")]);
    }
    #[cfg(feature = "analyzer")]
    {
      assert_eq!(lines.len(), 2, "Линия с комментарием должна быть сохранена");
      checkLines(&lines, &[(TokenType::Word, ""), (TokenType::Comment, "")]);
    }
  }

  // ===============================================================================================

  /// todo desk
  #[test]
  fn separators() -> ()
  {
    let mut lines: Vec< Arc<RwLock<Line>> > = vec![
      buildSeparator(),
      buildSeparator(),
      buildSeparator(),
      buildLine(&[(TokenType::Word, "")], vec![])
    ];

    //
    deleteNestedComment(&mut lines, 0);

    //
    assert_eq!(lines.len(), 2, "Должен остаться 1 разделитель и 1 линия с токеном");

    let line0 = lines[0].read().unwrap();
    assert!(line0.tokens.is_none(), "Первая линия должна быть разделителем");
    drop(line0);

    checkLines(&lines, &[(TokenType::Word, "")]);
  }

  // ===============================================================================================
}

// =================================================================================================