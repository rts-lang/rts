
// =================================================================================================

#[cfg(not(feature = "analyzer"))]
/// Проверяет buffer по index и так пропускаем возможные комментарии;
/// Потом они будут удалены по меткам
pub fn deleteComment(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> ()
{
  *index += 1;
  while *index < *bufferLength && buffer[*index] != b'\n' 
  {
    *index += 1;
  }
}

// =================================================================================================

// todo desk
#[cfg(feature = "analyzer")]
pub fn deleteComments(buffer: &[u8], index: &mut usize, bufferLength: &usize, startIndent: &usize) -> () 
{
  // 1. Пропускаем первую строку комментария до конца строки или буфера
  while *index < *bufferLength && buffer[*index] != b'\n' {
    *index += 1;
  }

  // Если достигли конца буфера, то комментарий закончился, выходим
  if *index >= *bufferLength {
    return;
  }

  // Теперь *index указывает на '\n' (или конец буфера, но мы проверили)
  loop {
    let newlinePosition: usize = *index; // позиция '\n'
    // Пропускаем '\n' только для проверки следующих строк, но не сдвигаем индекс навсегда, если продолжения нет
    let mut nextIndex: usize = *index + 1;
    if nextIndex >= *bufferLength {
      // Нет следующей строки, оставляем индекс на '\n'
      *index = newlinePosition;
      break;
    }

    let mut nextIndent: usize = 0;
    // Считаем пробелы в начале следующей строки
    while nextIndex < *bufferLength && buffer[nextIndex] == b' ' {
      nextIndent += 1;
      nextIndex += 1;
    }

    if nextIndent > *startIndent {
      // Это продолжение комментария: пропускаем всю строку до конца
      *index = nextIndex;
      while *index < *bufferLength && buffer[*index] != b'\n' {
        *index += 1;
      }
      // Если достигли конца буфера, выходим, иначе продолжаем loop
      if *index >= *bufferLength {
        break;
      }
      // Иначе *index указывает на '\n' следующей строки, продолжим loop
    } else {
      // Комментарий закончился, возвращаем индекс на исходный '\n'
      *index = newlinePosition;
      break;
    }
  }
  //
}

// =================================================================================================

#[cfg(test)]
mod tests
{
  #[cfg(not(feature = "analyzer"))]
  use crate::tokenizer::read::comments::{deleteComment};
  #[cfg(feature = "analyzer")]
  use crate::tokenizer::read::comments::deleteComments;
  // ===============================================================================================
  
  /// todo desk
  #[test]
  #[cfg(feature = "analyzer")]
  fn singleLine()
  {
    for (input, expectedIndex, indent) in vec![
      ( "# short\nrest ", 7, 0),
      ( "# end", 5, 0),
      ( "# mid\n", 5, 0) // тут не 6 т.к. \n не символ
    ] {
      let buffer:  &[u8] = input.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;
      
      //
      deleteComments(buffer,  &mut index,  &bufferLength, &indent);
      
      //
      assert_eq!(
        index,
        expectedIndex,
        "Для '{}' индекс должен остановиться на {}, получен {}",
        input,
        expectedIndex,
        index
      );
    }
    //
  }
  /// todo desk
  #[test]
  #[cfg(feature = "analyzer")]
  fn multiLine()
  {
    for (input, expectedIndex, indent) in vec![
      ( "# base\n  cont\nend ", 13, 0), // тут не 14 т.к. \n не символ
      ( "# a\n# b\n", 3, 0)
    ] {
      let buffer:  &[u8] = input.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;
      
      //
      deleteComments(buffer,  &mut index,  &bufferLength, &indent);
      
      //
      assert_eq!(
        index,
        expectedIndex,
        "Для '{}' индекс {} != {}",
        input,
        expectedIndex,
        index
      );
    }
    //
  }
  
  // ===============================================================================================
  
  /// todo desk
  #[test]
  #[cfg(not(feature = "analyzer"))]
  fn singleLine()
  {
    for (input, expectedIndex) in vec![
      ( "# short\nrest ", 7),
      ( "# end", 5),
      ( "# mid\n", 5) // тут не 6 т.к. \n не символ
    ] {
      let buffer:  &[u8] = input.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;
      
      //
      deleteComment(buffer,  &mut index,  &bufferLength);
      
      //
      assert_eq!(
        index,
        expectedIndex,
        "Для '{}' индекс должен остановиться на {}, получен {}",
        input,
        expectedIndex,
        index
      );
    }
    //
  }
  
  // ===============================================================================================
}

// =================================================================================================