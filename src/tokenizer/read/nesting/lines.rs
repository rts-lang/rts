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

  /// Вспомогательная функция для создания линий с заданным отступом
  fn createLine(indent: usize) -> Arc<RwLock<Line>>
  {
    Arc::new(RwLock::new(Line {
      tokens: None,
      indent: Some(indent),
      lines: None,
      parent: None
    }))
  }

  // ===============================================================================================

  /// todo desk
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
    assert_eq!(linesLinks.len(), 1, "Ожидается 1 корневая линия");

    //
    let level0: RwLockReadGuard<Line> = linesLinks[0].read().unwrap();
    let level0Lines: &Vec< Arc<RwLock<Line>> > = level0.lines.as_ref().expect("Ожидается вложение 1 уровня");
    assert_eq!(level0Lines.len(), 1, "Ожидается 1 дочерняя линия");
    assert_eq!(level0Lines[0].read().unwrap().indent.unwrap(), 2);
    assert!(level0Lines[0].read().unwrap().parent.is_some(), "Ожидается ссылка на родителя");

    //
    let level1: RwLockReadGuard<Line> = level0Lines[0].read().unwrap();
    let level1Lines: &Vec< Arc<RwLock<Line>> > = level1.lines.as_ref().expect("Ожидается вложение 2 уровня");
    assert_eq!(level1Lines.len(), 1, "Ожидается 1 дочерняя линия");
    assert_eq!(level1Lines[0].read().unwrap().indent.unwrap(), 4);
    assert!(level1Lines[0].read().unwrap().parent.is_some(), "Ожидается ссылка на родителя");
  }

  // ===============================================================================================

  /// todo desk
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
    assert_eq!(linesLinks.len(), 2, "Ожидается 2 корневые линии");
    
    //
    let l0: RwLockReadGuard<Line> = linesLinks[0].read().unwrap();
    let l1: RwLockReadGuard<Line> = linesLinks[1].read().unwrap();
    assert!(l0.lines.is_none(), "Вложений быть не должно");
    assert!(l1.lines.is_none(), "Вложений быть не должно");
    assert!(l0.parent.is_none(), "Родителя быть не должно");
    assert!(l1.parent.is_none(), "Родителя быть не должно");
  }

  // ===============================================================================================

  /// todo desk
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
    assert_eq!(linesLinks.len(), 2, "Ожидается 2 корневые линии из-за сброса отступа");

    //
    let root1: RwLockReadGuard<Line> = linesLinks[0].read().unwrap();
    let root1Lines: &Vec< Arc<RwLock<Line>> > = root1.lines.as_ref().expect("Ожидается вложение у первой линии");
    assert_eq!(root1Lines.len(), 1, "Ожидается 1 дочерняя линия");
    assert_eq!(root1Lines[0].read().unwrap().indent.unwrap(), 2);

    //
    let root2: RwLockReadGuard<Line> = linesLinks[1].read().unwrap();
    let root2Lines: &Vec< Arc<RwLock<Line>> > = root2.lines.as_ref().expect("Ожидается вложение у второй линии");
    assert_eq!(root2Lines.len(), 1, "Ожидается 1 дочерняя линия");
    assert_eq!(root2Lines[0].read().unwrap().indent.unwrap(), 4);
  }

  // ===============================================================================================
}

// =================================================================================================