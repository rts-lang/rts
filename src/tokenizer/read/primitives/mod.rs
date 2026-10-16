pub mod numbers;
pub mod words;
pub mod operators;
pub mod quotes;
pub mod comments;
// =================================================================================================

/// Вспомогательный метод: 
///
/// Пропускает все байты из заданного набора `whitespace` и обновляет индекс;
pub fn skipWhitespaceBytes(
  buffer: &[u8],
  index: &mut usize,
  bufferLength: usize,
  whitespace: &[u8],
) {
  while *index < bufferLength && whitespace.contains(&buffer[*index]) {
    *index += 1;
  }
}

// =================================================================================================
