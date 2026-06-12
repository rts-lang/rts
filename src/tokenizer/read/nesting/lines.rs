use std::sync::{Arc, RwLock, RwLockWriteGuard};
use crate::tokenizer::types::line::Line;
// =================================================================================================

/// Вкладывает линии токенов друг в друга
pub fn lineNesting(linesLinks: &mut Vec< Arc<RwLock<Line>> >) -> ()
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
        let mut currentLine: RwLockWriteGuard<Line> = linesLinks[index].write().unwrap();
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
    //
  }
}

// =================================================================================================

#[cfg(test)]
mod tests
{
  use std::sync::{Arc, RwLock, RwLockReadGuard};
  use crate::tokenizer::types::line::Line;
  use crate::tokenizer::read::nesting::lines::lineNesting;
  // ===============================================================================================

  /// Вспомогательная функция:
  /// Создание линий с заданным отступом
  fn createLine(indent: usize) -> Arc<RwLock<Line>>
  {
    Arc::new(RwLock::new(Line {
      tokens: None,
      indent: Some(indent),
      lines: None,
      parent: None
    }))
  }

  /// Вспомогательная функция:
  /// Рекурсивно разворачивает иерархию линий в плоский список пар (отступ, глубина вложения)
  fn flattenHierarchy(lines: &[Arc<RwLock<Line>>], depth: usize) -> Vec<(usize, usize)>
  {
    let mut result: Vec<(usize, usize)> = Vec::new();
    for lineArc in lines
    {
      let line: RwLockReadGuard<Line> = lineArc.read().unwrap();
      result.push((line.indent.unwrap_or(0), depth));

      if let Some(innerLines) = &line.lines
      {
        result.extend(flattenHierarchy(innerLines, depth + 1));
      }
    }
    result
  }

  /// Вспомогательная функция:
  /// Табличная проверка иерархии линий (отступ, ожидаемая глубина)
  fn checkHierarchy(lines: &[Arc<RwLock<Line>>], expected: &[(usize, usize)]) -> ()
  {
    let flat: Vec<(usize, usize)> = flattenHierarchy(lines, 0);
    assert_eq!(flat.len(), expected.len(), "Количество линий не совпадает с ожидаемым");

    //
    for (i, (actualIndent, actualDepth)) in flat.iter().enumerate()
    {
      let (expIndent, expDepth): &(usize, usize) = &expected[i];
      assert_eq!(actualIndent, expIndent, "Линия на позиции {}: несоответствие отступа", i);
      assert_eq!(actualDepth, expDepth, "Линия на позиции {}: несоответствие глубины", i);
    }
  }

  // ===============================================================================================

  /// Проверяет глубокое последовательное вложение линий (лесенка)
  #[test]
  fn deepNesting() -> ()
  {
    let mut linesLinks: Vec< Arc<RwLock<Line>> > = 
      vec![
        createLine(0),
        createLine(2),
        createLine(4),
      ];

    //
    lineNesting(&mut linesLinks);

    //
    checkHierarchy(&linesLinks, &[
      (0, 0), // Корень
      (2, 1), // Вложено в 0
      (4, 2), // Вложено в 2
    ]);
  }

  // ===============================================================================================

  /// Проверяет отсутствие вложений при одинаковом отступе
  #[test]
  fn noNesting() -> ()
  {
    let mut linesLinks: Vec< Arc<RwLock<Line>> > = 
      vec![
        createLine(0),
        createLine(0),
      ];

    //
    lineNesting(&mut linesLinks);

    //
    checkHierarchy(&linesLinks, &[
      (0, 0), // Корень 1
      (0, 0), // Корень 2
    ]);
  }

  // ===============================================================================================

  /// Проверяет смешанное вложение со сбросом отступа в корень
  #[test]
  fn mixedNesting() -> ()
  {
    let mut linesLinks: Vec< Arc<RwLock<Line>> > = 
      vec![
        createLine(0),
        createLine(2),
        createLine(0),
        createLine(4),
      ];

    //
    lineNesting(&mut linesLinks);

    //
    checkHierarchy(&linesLinks, &[
      (0, 0), // Корень 1
      (2, 1), // Вложено в первый 0
      (0, 0), // Корень 2 (сброс)
      (4, 1), // Вложено во второй 0
    ]);
  }

  // ===============================================================================================

  /// Проверяет частичный возврат отступа (сброс на один уровень назад)
  #[test]
  fn complexReturn() -> ()
  {
    let mut linesLinks: Vec< Arc<RwLock<Line>> > = vec![
      createLine(0),
      createLine(2),
      createLine(4),
      createLine(2), // Возврат на уровень 2
    ];

    //
    lineNesting(&mut linesLinks);

    //
    checkHierarchy(&linesLinks, &[
      (0, 0), // Корень
      (2, 1), // Вложено в 0 (первый узел)
      (4, 2), // Вложено в первый 2
      (2, 1), // Вложено в 0 (сосед первого 2)
    ]);
  }

  // ===============================================================================================
}

// =================================================================================================