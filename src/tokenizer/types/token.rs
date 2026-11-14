use std::fmt;
use std::sync::{Arc, RwLock};
use crate::parser::bytes::Bytes;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Элементарная единица хранения информации;
/// 
/// Представляет strings, numbers, operators и так далее.
#[derive(Clone)]
pub struct Token
{
  // ===============================================================================================
  
  /// Данные единицы хранения
  // 
  // todo В будущем лучше будет хранить более бинарно? (но это сломает абстрактные операции с Token).
  // 
  // todo Кстати мб хранить Option<Bytes>?
  data: Bytes,
  
  /// Тип данных единицы хранения
  dataType: TokenType,
  
  /// Набор вложенных единиц хранения
  pub lines: Option< Vec< Arc<RwLock<Line>> > >,
  
  // ===============================================================================================
  
  /// Начало токена для analyzer
  #[cfg(feature = "analyzer")]
  pub start: usize,
  /// Конец токена для analyzer
  #[cfg(feature = "analyzer")]
  pub end: usize

  // ===============================================================================================
}

impl Token
{
  /// Обычное создание.
  pub fn new<T: Into<Bytes>>(dataType: TokenType, data: T) -> Self
  {
    Self
    {
      data: data.into(),
      dataType,
      lines: None,
      #[cfg(feature = "analyzer")]
      start: 0,
      #[cfg(feature = "analyzer")]
      end: 0
    }
  }

  // ===============================================================================================
  
  /// Пустой, но имеет тип данных.
  pub const fn newEmpty(dataType: TokenType) -> Self
  {
    Self
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
  
  /// Пустой, но выполняет роль держателя вложения.
  pub const fn newNesting(lines: Vec< Arc<RwLock<Line>> >) -> Self
  {
    Self
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

  // ===============================================================================================

  /// Глубокое клонирование: `data` и `dataType` по значению,
  /// nested lines — рекурсивно.
  pub fn deepClone(&self) -> Self
  {
    Self
    {
      data: self.data.clone(),
      dataType: self.dataType.clone(),
      lines: self.lines.as_ref().map(|lines|
        lines.iter()
          .map(|lineLink| Arc::new(RwLock::new(lineLink.read().unwrap().deepClone())))
          .collect()
      ),
      #[cfg(feature = "analyzer")]
      start: self.start,
      #[cfg(feature = "analyzer")]
      end: self.end
    }
  }

  // ===============================================================================================
  
  // todo: В общем, это был метод, который вручную менял данные под границы текущего `TokenType`.
  //  Например если `TokenType::UInt` и данные стали отрицательными, а ожидались другие данные,
  //  то константное поведение сделает `0` и так далее в зависимости от типов.
  // 
  // todo: При этом `setData()` и `setDataType()` раздельные вещи. Поэтому в `parser`
  //  есть участки кода где они пишутся друг за другом.
  const fn convertData(&self) -> ()
  {
    return;
    /*
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
              TokenType::UInt => self.data = Bytes::from(String::from("0")),
              TokenType::UFloat => self.data = Bytes::from(String::from("0.0")),
              _ => {}
            }
          }
          _ => {}
        }
        //
      }
    }
    */
  }

  // ===============================================================================================

  /// Получает тип данных.
  pub const fn getDataType(&self) -> &TokenType
  {
    &self.dataType
  }
  
  /// Устанавливает тип данных.
  pub const fn setDataType(&mut self, newDataType: TokenType) -> ()
  {
    self.dataType = newDataType;
    self.convertData();
  }

  // ===============================================================================================

  /// Получает данные.
  pub fn getData(&self) -> Bytes
  {
    self.data.clone()
  }
  
  /// Устанавливает данные.
  pub fn setData<T: Into<Bytes>>(&mut self, newData: T)  -> ()
  {
    self.data = newData.into();
    self.convertData();
  }
  
  // ===============================================================================================
}

impl fmt::Display for Token
{ // todo: debug only ?
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    if let Some(data) = self.data.getAll()
    { // Есть данные - печатаем как символы.
      write!(f, "{}", std::str::from_utf8(data).unwrap_or_default())
    } else
    { // Данных нет - печатаем тип.
      write!(f, "{}", self.getDataType().to_string())
    }
    //
  }
}

impl fmt::Debug for Token
{ // todo: debug only ?
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
  {
    if let Some(data) = self.data.getAll()
    { // Есть данные - печатаем как символы.
      write!(f, "{}", std::str::from_utf8(data).unwrap_or_default())
    } else
    { // Данных нет - печатаем тип.
      write!(f, "{}", self.getDataType().to_string())
    }
    //
  }
}

// =================================================================================================
