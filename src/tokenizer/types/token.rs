use std::fmt;
use std::num::IntErrorKind;
use std::sync::{Arc, RwLock};
use crate::parser::bytes::Bytes;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Элементарная единица хранения информации;
/// 
/// Представляет strings, numbers, operators и так далее.
/// 
// todo Как одна из интересных идей - сделать `enum` для типов токенов и тогда
//  можно будет делать токены не со всеми полями. Это может быть гибридной структурой.
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
  // ===============================================================================================
  
  /// Обычное создание.
  pub fn new<T: Into<Bytes>>(dataType: TokenType, data: T) -> Self
  {
    let mut token: Self = Self
    {
      data: data.into(),
      dataType,
      lines: None,
      //
      #[cfg(feature = "analyzer")]
      start: 0,
      #[cfg(feature = "analyzer")]
      end: 0
    };

    // Сразу нормализируем данные.
    token.normalizeData();
    token
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
      //
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
      //
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
      //
      #[cfg(feature = "analyzer")]
      start: self.start,
      #[cfg(feature = "analyzer")]
      end: self.end
    }
  }

  // ===============================================================================================

  /// Нормализует `data` под текущий `TokenType` в Rust границы.
  ///
  /// `Token` бесконечен т.е. его `data` состоит из байтов, а Rust нет.
  fn normalizeData(&mut self) -> ()
  {
    let Some(data) = self.data.toString() else { return };
    
    let normalized: Option<String> = match self.dataType
    {
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      TokenType::UInt => match data.parse::<u64>()
      {
        Err(error) if *error.kind() == IntErrorKind::PosOverflow => Some(u64::MAX.to_string()),
        Err(error) if *error.kind() == IntErrorKind::InvalidDigit && data.starts_with('-') =>
          Some(String::from("0")),
        _ => None
      },

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      TokenType::Int => match data.parse::<i64>()
      {
        Err(error) if *error.kind() == IntErrorKind::PosOverflow => Some(i64::MAX.to_string()),
        Err(error) if *error.kind() == IntErrorKind::NegOverflow => Some(i64::MIN.to_string()),
        _ => None
      },

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      TokenType::UFloat | TokenType::Float => match data.parse::<f64>()
      {
        Ok(value) if value.is_nan() => Some(String::from("0")),
        Ok(value) if self.dataType == TokenType::UFloat && value < 0.0 =>Some(String::from("0.0")),
        Ok(value) if value.is_infinite() =>
          Some(format!("{:e}", if value.is_sign_negative() { f64::MIN } else { f64::MAX })),
        _ => None
      },

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      _ => None
      
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    };
    
    if let Some(normalized) = normalized
    { // Пишем напрямую, без `setData`: иначе нормализация запустится снова.
      self.data = Bytes::from(normalized);
    }
  }

  // ===============================================================================================

  /// Получает тип данных.
  pub const fn getDataType(&self) -> &TokenType
  {
    &self.dataType
  }
  
  /// Устанавливает тип данных.
  pub fn setDataType(&mut self, newDataType: TokenType) -> ()
  {
    self.dataType = newDataType;
    self.normalizeData();
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
    self.normalizeData();
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
