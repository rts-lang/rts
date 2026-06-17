use std::fmt;
use crate::parser::bytes::Bytes;
use crate::tokenizer::read::primitives::words::keywords;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Элементарная единица хранения информации;
/// Представляет strings, numbers, operators и так далее.
#[derive(Clone)]
pub struct Token 
{
  /// Данные единицы хранения
  /// todo В будущем лучше будет хранить более бинарно? (но это сломает абстрактные операции с Token)
  data: Bytes,
  /// Тип данных единицы хранения
  dataType: TokenType,
  /// Набор вложенных единиц хранения
  pub lines: Option< Vec<Line> >,
  
  /// Начало токена для analyzer
  #[cfg(feature = "analyzer")]
  pub start: usize,
  /// Конец токена для analyzer
  #[cfg(feature = "analyzer")]
  pub end: usize
}
impl Token 
{
  /// Обычное создание
  pub fn new<T: Into<Bytes>>(
    dataType: TokenType,
    data:     T,
  ) -> Self {
    Token {
      data: data.into(),
      dataType,
      lines: None,
      #[cfg(feature = "analyzer")]
      start: 0,
      #[cfg(feature = "analyzer")]
      end: 0
    }
  }
  
  /// Пустой, но имеет тип данных
  pub fn newEmpty(
    dataType: TokenType
  ) -> Self 
  {
    Token 
    {
      data: Bytes::empty(),
      dataType,
      lines: None,
      #[cfg(feature = "analyzer")]
      start: 0,
      #[cfg(feature = "analyzer")]
      end: 0
    }
  }
  /// Пустой, но выполняет роль держателя вложения
  pub fn newNesting(
    lines: Vec<Line>
  ) -> Self
  {
    Token
    {
      data: Bytes::empty(),
      dataType: TokenType::None,
      lines: Some(lines),
      #[cfg(feature = "analyzer")]
      start: 0,
      #[cfg(feature = "analyzer")]
      end: 0
    }
  }

  // convert data
  // todo: фиг его знает что это за ерунда,
  // но смысл такой, что если тип был Int или Float, 
  // а ожидается UInt или UFloat, то понятно,
  // что результат будет 0
  fn convertData(&mut self) -> ()
  {
    match self.data.toString()
    {
      None => {}
      Some(data) => 
      {
        match data.chars().nth(0)  
        {
          Some('-') => 
          {
            match self.dataType
            {
              TokenType::UInt  => { self.data = Bytes::from(String::from("0")); }
              TokenType::Float => { self.data = Bytes::from(String::from("0.0")); } // todo: use . (0.0)
              _ => { }
            }
          }
          _ => {}
        }
        //
      }
    }
    //
  }

  /// Получает тип данных
  pub fn getDataType(&self) -> &TokenType
  {
    &self.dataType
  }
  /// Устанавливает тип данных
  pub fn setDataType(&mut self, newDataType: TokenType) -> ()
  {
    self.dataType = newDataType;
    self.convertData();
  }

  /// Проверяет примитивный это токен или нет
  pub fn isPrimitive(&self) -> bool
  {
    keywords.iter().any(|(_, tt)| *tt == self.dataType)
  }

  /// Получает данные
  pub fn getData(&self) -> Bytes
  {
    self.data.clone()
  }
  /// Устанавливает данные
  pub fn setData<T: Into<Bytes>>(&mut self, newData: T) 
  {
    self.data = newData.into();
    self.convertData();
  }
}

impl fmt::Display for Token
{ // todo: debug only ?
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    match self.data.getAll()
    {
      Some(data) =>
      { // Есть данные - печатаем как символы
        write!(f, "{}", std::str::from_utf8(&data).unwrap_or_default())
      }
      None =>
      { // Данных нет - печатаем тип
        write!(f, "{}", self.getDataType().to_string())
      }
    }
    //
  }
}

impl fmt::Debug for Token
{ // todo: debug only ?
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
  {
    match self.data.getAll()
    {
      Some(data) =>
      { // Есть данные - печатаем как символы
        write!(f, "{}", std::str::from_utf8(&data).unwrap_or_default())
      }
      None =>
      { // Данных нет - печатаем тип
        write!(f, "{}", self.getDataType().to_string())
      }
    }
    //
  }
}

// =================================================================================================