/* /parser/bytes
*/

// =================================================================================================

/// Список байтов. Можно удобно конвертировать между другими типами данных.
#[derive(Clone)]
pub struct Bytes 
{
  /// Набор байтов или пустота
  data: Option< Vec<u8> >,
}

impl Bytes 
{
  /// Empty bytes
  pub fn empty() -> Self 
  {
    Self { data: None }
  }
  
  /// Создать из данных
  pub fn new<T: Into<Vec<u8>>>(data: T) -> Self 
  {
    let vec: Vec<u8> = data.into();
    match vec.is_empty() {
      true => Self { data: None },
      false => Self { data: Some(vec) },
    }
  }

  /// Получить данные
  pub fn getAll(&self) -> Option<&[u8]> 
  {
    self.data.as_deref()
  }

  /// Получает строку из байтов
  pub fn toString(&self) -> Option<String>
  {
    self.getAll()
      .and_then(|bytes| std::str::from_utf8(bytes).ok())
      .map(|s| s.to_string())
  }
}

// =================================================================================================

impl From<Vec<u8>> for Bytes 
{
  fn from(value: Vec<u8>) -> Self 
  {
    Bytes::new(value)
  }
}

impl From<&[u8]> for Bytes 
{
  fn from(value: &[u8]) -> Self 
  {
    Bytes::new(value)
  }
}

impl From<String> for Bytes 
{
  fn from(value: String) -> Self 
  {
    Bytes::new(value.into_bytes())
  }
}

impl From<&str> for Bytes 
{
  fn from(value: &str) -> Self 
  {
    Bytes::new(value.as_bytes())
  }
}

impl From< Option<Vec<u8>> > for Bytes 
{
  fn from(value: Option<Vec<u8>>) -> Self 
  {
    match value 
    {
      Some(data) if !data.is_empty() => Bytes { data: Some(data) },
      _ => Bytes::empty(),
    }
  }
}

// =================================================================================================