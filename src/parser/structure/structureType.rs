use std::num::IntErrorKind;
use serde::{Deserialize, Serialize};
use crate::parser::bytes::Bytes;
use crate::parser::structure::structure::Structure;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Идея простая - т.к. мы имеем хранение в токенах, то это абстрактные данные;
// Поэтому физические вещи стоит хранить привязывать к Structure;
// Внутри это все еще токены, но через StructureType - мы контролируем их.

// Поэтому нам следует "нормализовать" - т.е. привести к нужной форме 
// токены при хранении в структуре. На что указывает StructureType - 
// что вообще мы должны хранить и в каком виде в Structure.

// Это позволит TokenType -> StructureType на уровне типов.

// =================================================================================================

/// Тип данных структуры
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum StructureType
{
  None,
  Any,
  Link,

  Bool, // todo Потом надо будет заменить на True/False - issue #65

  U8, U16, U32, U64,
  I8, I16, I32, I64,
  F32, F64,
  Usize, Isize,
  
  /// Указатель (raw).
  Pointer,
  /// ABI-композит: 2 поля (.pointer .length), см. Structure::stringFields в ffi/bridge.rs.
  String,
  // CString = Pointer, алиас на уровне getStructureTypeSimple().
  /// Байты без длины/NUL в самом StructureType — длина берётся из токена при FFI-вызове.
  RawString,

  // todo Требует удаление для FFI-ABI?
  Method,
  // todo Требует удаление для FFI-ABI?
  List, // todo List<Type>
  
  /// Позволяет создавать пользовательские типы
  Custom(String)
}

// =================================================================================================

// todo Можно заменить данные на keywords из words.rs, 
//   но их не хватит т.к. тут есть другие, 
//   + здесь structure type
impl ToString for StructureType
{ // todo convert -> fmt::Display ?
  fn to_string(&self) -> String
  {
    match self
    { //
      Self::None => String::from("None"),
      Self::Any => String::from("Any"),
      Self::Link => String::from("Link"),
      Self::Bool => String::from("Bool"), // todo Требует True/False по issue #65

      // Беззнаковые
      Self::U8 => String::from("U8"),
      Self::U16 => String::from("U16"),
      Self::U32 => String::from("U32"),
      Self::U64 => String::from("U64"),
      // U128 нет т.к. это не FFI совместимый тип данных
      Self::Usize => String::from("Usize"),

      // Знаковые
      Self::I8 => String::from("I8"),
      Self::I16 => String::from("I16"),
      Self::I32 => String::from("I32"),
      Self::I64 => String::from("I64"),
      // I128 нет т.к. это не FFI совместимый тип данных
      Self::Isize => String::from("Isize"),

      // Плавающие
      // F16 нет т.к. это не FFI совместимый тип данных
      Self::F32 => String::from("F32"),
      Self::F64 => String::from("F64"),

      // Указатель
      Self::Pointer => String::from("Pointer"),
      Self::RawString => String::from("RawString"),
      Self::String => String::from("String"),

      // Служебные
      Self::Method => String::from("Method"),
      Self::List => String::from("List"),

      // custom
      Self::Custom(value) => value.clone()
    }
  }
}

// =================================================================================================

impl Structure
{
  /// Границы целочисленного типа: (минимум, максимум);
  fn integerLimits(structureType: StructureType) -> Option<(i64, u64)> 
  {
    match structureType 
    {
      StructureType::U8    => Some(( 0,             u8::MAX    as u64 )),
      StructureType::U16   => Some(( 0,             u16::MAX   as u64 )),
      StructureType::U32   => Some(( 0,             u32::MAX   as u64 )),
      StructureType::U64   => Some(( 0,             u64::MAX )),
      StructureType::Usize => Some(( 0,             usize::MAX as u64 )),
      StructureType::I8    => Some(( i8::MIN    as i64, i8::MAX    as u64 )),
      StructureType::I16   => Some(( i16::MIN   as i64, i16::MAX   as u64 )),
      StructureType::I32   => Some(( i32::MIN   as i64, i32::MAX   as u64 )),
      StructureType::I64   => Some(( i64::MIN,          i64::MAX   as u64 )),
      StructureType::Isize => Some(( isize::MIN as i64, isize::MAX as u64 )),
      _ => None
    }
  }

  /// Приводит данные токена в рамки требуемого StructureType,
  /// чтобы Structure смог его безопасно хранить;
  /// 
  /// Это делается только при присвоении или при изменении.
  /// 
  /// todo Есть идея ввести на уровне Structure в будущем bool,
  ///  чтобы можно было проверить и оптимизировать эту работу.
  pub fn normalizeToken(token: &mut Token, structureType: StructureType) 
  {
    let dataType: &TokenType = token.getDataType();

    // Получаем строку из данных
    let tokenData: String = if let Some(tokenData) =
      token.getData().toString() { tokenData } else 
      { // Нет данных
        token.setDefaultValue(structureType);
        return;
      };

    // Обработка типов
    match structureType 
    {
      StructureType::U8 | StructureType::U16 | StructureType::U32 | StructureType::U64 | StructureType::Usize |
      StructureType::I8 | StructureType::I16 | StructureType::I32 | StructureType::I64 | StructureType::Isize |
      StructureType::F32 | StructureType::F64 => 
      {
        match dataType
        {
          TokenType::UInt => 
          { // Токен бесконечен, а u64 нет: всё что больше - граница u64 (#71),
            // дальше значение зажимается в рамки структуры
            let value: Option<u64> = match tokenData.parse::<u64>() 
            {
              Ok(value) => Some(value),
              Err(error) if *error.kind() == IntErrorKind::PosOverflow => Some(u64::MAX),
              Err(_) => None
            };
            match value 
            {
              None => token.setDefaultValue(structureType), // Не распарсилось — базовое значение
              Some(value) => match structureType 
              {
                // Для float берём величину числа напрямую, без сжатия в u64
                StructureType::F32 => {
                  let floatValue: f64 = tokenData.parse::<f64>().unwrap_or(value as f64)
                    .clamp(f32::MIN as f64, f32::MAX as f64);
                  token.setData( Bytes::from((floatValue as f32).to_string()) );
                }
                StructureType::F64 => {
                  let floatValue: f64 = tokenData.parse::<f64>().unwrap_or(value as f64);
                  token.setData( Bytes::from(floatValue.to_string()) );
                }
                _ => if let Some((_, max)) = Self::integerLimits(structureType) {
                  token.setData( Bytes::from(value.min(max).to_string()) );
                }
              }
            }
          }
          TokenType::Int => 
          { // Токен бесконечен, а i64 нет: всё что больше или меньше - граница i64 (#71),
            // дальше значение зажимается в рамки структуры
            let value: Option<i64> = match tokenData.parse::<i64>() 
            {
              Ok(value) => Some(value),
              Err(error) if *error.kind() == IntErrorKind::NegOverflow => Some(i64::MIN),
              Err(error) if *error.kind() == IntErrorKind::PosOverflow => Some(i64::MAX),
              Err(_) => None
            };
            match value 
            {
              None => token.setDefaultValue(structureType), // Не распарсилось — базовое значение
              Some(value) => match structureType 
              {
                // Для float берём величину числа напрямую, без сжатия в i64
                StructureType::F32 => {
                  let floatValue: f64 = tokenData.parse::<f64>().unwrap_or(value as f64)
                    .clamp(f32::MIN as f64, f32::MAX as f64);
                  token.setData( Bytes::from((floatValue as f32).to_string()) );
                }
                StructureType::F64 => {
                  let floatValue: f64 = tokenData.parse::<f64>().unwrap_or(value as f64);
                  token.setData( Bytes::from(floatValue.to_string()) );
                }
                _ => if let Some((min, max)) = Self::integerLimits(structureType) {
                  let clamped: String = 
                    if value < min { min.to_string() } else 
                    if value > 0 && (value as u64) > max { max.to_string() } else 
                    { value.to_string() };
                  token.setData( Bytes::from(clamped) );
                }
              }
            }
          }
          TokenType::UFloat | TokenType::Float => 
          {
            if let Ok(mut value) = tokenData.parse::<f64>() 
            { // Для UFloat обрезаем отрицательные до 0
              if dataType == &TokenType::UFloat && value < 0.0 {
                value = 0.0;
              }
              if value.is_nan() 
              { // NaN — базовое значение; 
                // Бесконечность зажмётся в границы типа (#71)
                token.setDefaultValue(structureType);
                return;
              }
              match structureType 
              {
                StructureType::F32 => {
                  if value < f32::MIN as f64 { value = f32::MIN as f64; }
                  if value > f32::MAX as f64 { value = f32::MAX as f64; }
                  token.setData( Bytes::from((value as f32).to_string()) );
                }
                StructureType::F64 => {
                  if value < f64::MIN { value = f64::MIN; }
                  if value > f64::MAX { value = f64::MAX; }
                  token.setData( Bytes::from(value.to_string()) );
                }
                // Приведение к целочисленным типам
                target if matches!(target, 
                  StructureType::U8 | StructureType::U16 | StructureType::U32 | StructureType::U64 | StructureType::Usize |
                  //
                  StructureType::I8 | StructureType::I16 | StructureType::I32 | StructureType::I64 | StructureType::Isize) =>
                {
                  let integerValue: i128 = if value < 0.0 { 0 } else { value.round() as i128 };
                  match target 
                  {
                    StructureType::U8 => {
                      let clamped: i128 = integerValue.clamp(0, u8::MAX as i128);
                      token.setData( Bytes::from((clamped as u8).to_string()) );
                    }
                    StructureType::U16 => {
                      let clamped: i128 = integerValue.clamp(0, u16::MAX as i128);
                      token.setData( Bytes::from((clamped as u16).to_string()) );
                    }
                    StructureType::U32 => {
                      let clamped: i128 = integerValue.clamp(0, u32::MAX as i128);
                      token.setData( Bytes::from((clamped as u32).to_string()) );
                    }
                    StructureType::U64 => {
                      let clamped: i128 = integerValue.clamp(0, u64::MAX as i128);
                      token.setData( Bytes::from((clamped as u64).to_string()) );
                    }
                    StructureType::Usize => {
                      let clamped: i128 = integerValue.clamp(0, usize::MAX as i128);
                      token.setData( Bytes::from((clamped as usize).to_string()) );
                    }
                    StructureType::I8 => {
                      let clamped: i128 = integerValue.clamp(i8::MIN as i128, i8::MAX as i128);
                      token.setData( Bytes::from((clamped as i8).to_string()) );
                    }
                    StructureType::I16 => {
                      let clamped: i128 = integerValue.clamp(i16::MIN as i128, i16::MAX as i128);
                      token.setData( Bytes::from((clamped as i16).to_string()) );
                    }
                    StructureType::I32 => {
                      let clamped: i128 = integerValue.clamp(i32::MIN as i128, i32::MAX as i128);
                      token.setData( Bytes::from((clamped as i32).to_string()) );
                    }
                    StructureType::I64 => {
                      let clamped: i128 = integerValue.clamp(i64::MIN as i128, i64::MAX as i128);
                      token.setData( Bytes::from((clamped as i64).to_string()) );
                    }
                    StructureType::Isize => {
                      let clamped: i128 = integerValue.clamp(isize::MIN as i128, isize::MAX as i128);
                      token.setData( Bytes::from((clamped as isize).to_string()) );
                    }
                    _ => {}
                  }
                }
                _ => {}
              }
            } else 
            { // Не распарсилось — базовое значение
              token.setDefaultValue(structureType);
            }
          }
          _ => {
            // Здесь пытаются прировнять что-то левое
            token.setDefaultValue(structureType);
          }
          //
        }
      }
      _ => {
        // todo
        // Другие типы — ничего не делаем
      }
    }
    //
  }
}

// =================================================================================================

impl Token
{
  /// Ставит стандартное значение токена при указании требуемого StructureType;
  /// 
  /// Очень удобно для случаев, если в структуру приравнивают что-то левое,
  /// и нам надо default значение.
  /// 
  /// Потому что в если `a: U8 = "test"`, то будет 0 из-за константного поведения.
  fn setDefaultValue(&mut self, structureType: StructureType) -> () 
  {
    // Создаём базовый токен
    match structureType
    {
      // Целочисленные типы
      StructureType::U8 | StructureType::U16 | StructureType::U32 | StructureType::U64 |
      StructureType::Usize | StructureType::I8 | StructureType::I16 | StructureType::I32 |
      StructureType::I64 | StructureType::Isize => {
        self.setDataType(TokenType::UInt);
        self.setData("0");
      }
      // Числа с плавающей точкой
      StructureType::F32 | StructureType::F64 => {
        self.setDataType(TokenType::Float);
        self.setData("0.0");
      }
      // todo
      // Для остальных типов - ничего
      _ => {
        self.setDataType(TokenType::None);
        self.setData(None);
      }
    };
  }
}

// =================================================================================================

impl Token
{
  /// Вычисляет StructureType на основе токена;
  /// 
  /// Удобно, когда нет рамок структуры (не указан её тип) и нужно понять, 
  /// что в неё положили, но при этом в рамках StructureType;
  /// 
  /// Если станет None - то токен будет очищен.
  /// 
  /// todo Было бы круто убрать как-то mut отсюда.
  pub fn getStructureType(&mut self) -> StructureType
  {
    let result = |selfToken: &mut Self, structureType: StructureType| -> StructureType
    {
      if structureType == StructureType::None {
        selfToken.setData(None);
      }
      structureType
    };
    
    //
    let dataType: &TokenType = self.getDataType();
    
    // Получаем строку из данных токена
    let data: String = if let Some(string) = 
      self.getData().toString() { string }
      else { return result(self, StructureType::None) };

    // Токены UInt, Int, UFloat, Float бесконечны, а ABI типы структур ограничены;
    // Если число вышло за самый крайний ABI тип, то это константное поведение (#71):
    // тип становится крайним (U64, I64, F64), а значение - его границей;
    // Здесь хранится новое значение, если оно изменилось.
    let mut newData: Option<String> = None;

    let structureType: StructureType = match dataType 
    {
      TokenType::None => StructureType::None,
      TokenType::Any => StructureType::Any,
      TokenType::Link => StructureType::Link,
      //
      TokenType::UInt => 
      {
        match data.parse::<u64>() 
        {
          Ok(value) if value <= u8::MAX  as u64 => StructureType::U8,
          Ok(value) if value <= u16::MAX as u64 => StructureType::U16,
          Ok(value) if value <= u32::MAX as u64 => StructureType::U32,
          Ok(_) => StructureType::U64,
          // Больше U64: вид ошибки говорит, что это переполнение числа
          Err(error) if *error.kind() == IntErrorKind::PosOverflow => 
          {
            newData = Some( u64::MAX.to_string() );
            StructureType::U64
          }
          // Что-то непонятное
          Err(_) => StructureType::None
        }
      }
      TokenType::Int => 
      {
        match data.parse::<i64>() 
        {
          Ok(value) if value >= i8::MIN  as i64 && value <= i8::MAX  as i64 => StructureType::I8,
          Ok(value) if value >= i16::MIN as i64 && value <= i16::MAX as i64 => StructureType::I16,
          Ok(value) if value >= i32::MIN as i64 && value <= i32::MAX as i64 => StructureType::I32,
          Ok(_) => StructureType::I64,
          // Меньше или больше I64: вид ошибки говорит, что это переполнение числа
          Err(error) => match error.kind() 
          {
            IntErrorKind::NegOverflow => 
            {
              newData = Some( i64::MIN.to_string() );
              StructureType::I64
            }
            IntErrorKind::PosOverflow => 
            {
              newData = Some( i64::MAX.to_string() );
              StructureType::I64
            }
            // Что-то непонятное
            _ => StructureType::None
          }
        }
      }
      TokenType::UFloat | TokenType::Float => 
      {
        match data.parse::<f64>() 
        {
          Ok(value) if value.is_nan() => StructureType::None,
          Ok(value) if value >= f32::MIN as f64 && value <= f32::MAX as f64 => StructureType::F32,
          Ok(value) => 
          { // Самый крайний тип; inf становится границей F64
            if value.is_infinite() 
            {
              let border: f64 = if value.is_sign_negative() { f64::MIN } else { f64::MAX };
              newData = Some( format!("{:e}", border) );
            }
            StructureType::F64
          }
          // Что-то непонятное
          Err(_) => StructureType::None
        }
      }
      // Для остальных типов - возвращаем Custom
      // todo Сейчас могут попасть лишние т.к. они не объявлены выше
      _ => StructureType::None,
    };

    if let Some(newData) = newData {
      self.setData( Bytes::from(newData) );
    }
    result(self, structureType)
    //
  }

  /// Вычисляет StructureType на основе токена;
  /// Но делает это упрощенно по имени токена - 
  /// т.е. когда мы явно знаем уже тип в строке.
  pub fn getStructureTypeSimple(&self) -> StructureType
  {
    let data: String = self.to_string();
    match data.as_str()
    {
      //
      "None" => StructureType::None,
      "Any" => StructureType::Any,
      "Link" => StructureType::Link,
      "Bool" => StructureType::Bool,

      // Беззнаковые
      "U8" => StructureType::U8,
      "U16" => StructureType::U16,
      "U32" => StructureType::U32,
      "U64" => StructureType::U64,
      "Usize" => StructureType::Usize,

      // Знаковые
      "I8" => StructureType::I8,
      "I16" => StructureType::I16,
      "I32" => StructureType::I32,
      "I64" => StructureType::I64,
      "Isize" => StructureType::Isize,

      // Плавающие
      "F32" => StructureType::F32,
      "F64" => StructureType::F64,

      // Указатель
      "Pointer" => StructureType::Pointer,
      "CString" => StructureType::Pointer, // CString = Pointer (алиас)
      "RawString" => StructureType::RawString,
      "String" => StructureType::String,

      // Служебные
      // todo Под вопросом
      "Method" => StructureType::Method,
      "List" => StructureType::List,

      // Всё остальное — кастомное
      _ => StructureType::Custom(data)
    }
    //
  }
}

// =================================================================================================

#[cfg(test)]
mod tests
{
  use crate::tokenizer::types::token::Token;
  use crate::tokenizer::types::tokenType::TokenType;
  use crate::parser::structure::structureType::StructureType;
  use crate::parser::structure::structure::Structure;
  // ===============================================================================================

  /// Проверяет тип и значение токена после getStructureType();
  fn check(tokenType: TokenType, data: &str, expectedType: StructureType, expectedData: &str)
  {
    let mut token: Token = Token::new(tokenType, String::from(data));
    let structureType: StructureType = token.getStructureType();
    let tokenData: String = token.getData().toString().unwrap_or_default();
    assert!(
      structureType == expectedType && tokenData == expectedData,
      "Для '{}' ожидалось значение '{}', получено '{}' (тип совпал: {})",
      data, expectedData, tokenData, structureType == expectedType
    );
  }

  /// Числа внутри рамок ABI не меняются;
  #[test]
  fn inRange()
  {
    check(TokenType::UInt,   "255",                  StructureType::U8,  "255");
    check(TokenType::UInt,   "18446744073709551615", StructureType::U64, "18446744073709551615");
    check(TokenType::Int,    "-128",                 StructureType::I8,  "-128");
    check(TokenType::Int,    "-9223372036854775808", StructureType::I64, "-9223372036854775808");
    check(TokenType::UFloat, "1.5",                  StructureType::F32, "1.5");
  }

  /// За рамками ABI тип и значение становятся границей крайнего типа (#71);
  #[test]
  fn saturation()
  {
    // UInt: > u64, и > u128
    check(TokenType::UInt, "18446744073709551616", StructureType::U64, &u64::MAX.to_string());
    check(TokenType::UInt, "99999999999999999999999999999999999999999999", StructureType::U64, &u64::MAX.to_string());
    // Int: < i64, и < i128
    check(TokenType::Int, "-9223372036854775809", StructureType::I64, &i64::MIN.to_string());
    check(TokenType::Int, "-99999999999999999999999999999999999999999999", StructureType::I64, &i64::MIN.to_string());
    // Int: > i64
    check(TokenType::Int, "9223372036854775808", StructureType::I64, &i64::MAX.to_string());
    // Float: inf после parse
    check(TokenType::UFloat, "1e309",  StructureType::F64, &format!("{:e}", f64::MAX));
    check(TokenType::Float,  "-1e309", StructureType::F64, &format!("{:e}", f64::MIN));
  }

  /// Проверяет значение токена после normalizeToken() в явный тип;
  fn normalize(tokenType: TokenType, data: &str, structureType: StructureType, expectedData: &str)
  {
    let mut token: Token = Token::new(tokenType, String::from(data));
    Structure::normalizeToken(&mut token, structureType);
    let tokenData: String = token.getData().toString().unwrap_or_default();
    assert!(
      tokenData == expectedData,
      "Для '{}' ожидалось значение '{}', получено '{}'",
      data, expectedData, tokenData
    );
  }

  /// Явный тип зажимает значение в свои границы, даже если число больше u64 и i64 (#71);
  #[test]
  fn normalizeClamp()
  {
    let big: &str = "44444444444444444444444444444444444444444444"; // 44 цифры
    let negBig: String = format!("-{}", big);
    // Обычный clamp
    normalize(TokenType::UInt, "300", StructureType::U8, "255");
    normalize(TokenType::Int, "-10", StructureType::U8, "0");
    normalize(TokenType::Int, "-300", StructureType::I8, "-128");
    // Больше u64
    normalize(TokenType::UInt, "18446744073709551616", StructureType::U64, &u64::MAX.to_string());
    normalize(TokenType::UInt, big, StructureType::U8, "255");
    normalize(TokenType::UInt, big, StructureType::I8, "127");
    normalize(TokenType::UInt, big, StructureType::I64, &i64::MAX.to_string());
    normalize(TokenType::UInt, big, StructureType::Usize, &usize::MAX.to_string());
    // Меньше i64
    normalize(TokenType::Int, &negBig, StructureType::I8, "-128");
    normalize(TokenType::Int, &negBig, StructureType::I64, &i64::MIN.to_string());
    normalize(TokenType::Int, &negBig, StructureType::Isize, &isize::MIN.to_string());
    normalize(TokenType::Int, &negBig, StructureType::U64, "0");
    // Не число - базовое значение
    normalize(TokenType::UInt, "abc", StructureType::U8, "0");
  }

  /// Большие числа в float сохраняют величину, а не сжимаются в u64 (#71);
  #[test]
  fn normalizeFloat()
  {
    let big: &str = "44444444444444444444444444444444444444444444"; // 44 цифры
    normalize(TokenType::UInt, big, StructureType::F32, &f32::MAX.to_string());
    normalize(TokenType::UInt, big, StructureType::F64, &big.parse::<f64>().unwrap().to_string());
    // Бесконечность зажимается в границу типа
    normalize(TokenType::Float, "-1e309", StructureType::F32, &f32::MIN.to_string());
    normalize(TokenType::UFloat, "1e309", StructureType::F64, &f64::MAX.to_string());
    normalize(TokenType::UFloat, "1e309", StructureType::U8, "255");
  }

  /// Не число - по прежнему None, токен очищается;
  #[test]
  fn notNumber()
  {
    check(TokenType::UInt,   "abc", StructureType::None, "");
    check(TokenType::Int,    "-",   StructureType::None, "");
    check(TokenType::UFloat, "NaN", StructureType::None, "");
  }
}
