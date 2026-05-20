use std::sync::{Arc, RwLock, RwLockWriteGuard};
use crate::tokenizer::types::line::Line;
// =================================================================================================

#[cfg(not(feature = "analyzer"))]
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