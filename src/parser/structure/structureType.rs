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
#[derive(PartialEq)]
#[derive(Clone)]
pub enum StructureType
{
// primitives
  None,
  Any,
  Link,

  Bool, // todo Потом надо будет заменить на True/False - issue #65

  U8, U16, U32, U64,
  I8, I16, I32, I64,
  F32, F64,
  Usize, Isize,
  Pointer, // указатель (raw)

  // todo Требует удаление для FFI-ABI?
  Method,
  // todo Требует удаление для FFI-ABI?
  List, // todo List<Type>

// custom
  /// Позволяет создавать пользовательские типы
  Custom(String),
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
      StructureType::None => String::from("None"),
      StructureType::Any => String::from("Any"),
      StructureType::Link => String::from("Link"),
      StructureType::Bool => String::from("Bool"), // todo Требует True/False по issue #65

      // Беззнаковые
      StructureType::U8 => String::from("U8"),
      StructureType::U16 => String::from("U16"),
      StructureType::U32 => String::from("U32"),
      StructureType::U64 => String::from("U64"),
      // U128 нет т.к. это не FFI совместимый тип данных
      StructureType::Usize => String::from("Usize"),

      // Знаковые
      StructureType::I8 => String::from("I8"),
      StructureType::I16 => String::from("I16"),
      StructureType::I32 => String::from("I32"),
      StructureType::I64 => String::from("I64"),
      // I128 нет т.к. это не FFI совместимый тип данных
      StructureType::Isize => String::from("Isize"),

      // Плавающие
      // F16 нет т.к. это не FFI совместимый тип данных
      StructureType::F32 => String::from("F32"),
      StructureType::F64 => String::from("F64"),

      // Указатель
      StructureType::Pointer => String::from("Pointer"),

      // Служебные
      StructureType::Method => String::from("Method"),
      StructureType::List => String::from("List"),

      // custom
      StructureType::Custom(value) => value.clone(),
    }
  }
}

// =================================================================================================

impl Structure
{
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
    let tokenData: String = match token.getData().toString()
    {
      Some(s) => s,
      None =>
      { // Нет данных
        token.setDefaultValue(structureType);
        return;
      }
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
          {
            if let Ok(mut value) = tokenData.parse::<u128>() 
            {
              match structureType 
              {
                StructureType::U8 => value = value.clamp(0, u8::MAX as u128),
                StructureType::U16 => value = value.clamp(0, u16::MAX as u128),
                StructureType::U32 => value = value.clamp(0, u32::MAX as u128),
                StructureType::U64 => value = value.clamp(0, u64::MAX as u128),
                StructureType::Usize => value = value.clamp(0, usize::MAX as u128),
                StructureType::I8 => value = value.clamp(0, i8::MAX as u128),
                StructureType::I16 => value = value.clamp(0, i16::MAX as u128),
                StructureType::I32 => value = value.clamp(0, i32::MAX as u128),
                StructureType::I64 => value = value.clamp(0, i64::MAX as u128),
                StructureType::Isize => value = value.clamp(0, isize::MAX as u128),
                StructureType::F32 => {
                  let floatValue: f64 = (value as f64).clamp(f32::MIN as f64, f32::MAX as f64);
                  token.setData( Bytes::from((floatValue as f32).to_string()) );
                  return;
                }
                StructureType::F64 => {
                  let floatValue: f64 = value as f64;
                  token.setData( Bytes::from(floatValue.to_string()) );
                  return;
                }
                _ => {}
              }
              token.setData( Bytes::from(value.to_string()) );
            } else 
            { // Не распарсилось — базовое значение
              token.setDefaultValue(structureType);
            }
          }
          TokenType::Int => 
          {
            if let Ok(mut value) = tokenData.parse::<i128>() 
            {
              match structureType 
              {
                StructureType::U8 => {
                  if value < 0 { value = 0; }
                  value = value.clamp(0, u8::MAX as i128);
                }
                StructureType::U16 => {
                  if value < 0 { value = 0; }
                  value = value.clamp(0, u16::MAX as i128);
                }
                StructureType::U32 => {
                  if value < 0 { value = 0; }
                  value = value.clamp(0, u32::MAX as i128);
                }
                StructureType::U64 => {
                  if value < 0 { value = 0; }
                  value = value.clamp(0, u64::MAX as i128);
                }
                StructureType::Usize => {
                  if value < 0 { value = 0; }
                  value = value.clamp(0, usize::MAX as i128);
                }
                StructureType::I8 => value = value.clamp(i8::MIN as i128, i8::MAX as i128),
                StructureType::I16 => value = value.clamp(i16::MIN as i128, i16::MAX as i128),
                StructureType::I32 => value = value.clamp(i32::MIN as i128, i32::MAX as i128),
                StructureType::I64 => value = value.clamp(i64::MIN as i128, i64::MAX as i128),
                StructureType::Isize => value = value.clamp(isize::MIN as i128, isize::MAX as i128),
                StructureType::F32 => {
                  let floatValue: f64 = (value as f64).clamp(f32::MIN as f64, f32::MAX as f64);
                  token.setData( Bytes::from((floatValue as f32).to_string()) );
                  return;
                }
                StructureType::F64 => {
                  let floatValue: f64 = value as f64;
                  token.setData( Bytes::from(floatValue.to_string()) );
                  return;
                }
                _ => {}
              }
              token.setData( Bytes::from(value.to_string()) );
            } else 
            { // Не распарсилось — базовое значение
              token.setDefaultValue(structureType);
            }
          }
          TokenType::UFloat | TokenType::Float => 
          {
            if let Ok(mut value) = tokenData.parse::<f64>() 
            { // Для UFloat обрезаем отрицательные до 0
              if dataType == &TokenType::UFloat && value < 0.0 {
                value = 0.0;
              }
              if !value.is_finite() 
              {
                // Бесконечность или NaN — базовое значение
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
  pub fn getStructureType(&mut self) -> StructureType
  {
    let result = |selfToken: &mut Token, structureType: StructureType| -> StructureType
    {
      if structureType == StructureType::None {
        selfToken.setData(None);
      }
      return structureType;
    };
    
    //
    let dataType: &TokenType = self.getDataType();
    
    // Получаем строку из данных токена
    let data: String = match self.getData().toString() {
      Some(s) => s,
      None => return result(self, StructureType::None),
    };

    result(self, match dataType 
    {
      TokenType::None => StructureType::None,
      TokenType::Any => StructureType::Any,
      TokenType::Link => StructureType::Link,
      //
      TokenType::UInt => 
      {
        if let Ok(value) = data.parse::<u128>() 
        {
          if value <= u8::MAX as u128 {
            StructureType::U8
          } else if value <= u16::MAX as u128 {
            StructureType::U16
          } else if value <= u32::MAX as u128 {
            StructureType::U32
          } else if value <= u64::MAX as u128 {
            StructureType::U64
          } else if value <= usize::MAX as u128 {
            StructureType::Usize
          } else {
            // Выходит за рамки хранения структуры - не обрабатываем
            StructureType::None
          }
        } else {
          // Что-то непонятное
          StructureType::None
        }
      }
      TokenType::Int => 
      {
        if let Ok(value) = data.parse::<i128>() 
        {
          if value >= i8::MIN as i128 && value <= i8::MAX as i128 {
            StructureType::I8
          } else if value >= i16::MIN as i128 && value <= i16::MAX as i128 {
            StructureType::I16
          } else if value >= i32::MIN as i128 && value <= i32::MAX as i128 {
            StructureType::I32
          } else if value >= i64::MIN as i128 && value <= i64::MAX as i128 {
            StructureType::I64
          } else if value >= isize::MIN as i128 && value <= isize::MAX as i128 {
            StructureType::Isize
          } else {
            // Выходит за рамки хранения структуры - не обрабатываем
            StructureType::None
          }
        } else {
          // Что-то непонятное
          StructureType::None
        }
      }
      TokenType::UFloat | TokenType::Float => 
      {
        if let Ok(value) = data.parse::<f64>() 
        {
          if value >= f32::MIN as f64 && value <= f32::MAX as f64 {
            StructureType::F32
          } else if value >= f64::MIN && value <= f64::MAX {
            // Самый крайний тип
            StructureType::F64
          } else {
            // Выходит за рамки хранения структуры - не обрабатываем
            StructureType::None
          }
        } else {
          // Что-то непонятное
          StructureType::None
        }
      }
      // Для остальных типов - возвращаем Custom
      // todo Сейчас могут попасть лишние т.к. они не объявлены выше
      _ => StructureType::None,
    })
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

      // Служебные
      // todo Под вопросом
      "Method" => StructureType::Method,
      "List" => StructureType::List,

      // Всё остальное — кастомное
      _ => StructureType::Custom(data),
    }
    //
  }
}

// =================================================================================================