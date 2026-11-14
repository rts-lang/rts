use crate::parser::structure::structureType::PrimitiveType;
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

/*
impl PrimitiveTypeFrame
{
  // ===============================================================================================
  
  /// Положить число в рамку.
  ///
  /// `number` - прочитанное по виду данных токена число.
  fn normalizeTokenNumber(&self, token: &mut Token, number: Number) -> ()
  {
    match *self
    {
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      CastFrame::Integer(min, max) => match number
      {
        Number::Unsigned(value, _) =>
          token.setData( Bytes::from(value.min(max).to_string()) ),
        //
        Number::Signed(value, _) =>
        {
          let normalized: String =
            if value < min {
              min.to_string()
            } else if value > 0 && (value as u64) > max {
              max.to_string()
            } else {
              value.to_string()
            };
          token.setData( Bytes::from(normalized) );
        }
        //
        Number::Float(value) =>
        {
          // Приведение к целочисленным типам: округление и зажим в границы типа;
          //
          // Отрицательное значение остаётся, если тип его принимает (`I8 = -5.5` -> `-6`),
          // а для беззнаковых типов оно становится 0 (`U8 = -5.5` -> `0`).
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
      }
      
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      CastFrame::Float32 =>
      {
        let floatValue: f64 = number.magnitude().clamp(f32::MIN as f64, f32::MAX as f64);
        // `{:?}` для `f32`/`f64` сохраняет точку для целых `float`:
        // `1.0` -> `1.0`, не `1`; и использует `e`-нотацию только для
        // больших/маленьких чисел. Так `data` всегда одинаково проходит 
        // через токенайзер как `Float`/`UFloat`.
        token.setData( Bytes::from(format!("{:?}", floatValue as f32)) );
      }
      
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      CastFrame::Float64 =>
      {
        let floatValue: f64 = number.magnitude().clamp(f64::MIN, f64::MAX);
        // `{:?}` для `f32`/`f64` сохраняет точку для целых `float`:
        // `1.0` -> `1.0`, не `1`; и использует `e`-нотацию только для
        // больших/маленьких чисел. Так `data` всегда одинаково проходит 
        // через токенайзер как `Float`/`UFloat`.
        token.setData( Bytes::from(format!("{:?}", floatValue)) );
      }
      
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    }
  }

  // ===============================================================================================
}
*/

// =================================================================================================
