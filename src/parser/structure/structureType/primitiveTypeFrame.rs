use std::num::IntErrorKind;
use crate::parser::bytes::Bytes;
use crate::parser::structure::structureType::PrimitiveType;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Перечисляет все типы чисел для `min`/`max` границ примитива.
#[derive(Clone, Copy)]
pub enum PrimitiveTypeFrameNumber
{
  U8(u8),
  U16(u16),
  U32(u32),
  U64(u64),
  U128(u128),
  Usize(usize),

  I8(i8),
  I16(i16),
  I32(i32),
  I64(i64),
  I128(i128),
  Isize(isize),

  F32(f32),
  F64(f64)
}

/// Границы `примитива`, накладываются поверх для `PrimitiveType`;
/// Требуют `default` значения.
/// 
/// Только числовые типы имеют `min`/`max` значения.
#[derive(Clone, Copy)]
pub struct PrimitiveTypeFrame
{
  /// Нижняя граница значения; `None` - её нет.
  pub min: Option<PrimitiveTypeFrameNumber>,
  
  /// Верхняя граница значения; `None` - её нет.
  pub max: Option<PrimitiveTypeFrameNumber>,
  
  /// Значение по умолчанию. Всегда должно быть.
  pub default: (TokenType, &'static str)
}

// =================================================================================================

/// Макрос для границ примитива.
///
/// Например:
/// ```ignore
/// primitiveTypeFrame(min: 0, max: 255, default: (TokenType::UInt, "0"))
/// primitiveTypeFrame(default: (TokenType::String, ""))
/// ```
macro_rules! primitiveTypeFrame
{
  (@build $min:expr, $max:expr, $default:expr) =>
  {
    PrimitiveTypeFrame
    {
      min: $min,
      max: $max,
      default: $default
    }
  };
  
  (min: $min:expr, max: $max:expr, default: $default:expr) =>
  {
    primitiveTypeFrame!(@build Some($min), Some($max), $default)
  };
  (min: $min:expr, default: $default:expr) =>
  {
    primitiveTypeFrame!(@build Some($min), None, $default)
  };
  (max: $max:expr, default: $default:expr) =>
  {
    primitiveTypeFrame!(@build None, Some($max), $default)
  };
  (default: $default:expr) =>
  {
    primitiveTypeFrame!(@build None, None, $default)
  };
}

impl PrimitiveType
{
  // ===============================================================================================
  
  /// Рамка примитива: границы значения и значение по умолчанию.
  ///
  /// Здесь описаны все примитивы: и границы чисел, и значение по умолчанию.
  /// Чего не задали - того у примитива нет.
  pub fn getFrame(&self) -> PrimitiveTypeFrame
  {
    match self
    {
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      PrimitiveType::None => primitiveTypeFrame!(default: (TokenType::None, "")),
      PrimitiveType::Any  => primitiveTypeFrame!(default: (TokenType::None, "")),
      
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

      // todo По сути не должно быть `min`, потому что у него `default` всегда `0`,
      //  однако не понятно, ведь мб нужно для `range` в других местах.
      PrimitiveType::U8 => primitiveTypeFrame!(
        min: PrimitiveTypeFrameNumber::U8(u8::MIN),
        max: PrimitiveTypeFrameNumber::U8(u8::MAX),
        default: (TokenType::UInt, "0")
      ),
      // todo По сути не должно быть `min`, потому что у него `default` всегда `0`,
      //  однако не понятно, ведь мб нужно для `range` в других местах.
      PrimitiveType::U16 => primitiveTypeFrame!(
        min: PrimitiveTypeFrameNumber::U16(u16::MIN),
        max: PrimitiveTypeFrameNumber::U16(u16::MAX),
        default: (TokenType::UInt, "0")
      ),
      // todo По сути не должно быть `min`, потому что у него `default` всегда `0`,
      //  однако не понятно, ведь мб нужно для `range` в других местах.
      PrimitiveType::U32 => primitiveTypeFrame!(
        min: PrimitiveTypeFrameNumber::U32(u32::MIN),
        max: PrimitiveTypeFrameNumber::U32(u32::MAX),
        default: (TokenType::UInt, "0")
      ),
      // todo По сути не должно быть `min`, потому что у него `default` всегда `0`,
      //  однако не понятно, ведь мб нужно для `range` в других местах.
      PrimitiveType::U64 => primitiveTypeFrame!(
        min: PrimitiveTypeFrameNumber::U64(u64::MIN),
        max: PrimitiveTypeFrameNumber::U64(u64::MAX),
        default: (TokenType::UInt, "0")
      ),
      // todo По сути не должно быть `min`, потому что у него `default` всегда `0`,
      //  однако не понятно, ведь мб нужно для `range` в других местах.
      PrimitiveType::Usize => primitiveTypeFrame!(
        min: PrimitiveTypeFrameNumber::Usize(usize::MIN),
        max: PrimitiveTypeFrameNumber::Usize(usize::MAX),
        default: (TokenType::UInt, "0")
      ),
      
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      PrimitiveType::I8 => primitiveTypeFrame!(
        min: PrimitiveTypeFrameNumber::I8(i8::MIN),
        max: PrimitiveTypeFrameNumber::I8(i8::MAX),
        default: (TokenType::UInt, "0")
      ),
      PrimitiveType::I16 => primitiveTypeFrame!(
        min: PrimitiveTypeFrameNumber::I16(i16::MIN),
        max: PrimitiveTypeFrameNumber::I16(i16::MAX),
        default: (TokenType::UInt, "0")
      ),
      PrimitiveType::I32 => primitiveTypeFrame!(
        min: PrimitiveTypeFrameNumber::I32(i32::MIN),
        max: PrimitiveTypeFrameNumber::I32(i32::MAX),
        default: (TokenType::UInt, "0")
      ),
      PrimitiveType::I64 => primitiveTypeFrame!(
        min: PrimitiveTypeFrameNumber::I64(i64::MIN),
        max: PrimitiveTypeFrameNumber::I64(i64::MAX),
        default: (TokenType::UInt, "0")
      ),
      PrimitiveType::Isize => primitiveTypeFrame!(
        min: PrimitiveTypeFrameNumber::Isize(isize::MIN),
        max: PrimitiveTypeFrameNumber::Isize(isize::MAX),
        default: (TokenType::UInt, "0")
      ),
      
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      PrimitiveType::F32 => primitiveTypeFrame!(
        min: PrimitiveTypeFrameNumber::F32(f32::MIN),
        max: PrimitiveTypeFrameNumber::F32(f32::MAX),
        default: (TokenType::UFloat, "0.0")
      ),
      PrimitiveType::F64 => primitiveTypeFrame!(
        min: PrimitiveTypeFrameNumber::F64(f64::MIN),
        max: PrimitiveTypeFrameNumber::F64(f64::MAX),
        default: (TokenType::UFloat, "0.0")
      ),
      
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      PrimitiveType::String    => primitiveTypeFrame!(default: (TokenType::String, "")),
      PrimitiveType::CString   => primitiveTypeFrame!(default: (TokenType::String, "")), // todo Должен быть `String`, но могут быть ошибки сейчас.
      PrimitiveType::RawString => primitiveTypeFrame!(default: (TokenType::String, "")), // todo Должен быть `String`, но могут быть ошибки сейчас.

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      PrimitiveType::True  => primitiveTypeFrame!(default: (TokenType::True,  "True")), // todo Не уверен, что там такое значение в "", надо проверить токен.
      PrimitiveType::False => primitiveTypeFrame!(default: (TokenType::False, "False")), // todo Не уверен, что там такое значение в "", надо проверить токен.

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      PrimitiveType::Pointer => primitiveTypeFrame!(default: (TokenType::None, "")),

      PrimitiveType::Method => primitiveTypeFrame!(default: (TokenType::None, "")),
      PrimitiveType::List   => primitiveTypeFrame!(default: (TokenType::None, "")),

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      PrimitiveType::Custom(_) => primitiveTypeFrame!(default: (TokenType::None, ""))

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    }
  }
  
  // ===============================================================================================
}

// =================================================================================================

impl PrimitiveType
{
  pub(super) fn normalizeToken(
    &self,
    frame: &PrimitiveTypeFrame,
    token: &mut Token,
    dataType: &TokenType
  ) -> ()
  {
    // Получаем строку из данных.
    let tokenData: String =
      if let Some(tokenData) = token.getData().toString() {
        tokenData
      } else
      { // Нет данных.
        self.setTokenDefault(token);
        self.retypeToken(token);
        return;
      };
    
    match dataType
    {
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      TokenType::UInt =>
      {
        // Токен бесконечен, а `u64` нет: поэтому всё что больше - граница `u64`,
        // дальше значение зажимается в рамки структуры.
        let value: Option<u64> = match tokenData.parse::<u64>()
        {
          Ok(value) => Some(value),
          Err(error) if *error.kind() == IntErrorKind::PosOverflow => Some(u64::MAX),
          Err(_) => None
        };

        match value
        {
          None => self.setTokenDefault(token), // Не распарсилось — базовое значение.
          Some(value) =>
          {
            // Для `float` берём величину числа напрямую, без сжатия в `u64`.
            let magnitude: f64 = tokenData.parse::<f64>().ok().unwrap_or(value as f64);
            frame.normalizeTokenUnsigned(token, value, magnitude)
          }
        }
      }

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

      TokenType::Int =>
      {
        // Токен бесконечен, а `i64` нет: всё что больше или меньше - граница `i64`,
        // дальше значение зажимается в рамки структуры.
        let value: Option<i64> = match tokenData.parse::<i64>()
        { // todo Напоминает очень сильно границы `calculate`, получается дубль так то.
          Ok(value) => Some(value),
          Err(error) if *error.kind() == IntErrorKind::NegOverflow => Some(i64::MIN),
          Err(error) if *error.kind() == IntErrorKind::PosOverflow => Some(i64::MAX),
          Err(_) => None
        };

        match value
        {
          None => self.setTokenDefault(token), // Не распарсилось — базовое значение.
          Some(value) =>
          {
            // Для `float` берём величину числа напрямую, без сжатия в `i64`.
            let magnitude: f64 = tokenData.parse::<f64>().ok().unwrap_or(value as f64);
            frame.normalizeTokenSigned(token, value, magnitude)
          }
        }
      }

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

      TokenType::UFloat | TokenType::Float =>
      {
        if let Ok(mut value) = tokenData.parse::<f64>()
        {
          // Для `UFloat` обрезаем отрицательные до `0`.
          if dataType == &TokenType::UFloat && value < 0.0 {
            value = 0.0;
          }

          if value.is_nan()
          { // `NaN` — базовое значение;
            //
            // Бесконечность зажмётся в границы типа.
            self.setTokenDefault(token);
            self.retypeToken(token);
            return;
          }

          frame.normalizeTokenFloat(token, value);
        } else
        { // Не распарсилось — базовое значение.
          self.setTokenDefault(token);
        }
      }

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

      _ =>
      { // Здесь пытаются прировнять что-то не то.
        self.setTokenDefault(token);
      }

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    }
  }

  // ===============================================================================================

}

// =================================================================================================

impl PrimitiveTypeFrame
{
  // ===============================================================================================

  /// `true`, если рамка дробная (`f32`/`f64`):
  /// её значения зажимаются величиной числа, а не точным целым.
  fn isFloat(&self) -> bool
  {
    matches!(
      self.min,
      Some(PrimitiveTypeFrameNumber::F32(_)) | Some(PrimitiveTypeFrameNumber::F64(_))
    )
  }

  // ===============================================================================================

  /// Границы целочисленной рамки: `(минимум, максимум)`.
  ///
  /// У `float`-рамок целочисленных границ нет - будет `None`:
  /// их значения зажимаются величиной в границы `f32`/`f64`.
  //
  //  `U128`/`I128` рамок пока нет: их границы не вмещаются в
  //  `i64`/`u64`, а молча исказить границу нельзя - поэтому `None`.
  fn integerBounds(&self) -> Option<(i64, u64)>
  {
    match (self.min, self.max)
    {
      (Some(PrimitiveTypeFrameNumber::U8(min)),    Some(PrimitiveTypeFrameNumber::U8(max)))    => Some((min as i64, max as u64)),
      (Some(PrimitiveTypeFrameNumber::U16(min)),   Some(PrimitiveTypeFrameNumber::U16(max)))   => Some((min as i64, max as u64)),
      (Some(PrimitiveTypeFrameNumber::U32(min)),   Some(PrimitiveTypeFrameNumber::U32(max)))   => Some((min as i64, max as u64)),
      (Some(PrimitiveTypeFrameNumber::U64(min)),   Some(PrimitiveTypeFrameNumber::U64(max)))   => Some((min as i64, max as u64)),
      (Some(PrimitiveTypeFrameNumber::Usize(min)), Some(PrimitiveTypeFrameNumber::Usize(max))) => Some((min as i64, max as u64)),
      //
      (Some(PrimitiveTypeFrameNumber::I8(min)),    Some(PrimitiveTypeFrameNumber::I8(max)))    => Some((min as i64, max as u64)),
      (Some(PrimitiveTypeFrameNumber::I16(min)),   Some(PrimitiveTypeFrameNumber::I16(max)))   => Some((min as i64, max as u64)),
      (Some(PrimitiveTypeFrameNumber::I32(min)),   Some(PrimitiveTypeFrameNumber::I32(max)))   => Some((min as i64, max as u64)),
      (Some(PrimitiveTypeFrameNumber::I64(min)),   Some(PrimitiveTypeFrameNumber::I64(max)))   => Some((min as i64, max as u64)),
      (Some(PrimitiveTypeFrameNumber::Isize(min)), Some(PrimitiveTypeFrameNumber::Isize(max))) => Some((min as i64, max as u64)),
      //
      _ => None
    }
  }

  // ===============================================================================================

  /// Кладёт величину числа в токен как значение `float`-рамки.
  ///
  /// Величина зажимается в `min`/`max` самой рамки: иначе бесконечность
  /// осталась бы бесконечной и в `f32`/`f64`.
  fn setFloatToken(&self, token: &mut Token, magnitude: f64) -> ()
  {
    // `{:?}` для `f32`/`f64` сохраняет точку для целых `float`:
    // `1.0` -> `1.0`, не `1`; и использует `e`-нотацию только для
    // больших/маленьких чисел. Так `data` всегда одинаково проходит
    // через токенайзер как `Float`/`UFloat`.
    match (self.min, self.max)
    {
      (Some(PrimitiveTypeFrameNumber::F32(min)), Some(PrimitiveTypeFrameNumber::F32(max))) =>
      {
        let floatValue: f64 = magnitude.clamp(min as f64, max as f64);
        token.setData( Bytes::from(format!("{:?}", floatValue as f32)) );
      }
      //
      (Some(PrimitiveTypeFrameNumber::F64(min)), Some(PrimitiveTypeFrameNumber::F64(max))) =>
      {
        let floatValue: f64 = magnitude.clamp(min, max);
        token.setData( Bytes::from(format!("{:?}", floatValue)) );
      }
      //
      _ => {}
    }
  }

  // ===============================================================================================

  /// Зажимает беззнаковое целое из токена в рамки примитива.
  ///
  /// `value` - точное значение, насыщенное в границы `u64`:
  /// токен бесконечен, а `u64` нет - всё что больше, становится границей;
  ///
  /// `magnitude` - величина числа из данных токена: для `float`-рамок число
  /// берётся из неё, а не из насыщенного целого - иначе большие числа теряют
  /// величину ещё до рамки (`44444...` зажалось бы в `u64::MAX`, а не в `f32::MAX`).
  pub(super) fn normalizeTokenUnsigned(&self, token: &mut Token, value: u64, magnitude: f64) -> ()
  {
    // `float`-рамка: значение зажимается величиной в границы `f32`/`f64`.
    if self.isFloat()
    {
      self.setFloatToken(token, magnitude);
      return;
    }

    // Целочисленная рамка: точное значение зажимается в максимум.
    //
    // Минимум не нужен: беззнаковое значение всегда `>= 0`,
    // а минимум целочисленной рамки положительным не бывает.
    if let Some((_, max)) = self.integerBounds()
    {
      token.setData( Bytes::from(value.min(max).to_string()) );
    }

    // Рамки нет: значение проходит как есть.
  }

  // ===============================================================================================

  /// Зажимает знаковое целое из токена в рамки примитива.
  ///
  /// `value` - точное значение, насыщенное в границы `i64`;
  /// `magnitude` - величина числа из данных токена, как у беззнакового.
  pub(super) fn normalizeTokenSigned(&self, token: &mut Token, value: i64, magnitude: f64) -> ()
  {
    // `float`-рамка: значение зажимается величиной в границы `f32`/`f64`.
    if self.isFloat()
    {
      self.setFloatToken(token, magnitude);
      return;
    }

    if let Some((min, max)) = self.integerBounds()
    {
      let normalized: String =
        if value < min
        {
          min.to_string()
        }
        else if value > 0 && (value as u64) > max
        {
          max.to_string()
        }
        else
        {
          value.to_string()
        };
      token.setData( Bytes::from(normalized) );
    }

    // Рамки нет: значение проходит как есть.
  }

  // ===============================================================================================

  /// Зажимает дробное значение из токена в рамки примитива.
  pub(super) fn normalizeTokenFloat(&self, token: &mut Token, value: f64) -> ()
  {
    // `float`-рамка: значение зажимается величиной в границы `f32`/`f64`.
    if self.isFloat()
    {
      self.setFloatToken(token, value);
      return;
    }

    if let Some((min, max)) = self.integerBounds()
    {
      // Приведение к целочисленному типу: округление и зажим в границы типа;
      //
      // Отрицательное значение остаётся, если тип его принимает
      // (`I8 = -5.5` -> `-6`), а для беззнаковых типов оно становится `0`
      // (`U8 = -5.5` -> `0`).
      let rounded: f64 = value.round();
      let normalized: String = if rounded < 0.0
      { // Приведение `f64` в `i64` насыщается само, `inf` тоже.
        // todo Тут упомянут `inf` - нужно разобраться т.к. его нет в RTS.
        (rounded as i64).max(min).to_string()
      } else
      { // todo Тут бы desc как сделано в ветке сверху?
        (rounded as u64).min(max).to_string()
      };
      token.setData( Bytes::from(normalized) );
    }

    // Рамки нет: значение проходит как есть.
  }

  // ===============================================================================================
}

// =================================================================================================
