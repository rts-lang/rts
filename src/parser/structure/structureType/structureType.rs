use crate::parser::bytes::Bytes;
use crate::parser::structure::structure::Structure;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use serde::{Deserialize, Serialize};
use std::num::IntErrorKind;
// =================================================================================================

// Идея простая - т.к. мы имеем хранение в токенах, то это абстрактные данные;
// Поэтому физические вещи стоит хранить привязывать к Structure;
// Внутри это все еще токены, но через StructureType - мы контролируем их.
//
// Поэтому нам следует "нормализовать" - т.е. привести к нужной форме 
// токены при хранении в структуре. На что указывает StructureType - 
// что вообще мы должны хранить и в каком виде в Structure.
//
// Это позволит TokenType -> StructureType на уровне типов.

// =================================================================================================

/// Вид примитива, допустимого как граница типизации (`a: 10 | "text"`).
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveKind
{
  /// Число: UInt, Int, UFloat, Float.
  Number,
  
  /// Строка: String, RawString.
  Text
}

// =================================================================================================

/// Тип данных структуры.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum StructureType
{
  None,
  Any,
  Link,

  /// Логический примитив
  True,
  /// Логический примитив
  False,

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
  
  /// Позволяет создавать пользовательские типы.
  Custom(String),

  /// Модуль из `import(...)`; id — ключ lookup без имени структуры.
  /// 
  /// Это правильно т.к. поиск по имени может уйти за структуру, а id
  /// позволяет остаться только в `self` scope.
  Module(usize),

  /// Объединение типов: `a: U8 | String = ...` (issue #59).
  ///
  /// Это НЕ аналог TypeScript: здесь хранится не "пересечение", а список
  /// допустимых вариантов, и в структуре единовременно лежит РОВНО один из них.
  ///
  /// Семантика:
  /// - при присваивании выбирается тот вариант, в который значение
  ///   помещается как есть, либо в который оно приводится;
  /// - если ни один вариант не подходит и привести нельзя — `None` (#71/#59);
  /// - при полной мутабельности `~~` тип может меняться, поэтому объединение
  ///   можно указать, но оно ничего не ограничивает (см. issue #22).
  ///
  /// Пустой список вариантов равносилен `None`.
  ///
  /// Вариантом может быть и примитив (`Primitive`): `a: 10 | 20`, `a: "text" | U8`.
  Union(Vec<Self>),

  /// Примитив как граница типизации: `a: 10 | 20 = 10`.
  ///
  /// Это ограничение значения: допустимо только точное совпадение,
  /// без приведения и без зажима. При матче примитив сам становится
  /// `dataType` структуры (`stype` → `10` / `"text"`), а не natural ABI-тип.
  /// Для ABI (FFI, string fields) см. `abiType()`: Text → String, Number → natural.
  /// Значение вне примитивов - `None` (#71).
  ///
  /// Живёт только в type-секции (слева от `=`): справа `|` - это логическое ИЛИ.
  /// Отдельный примитив `a: 10` оборачивается в `Union` из одного варианта.
  Primitive(PrimitiveKind, String)
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
      Self::True => String::from("True"),
      Self::False => String::from("False"),

      // Беззнаковые.
      Self::U8 => String::from("U8"),
      Self::U16 => String::from("U16"),
      Self::U32 => String::from("U32"),
      Self::U64 => String::from("U64"),
      // U128 нет т.к. это не FFI совместимый тип данных.
      Self::Usize => String::from("Usize"),

      // Знаковые.
      Self::I8 => String::from("I8"),
      Self::I16 => String::from("I16"),
      Self::I32 => String::from("I32"),
      Self::I64 => String::from("I64"),
      // I128 нет т.к. это не FFI совместимый тип данных.
      Self::Isize => String::from("Isize"),

      // Плавающие.
      // F16 нет т.к. это не FFI совместимый тип данных.
      Self::F32 => String::from("F32"),
      Self::F64 => String::from("F64"),

      // Указатель.
      Self::Pointer => String::from("Pointer"),
      Self::RawString => String::from("RawString"),
      Self::String => String::from("String"),

      // Служебные.
      Self::Method => String::from("Method"),
      Self::List => String::from("List"),

      // Объединение типов (issue #59) — печатается так же, как записывается.
      Self::Union(variants) => variants
        .iter()
        .map(|variant: &Self| variant.to_string())
        .collect::<Vec<String>>()
        .join(" | "),

      // Примитив печатается так же, как записывается.
      Self::Primitive(PrimitiveKind::Number, value) => value.clone(),
      Self::Primitive(PrimitiveKind::Text, value) => format!("\"{}\"", value),

      // Module.
      Self::Module(_) => String::from("Module"),

      // Custom.
      Self::Custom(value) => value.clone()
    }
  }
}

// =================================================================================================

impl StructureType
{
  /// ABI / storage тип: примитив сводится к natural-форме значения.
  ///
  /// `Primitive(Text, _)` → `String`; `Primitive(Number, v)` → наименьший числовой
  /// ABI-тип для `v`; всё остальное — как есть. Нужен там, где `dataType`
  /// участвует в FFI, string-полях и `normalizeToken`, а не в `stype`.
  pub fn abiType(&self) -> Self
  {
    match self
    {
      Self::Primitive(PrimitiveKind::Text, _) => Self::String,
      Self::Primitive(PrimitiveKind::Number, value) =>
      {
        // Те же правила, что naturalType для числового токена: через временный UInt/Int.
        let tokenType: TokenType = 
          if value.starts_with('-') {
            TokenType::Int
          } else
          if value.contains('.') {
            TokenType::UFloat
          } else {
            TokenType::UInt
          };
        let mut token: Token = Token::new(tokenType, value.clone());
        Self::naturalType(&mut token)
      }
      other => other.clone()
    }
  }

  /// Тип, который значение занимает САМО по себе, без учёта объявления.
  ///
  /// Для числа это наименьший ABI-тип, в который оно помещается
  /// (и насыщение по границам самого широкого типа, #71);
  /// для строки — `String`. Токен при этом НЕ очищается,
  /// в отличие от `Token::getStructureType()`, который чистит всё нечисловое.
  pub fn naturalType(token: &mut Token) -> Self
  {
    match token.getDataType()
    {
      TokenType::None => Self::None,
      TokenType::Any => Self::Any,
      TokenType::Link => Self::Link,
      TokenType::True => Self::True,
      TokenType::False => Self::False,
      TokenType::String => Self::String,
      TokenType::RawString => Self::RawString,
      // Числа: ширину и насыщение считает сам getStructureType.
      TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float =>
        token.getStructureType(),
      // Неизвестное — трактуем как отсутствие значения.
      _ => Self::None
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
      StructureType::U8    => Some(( 0,                 u8::MAX    as u64 )),
      StructureType::U16   => Some(( 0,                 u16::MAX   as u64 )),
      StructureType::U32   => Some(( 0,                 u32::MAX   as u64 )),
      StructureType::U64   => Some(( 0,                 u64::MAX          )),
      StructureType::Usize => Some(( 0,                 usize::MAX as u64 )),
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

    // Получаем строку из данных.
    let tokenData: String = if let Some(tokenData) =
      token.getData().toString() { tokenData } else 
      { // Нет данных.
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
            // дальше значение зажимается в рамки структуры.
            let value: Option<u64> = match tokenData.parse::<u64>() 
            {
              Ok(value) => Some(value),
              Err(error) if *error.kind() == IntErrorKind::PosOverflow => Some(u64::MAX),
              Err(_) => None
            };
            match value 
            {
              None => token.setDefaultValue(structureType), // Не распарсилось — базовое значение.
              Some(value) => match structureType 
              {
                // Для float берём величину числа напрямую, без сжатия в u64.
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
            // дальше значение зажимается в рамки структуры.
            let value: Option<i64> = match tokenData.parse::<i64>() 
            {
              Ok(value) => Some(value),
              Err(error) if *error.kind() == IntErrorKind::NegOverflow => Some(i64::MIN),
              Err(error) if *error.kind() == IntErrorKind::PosOverflow => Some(i64::MAX),
              Err(_) => None
            };
            match value 
            {
              None => token.setDefaultValue(structureType), // Не распарсилось — базовое значение.
              Some(value) => match structureType 
              {
                // Для float берём величину числа напрямую, без сжатия в i64.
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
                  let normalized: String = 
                    if value < min { min.to_string() } else 
                    if value > 0 && (value as u64) > max { max.to_string() } else 
                    { value.to_string() };
                  token.setData( Bytes::from(normalized) );
                }
              }
            }
          }
          TokenType::UFloat | TokenType::Float => 
          {
            if let Ok(mut value) = tokenData.parse::<f64>() 
            { // Для UFloat обрезаем отрицательные до 0.
              if dataType == &TokenType::UFloat && value < 0.0 {
                value = 0.0;
              }
              if value.is_nan() 
              { // NaN — базовое значение;
                // Бесконечность зажмётся в границы типа (#71).
                token.setDefaultValue(structureType);
                return;
              }
              match structureType 
              {
                StructureType::F32 => {
                  value = value.clamp(f32::MIN as f64, f32::MAX as f64);
                  token.setData( Bytes::from((value as f32).to_string()) );
                }
                StructureType::F64 => {
                  value = value.clamp(f64::MIN, f64::MAX);
                  token.setData( Bytes::from(value.to_string()) );
                }
                // Приведение к целочисленным типам: округление и зажим в границы типа;
                // Отрицательное значение остаётся, если тип его принимает (I8 = -5.5 -> -6),
                // а для беззнаковых типов оно становится 0 (U8 = -5.5 -> 0).
                target => if let Some((min, max)) = Self::integerLimits(target) 
                {
                  let rounded: f64 = value.round();
                  let normalized: String = if rounded < 0.0 
                  { // Приведение f64 в i64 насыщается само, inf тоже.
                    (rounded as i64).max(min).to_string() 
                  } else 
                  { 
                    (rounded as u64).min(max).to_string() 
                  };
                  token.setData( Bytes::from(normalized) );
                }
              }
            } else 
            { // Не распарсилось — базовое значение.
              token.setDefaultValue(structureType);
            }
          }
          _ => {
            // Здесь пытаются прировнять что-то левое.
            token.setDefaultValue(structureType);
          }
          //
        }
      }
      _ => {
        // todo
        // Другие типы — ничего не делаем.
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
    // Создаём базовый токен.
    match structureType
    {
      // Целочисленные типы.
      StructureType::U8 | StructureType::U16 | StructureType::U32 | StructureType::U64 |
      StructureType::Usize | StructureType::I8 | StructureType::I16 | StructureType::I32 |
      StructureType::I64 | StructureType::Isize => {
        self.setDataType(TokenType::UInt);
        self.setData("0");
      }
      // Числа с плавающей точкой.
      StructureType::F32 | StructureType::F64 => {
        self.setDataType(TokenType::Float);
        self.setData("0.0");
      }
      StructureType::True => {
        self.setDataType(TokenType::True);
        self.setData("True");
      }
      StructureType::False => {
        self.setDataType(TokenType::False);
        self.setData("False");
      }
      // todo
      // Для остальных типов - ничего.
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
    
    // Строки: без этого значение без аннотации (`a := "text"`) получало None,
    // а токен очищался. Поля `pointer` и `length` тоже зависят от этого типа.
    // Идёт до чтения данных: у пустой строки `""` данных нет, но это всё равно String.
    match dataType
    {
      TokenType::String => return StructureType::String,
      TokenType::RawString => return StructureType::RawString,
      _ => {}
    }

    // Получаем строку из данных токена.
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
          // Больше U64: вид ошибки говорит, что это переполнение числа.
          Err(error) if *error.kind() == IntErrorKind::PosOverflow => 
          {
            newData = Some( u64::MAX.to_string() );
            StructureType::U64
          }
          // Что-то непонятное.
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
          // Меньше или больше I64: вид ошибки говорит, что это переполнение числа.
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
            // Что-то непонятное.
            _ => StructureType::None
          }
        }
      }
      TokenType::True => StructureType::True,
      TokenType::False => StructureType::False,
      TokenType::UFloat | TokenType::Float => 
      {
        match data.parse::<f64>() 
        {
          Ok(value) if value.is_nan() => StructureType::None,
          Ok(value) if value >= f32::MIN as f64 && value <= f32::MAX as f64 => StructureType::F32,
          Ok(value) => 
          { // Самый крайний тип; inf становится границей F64.
            if value.is_infinite() 
            {
              let border: f64 = if value.is_sign_negative() { f64::MIN } else { f64::MAX };
              newData = Some( format!("{:e}", border) );
            }
            StructureType::F64
          }
          // Что-то непонятное.
          Err(_) => StructureType::None
        }
      }
      // Для остальных типов - None, и токен очищается.
      // todo Сейчас могут попасть лишние т.к. они не объявлены выше.
      _ => StructureType::None,
    };

    if let Some(newData) = newData {
      self.setData( Bytes::from(newData) );
    }
    result(self, structureType)
    //
  }

  /// Вычисляет StructureType на основе токена;
  /// 
  /// Но делает это упрощенно по имени токена - т.е. когда мы явно знаем уже тип в строке.
  pub fn getStructureTypeSimple(&self) -> StructureType
  {
    let data: String = self.to_string();
    match data.as_str()
    {
      //
      "None" => StructureType::None,
      "Any" => StructureType::Any,
      "Link" => StructureType::Link,
      "True" => StructureType::True,
      "False" => StructureType::False,

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
      "CString" => StructureType::Pointer, // CString = Pointer (алиас).
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
