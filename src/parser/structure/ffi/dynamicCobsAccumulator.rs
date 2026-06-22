
// =================================================================================================

use serde::de::DeserializeOwned;

/// Результат обработки очередной порции данных аккумулятором COBS
pub enum DynamicFeedResult<'a, T> 
{
  /// Данные поглощены, ждём завершения сообщения
  Consumed,
  /// Ошибка десериализации COBS или postcard
  DeserError(String),
  /// Успешно декодировано сообщение с остатком данных
  Success { data: T, remaining: &'a [u8] }
}

/// Аккумулятор для накопления и декодирования COBS-сообщений из потока;
/// Он динамического размера, чтобы мы могли не нарушать работу FFI потоков извне;
/// Поскольку они бы не смогли изменить runtime и не писали бы это - 
/// мы должны дать им это из коробки.
pub struct DynamicCobsAccumulator 
{
  /// Буфер сырых байт COBS
  rawBuffer: Vec<u8>,
  /// Буфер раскодированных данных
  decodedBuffer: Vec<u8>
}

impl DynamicCobsAccumulator 
{
  // ===============================================================================================

  /// Создаёт аккумулятор с начальной ёмкостью буферов (по умолчанию 4096 байт).
  pub fn new() -> Self {
    Self::withCapacity(4096)
  }

  /// Создаёт аккумулятор с заданной начальной ёмкостью.
  pub fn withCapacity(capacity: usize) -> Self 
  {
    Self {
      rawBuffer: Vec::with_capacity(capacity),
      decodedBuffer: Vec::with_capacity(capacity),
    }
  }

  // ===============================================================================================

  /// Очищает оба буфера для подготовки к новому сообщению
  pub fn clear(&mut self) 
  {
    self.rawBuffer.clear();
    self.decodedBuffer.clear();
  }

  // ===============================================================================================

  /// Подаёт порцию данных, пытается извлечь законченное COBS-сообщение
  pub fn feed<'a, T: DeserializeOwned>(&mut self, data: &'a [u8]) -> DynamicFeedResult<'a, T> 
  {
    for (i, &byte) in data.iter().enumerate() 
    {
      if byte == 0x00 
      { // Найден разделитель COBS-пакета
        if let Err(e) = Self::decodeCobs(&self.rawBuffer, &mut self.decodedBuffer) {
          self.clear();
          return DynamicFeedResult::DeserError(e);
        }
        // Очистка сырого буфера после декодирования
        self.rawBuffer.clear();

        match postcard::from_bytes::<T>(&self.decodedBuffer) 
        {
          Ok(val) => 
          { // Успешная десериализация сообщения
            return DynamicFeedResult::Success {
              data: val,
              remaining: &data[i + 1..],
            };
          }
          Err(e) => 
          { // Ошибка десериализации, полный сброс состояния
            self.clear();
            return DynamicFeedResult::DeserError(format!("{:?}", e));
          }
        }
      } else 
      { // Накопление байтов текущего сообщения;
        // Буфер автоматически расширится.
        self.rawBuffer.push(byte);
      }
    }
    //
    DynamicFeedResult::Consumed
  }

  // ===============================================================================================

  /// Декодирует COBS-последовательность в обычные байты
  fn decodeCobs(src: &[u8], dst: &mut Vec<u8>) -> Result<(), String> 
  {
    dst.clear(); // Очистка выходного буфера
    let mut srcIndex: usize = 0; // Текущая позиция чтения

    while srcIndex < src.len()
    { // Цикл по входным данным
      let code: usize = src[srcIndex] as usize; // Байт-код длины блока
      if code == 0 {
        return Err("Invalid COBS: zero byte in data".into()); // 0 запрещён в COBS
      }
      srcIndex += 1; // Переход к данным блока
      let end: usize = srcIndex + code - 1; // Граница текущего блока

      if end > src.len() {
        return Err("Invalid COBS: unexpected end".into()); // Выход за пределы буфера
      }

      dst.extend_from_slice(&src[srcIndex..end]); // Копирование блока данных
      srcIndex = end; // Переход к следующему коду

      if code < 0xFF && srcIndex < src.len() {
        dst.push(0); // Восстановление разделителя нуля
      }
    }

    Ok(())
  }
  
  // ===============================================================================================
}