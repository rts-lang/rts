// =================================================================================================

/// Считает количество подряд идущих `#`, начиная с buffer[index];
/// 
/// Ограничено 3 - это максимальный уровень метки комментария
fn hashRunLength(buffer: &[u8], index: usize, bufferLength: usize) -> usize
{
  let mut length: usize = 0;
  while length < 3 && index+length < bufferLength && buffer[index+length] == b'#'
  {
    length += 1;
  }
  length
}

/// Пропускает комментарий любого уровня, начиная с buffer[*index] == b'#';
/// Определяет уровень по количеству `#` и передаёт чтение дальше
///
/// `#` - до конца строки строго; либо до `##`/`###`, если они встретились раньше
/// 
/// `##` - `\n` игнорируется, идёт до закрывающей `##`; либо до `#` или `###`
/// 
/// `###` - `\n`, `#` и `##` внутри игнорируются, идёт строго до закрывающей `###`;
/// удобно для комментирования больших участков кода с комментами внутри
pub fn deleteComment(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> ()
{
  let level: usize = hashRunLength(buffer, *index, *bufferLength);
  *index += level; // Пропускаем открывающую метку

  match level
  {
    1 => deleteSingleComment(buffer, index, bufferLength),
    2 => deleteDoubleComment(buffer, index, bufferLength),
    _ => deleteTripleComment(buffer, index, bufferLength),
  }
  //
}

/// `#` - идёт строго до конца строки;
/// 
/// прерывается раньше, если встретил `##` или `###` - они не потребляются
fn deleteSingleComment(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> ()
{
  while *index < *bufferLength && buffer[*index] != b'\n'
  {
    if buffer[*index] == b'#' && hashRunLength(buffer, *index, *bufferLength) >= 2
    { // Началась ## или ### - строчный комментарий обрываем здесь
      return;
    }
    *index += 1;
  }
  //
}

/// `##` - `\n` игнорируется, читает до закрывающей `##`;
/// 
/// на одиночном `#` или на `###` обрывается раньше - они не потребляются
fn deleteDoubleComment(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> ()
{
  while *index < *bufferLength
  {
    if buffer[*index] == b'#'
    {
      match hashRunLength(buffer, *index, *bufferLength)
      {
        2 => { *index += 2; return; } // Нашли закрывающую ## - потребляем
        _ => { return; }              // Одиночный # или ### - обрыв без потребления
      }
    }
    *index += 1;
  }
  //
}

/// `###` - `\n`, `#` и `##` внутри игнорируются;
/// 
/// читает строго до закрывающей `###`
fn deleteTripleComment(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> ()
{
  while *index < *bufferLength
  {
    if buffer[*index] == b'#' && hashRunLength(buffer, *index, *bufferLength) >= 3
    { // Нашли закрывающую ### - потребляем
      *index += 3;
      return;
    }
    *index += 1;
  }
  //
}

// =================================================================================================

#[cfg(test)]
mod tests
{
  use crate::tokenizer::read::primitives::comments::deleteComment;
  // ===============================================================================================

  /// Табличная проверка: buffer, ожидаемый index после чтения
  fn checkCases(cases: Vec<(&str, usize)>) -> ()
  {
    for (input, expectedIndex) in cases
    {
      let buffer: &[u8] = input.as_bytes();
      let bufferLength: usize = buffer.len();
      let mut index: usize = 0;

      //
      deleteComment(buffer, &mut index, &bufferLength);

      //
      assert_eq!(
        index, expectedIndex,
        "Для '{}' индекс должен быть {}, получен {}", input, expectedIndex, index
      );
    }
  }

  // ===============================================================================================

  /// `#` идёт строго до конца строки
  #[test]
  fn single() -> ()
  {
    checkCases(vec![
      ("# short\nrest ", 7), // Остановка перед \n
      ("# end", 5), // Остановка в конце буфера
    ]);
  }

  /// `#` обрывается раньше на `##` или `###`, не потребляя их
  #[test]
  fn singleInterruptedByHigherLevel() -> ()
  {
    checkCases(vec![
      ("# text ## more\n", 7),  // "# text " -> обрыв на ##
      ("# text ### more\n", 7), // "# text " -> обрыв на ###
    ]);
  }

  // ===============================================================================================

  /// `##` игнорирует \n и идёт до закрывающей ##
  #[test]
  fn double() -> ()
  {
    checkCases(vec![
      ("## a\nb\nc ##rest", 11), // Потребляет закрывающую ##
      ("## unterminated", 15), // До конца буфера, если нет закрытия
    ]);
  }

  /// `##` обрывается раньше на одиночном `#` или на `###`, не потребляя их
  #[test]
  fn doubleInterruptedByOtherLevel() -> ()
  {
    checkCases(vec![
      ("## a\n# rest", 5), // Обрыв на одиночном #
      ("## a\n### rest", 5), // Обрыв на ###
    ]);
  }

  // ===============================================================================================

  /// `###` игнорирует \n, # и ##, идёт строго до закрывающей ###
  #[test]
  fn triple() -> ()
  {
    checkCases(vec![
      ("### a\n# b\n## c\n###rest", 18), // # и ## внутри игнорируются
      ( "### unterminated", 16), // До конца буфера, если нет закрытия
    ]);
  }

  // ===============================================================================================
}

// =================================================================================================
