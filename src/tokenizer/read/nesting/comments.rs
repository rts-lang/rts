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