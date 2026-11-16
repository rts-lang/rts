use crate::parser::bytes::Bytes;
use crate::parser::structure::structureType::PrimitiveType;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use serde::{Deserialize, Serialize};
use std::num::IntErrorKind;
// =================================================================================================

// Идея простая - т.к. мы имеем хранение в токенах, то это абстрактные данные;
// Вообще сам по себе токен бесконечен;
//
// Когда из `токенов` ⟶ `линии` ⟶ `блоки` цепочки появляется `Structure`, то он
// привязан к FFI уже, поэтому требует определенные физические типы;
//
// Внутри это все еще токены, но через `StructureType` - мы контролируем их.
//
//
// Поэтому нам следует "нормализовать" - т.е. привести к нужной форме 
// токены при хранении в `Structure`;
// 
// `StructureType` как раз указывает - как мы должны хранить токены и в каком виде.

// =================================================================================================

/// Вид примитива-значения.
/// 
/// Они также могут контактировать при перечислении через `|` уже в [`StructureType::Union`].
/// 
/// Для FFI их значения преобразовываются отдельно т.к. не являются типами-примитивами.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveValueKind
{
  /// Число;
  /// 
  /// `a: 10 = 10`.
  Number,
  
  /// Строка;
  /// 
  /// `a: "text" = "text"`.
  String
}

// =================================================================================================

/// Тип данных структуры.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum StructureType
{
  /// Примитивные и пользовательские типы.
  /// 
  /// Формально пользовательские типы это не примитив, но они должны находиться в этой группе.
  Primitive(PrimitiveType),

  /// Примитив-значение как граница типизации: `a: 10 = 10`.
  ///
  /// Это ограничение значения - допустимо только точное совпадение.
  PrimitiveValue(PrimitiveValueKind, String),
  
  /// Объединение типов: `a: U8 | String = ...`.
  ///
  /// Хранится не "пересечение", а список допустимых вариантов,
  /// и в структуре единовременно лежит ровно один из них.
  ///
  /// **Семантика**:
  /// - При присваивании выбирается тот вариант, в который значение
  ///   помещается как есть, либо в который оно приводится;
  /// - Если ни один вариант не подходит и привести нельзя — `None`;
  /// - При полной мутабельности `~~` тип может меняться, поэтому объединение
  ///   можно указать, но оно ничего не ограничивает.
  ///
  /// Пустой список вариантов равносилен `None`.
  ///
  /// Вариантом может быть и [`StructureType::PrimitiveValue`]: `a: 10 | 20`, `a: "text" | U8`.
  Union(Vec<Self>),
  
  /// Модуль из `import(...)`; `id` — ключ lookup без имени структуры.
  ///
  /// Это правильно т.к. поиск по имени может уйти за структуру, а `id`
  /// позволяет остаться только в `self` scope.
  Module(usize)
}

// =================================================================================================

// todo Можно заменить данные на keywords из words.rs, 
//   но их не хватит т.к. тут есть другие, 
//   + здесь structure type
impl ToString for StructureType
{ // todo convert ⟶ fmt::Display ?
  fn to_string(&self) -> String
  {
    match self
    { //
      Self::Primitive(PrimitiveType::None) => String::from("None"),
      Self::Primitive(PrimitiveType::Any) => String::from("Any"),
      
      //
      Self::Primitive(PrimitiveType::True) => String::from("True"),
      Self::Primitive(PrimitiveType::False) => String::from("False"),

      // Беззнаковые.
      Self::Primitive(PrimitiveType::U8) => String::from("U8"),
      Self::Primitive(PrimitiveType::U16) => String::from("U16"),
      Self::Primitive(PrimitiveType::U32) => String::from("U32"),
      Self::Primitive(PrimitiveType::U64) => String::from("U64"),
      // U128 нет т.к. это не FFI совместимый тип данных (зависит от `chillffi`).
      Self::Primitive(PrimitiveType::Usize) => String::from("Usize"),

      // Знаковые.
      Self::Primitive(PrimitiveType::I8) => String::from("I8"),
      Self::Primitive(PrimitiveType::I16) => String::from("I16"),
      Self::Primitive(PrimitiveType::I32) => String::from("I32"),
      Self::Primitive(PrimitiveType::I64) => String::from("I64"),
      // I128 нет т.к. это не FFI совместимый тип данных (зависит от `chillffi`).
      Self::Primitive(PrimitiveType::Isize) => String::from("Isize"),

      // Плавающие.
      // F16 нет т.к. это не FFI совместимый тип данных (зависит от `chillffi`).
      Self::Primitive(PrimitiveType::F32) => String::from("F32"),
      Self::Primitive(PrimitiveType::F64) => String::from("F64"),

      // Указатель.
      Self::Primitive(PrimitiveType::Pointer) => String::from("Pointer"),
      // Алиас от `Pointer`, но для FFI - отдельный примитив (см. `abiType`).
      Self::Primitive(PrimitiveType::CString) => String::from("CString"),
      Self::Primitive(PrimitiveType::RawString) => String::from("RawString"),
      Self::Primitive(PrimitiveType::String) => String::from("String"),

      //
      Self::Primitive(PrimitiveType::Method) => String::from("Method"),
      Self::Primitive(PrimitiveType::List) => String::from("List"),

      // Пользовательский тип.
      Self::Primitive(PrimitiveType::Custom(value)) => value.clone(),

      // Объединение типов.
      Self::Union(variants) => variants
        .iter()
        .map(|variant: &Self| variant.to_string())
        .collect::<Vec<String>>()
        .join(" | "),

      // Примитив-значение.
      Self::PrimitiveValue(PrimitiveValueKind::Number, value) => value.clone(),
      Self::PrimitiveValue(PrimitiveValueKind::String, value) => format!("\"{}\"", value),

      // Module.
      Self::Module(_) => String::from("Module")
    }
  }
}

// =================================================================================================

impl StructureType
{
  /// ABI / storage тип: примитив сводится к natural-форме значения.
  ///
  /// `Primitive(String, _)` ⟶ `String`; `Primitive(Number, v)` ⟶ наименьший числовой
  /// ABI-тип для `v`; `Primitive(CString)` ⟶ `Pointer` (алиас); всё остальное — как есть.
  /// 
  /// Нужен там, где `dataType` участвует в FFI, string-полях и `castToken`, а не в `stype`.
  /// 
  /// todo Я бы изменил название и desc и так далее. Потому что слишком непонятная штука.
  pub fn abiType(&self) -> Self
  {
    match self
    {
      //
      Self::PrimitiveValue(PrimitiveValueKind::String, _) => Self::Primitive(PrimitiveType::String),
      // `CString` - алиас от `Pointer`: в ABI-форме это указатель.
      Self::Primitive(PrimitiveType::CString) => Self::Primitive(PrimitiveType::Pointer),
      //
      Self::PrimitiveValue(PrimitiveValueKind::Number, value) =>
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
      TokenType::None => Self::Primitive(PrimitiveType::None),
      TokenType::Any => Self::Primitive(PrimitiveType::Any),
      TokenType::True => Self::Primitive(PrimitiveType::True),
      TokenType::False => Self::Primitive(PrimitiveType::False),
      TokenType::String => Self::Primitive(PrimitiveType::String),
      TokenType::RawString => Self::Primitive(PrimitiveType::RawString),
      // Числа: ширину и насыщение считает сам getStructureType.
      TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float =>
        token.getStructureType(),
      // Неизвестное — трактуем как отсутствие значения.
      _ => Self::Primitive(PrimitiveType::None)
    }
  }

  /// Приводит `Token` в рамки `примитива`.
  pub fn castToken(&self, token: &mut Token) -> ()
  {
    match self
    {
      // Примитив: просто каст.
      Self::Primitive(primitive) => primitive.castToken(token),
      
      // Для остальных ничего не делаем.
      _ => {}
    }
  }
  
  // ===============================================================================================
}

// =================================================================================================

impl Token
{
  // ===============================================================================================
  
  /// Вычисляет [`StructureType`] на основе токена.
  /// 
  /// Удобно, когда нет рамок структуры (не указан её тип) и нужно понять, 
  /// что в неё положили, но при этом в рамках [`StructureType`].
  /// 
  /// Если станет `None` - то токен будет очищен.
  /// 
  /// todo Было бы хорошо убрать как-то mut отсюда.
  pub fn getStructureType(&mut self) -> StructureType
  {
    fn result(selfToken: &mut Token, structureType: StructureType) -> StructureType
    {
      if structureType == StructureType::Primitive(PrimitiveType::None) {
        selfToken.setData(None);
      }
      structureType
    }
    
    //
    let dataType: &TokenType = self.getDataType();
    
    // Строки: без этого значение без аннотации (`a := "text"`) получало `None`,
    // а токен очищался.
    // 
    // Поля `pointer` и `length` тоже зависят от этого типа.
    //
    // Идёт до чтения данных: у пустой строки `""` данных нет, но это всё равно `String`.
    match dataType
    {
      TokenType::String => return StructureType::Primitive(PrimitiveType::String),
      TokenType::RawString => return StructureType::Primitive(PrimitiveType::RawString),
      _ => {}
    }

    // Получаем строку из данных токена.
    let data: String = if let Some(string) = self.getData().toString() {
      string
    } else {
      return result(self, StructureType::Primitive(PrimitiveType::None))
    };

    // Токены `UInt`/`Int`/`UFloat`/`Float` бесконечны, а ABI типы структур ограничены;
    //
    // Если число вышло за самый крайний ABI тип, то это константное поведение:
    // тип становится крайним (`U64`, `I64`, `F64`), а значение - его границей;
    //
    // Здесь хранится новое значение, если оно изменилось.
    let mut newData: Option<String> = None;

    let structureType: StructureType = match dataType 
    {
      TokenType::None => StructureType::Primitive(PrimitiveType::None),
      TokenType::Any => StructureType::Primitive(PrimitiveType::Any),
      //
      TokenType::True => StructureType::Primitive(PrimitiveType::True),
      TokenType::False => StructureType::Primitive(PrimitiveType::False),
      // Ссылка - не значение, а указатель на структуру: своего типа у неё нет,
      // но данные сохраняются - ссылка должна остаться ссылкой.
      TokenType::Link => return StructureType::Primitive(PrimitiveType::None),
      //
      TokenType::UInt => 
      {
        match data.parse::<u64>() 
        {
          Ok(value) if value <= u8::MAX  as u64 => StructureType::Primitive(PrimitiveType::U8),
          Ok(value) if value <= u16::MAX as u64 => StructureType::Primitive(PrimitiveType::U16),
          Ok(value) if value <= u32::MAX as u64 => StructureType::Primitive(PrimitiveType::U32),
          Ok(_) => StructureType::Primitive(PrimitiveType::U64),
          // Больше `U64`: вид ошибки говорит, что это переполнение числа.
          Err(error) if *error.kind() == IntErrorKind::PosOverflow => 
          {
            newData = Some( u64::MAX.to_string() );
            StructureType::Primitive(PrimitiveType::U64)
          }
          // Что-то непонятное.
          Err(_) => StructureType::Primitive(PrimitiveType::None)
        }
      }
      TokenType::Int => 
      {
        match data.parse::<i64>() 
        {
          Ok(value) if value >= i8::MIN  as i64 && value <= i8::MAX  as i64 => StructureType::Primitive(PrimitiveType::I8),
          Ok(value) if value >= i16::MIN as i64 && value <= i16::MAX as i64 => StructureType::Primitive(PrimitiveType::I16),
          Ok(value) if value >= i32::MIN as i64 && value <= i32::MAX as i64 => StructureType::Primitive(PrimitiveType::I32),
          Ok(_) => StructureType::Primitive(PrimitiveType::I64),
          // Меньше или больше `I64`: вид ошибки говорит, что это переполнение числа.
          Err(error) => match error.kind() 
          {
            IntErrorKind::NegOverflow => 
            {
              newData = Some( i64::MIN.to_string() );
              StructureType::Primitive(PrimitiveType::I64)
            }
            IntErrorKind::PosOverflow => 
            {
              newData = Some( i64::MAX.to_string() );
              StructureType::Primitive(PrimitiveType::I64)
            }
            // Что-то непонятное.
            _ => StructureType::Primitive(PrimitiveType::None)
          }
        }
      }
      //
      TokenType::UFloat | TokenType::Float => 
      {
        match data.parse::<f64>() 
        {
          Ok(value) if value.is_nan() => StructureType::Primitive(PrimitiveType::None),
          Ok(value) if value >= f32::MIN as f64 && value <= f32::MAX as f64 => StructureType::Primitive(PrimitiveType::F32),
          Ok(value) => 
          { // Самый крайний тип; `inf` становится границей `F64`.
            if value.is_infinite() 
            {
              let border: f64 = if value.is_sign_negative() { f64::MIN } else { f64::MAX };
              newData = Some( format!("{:e}", border) );
            }
            StructureType::Primitive(PrimitiveType::F64)
          }
          // Что-то непонятное.
          Err(_) => StructureType::Primitive(PrimitiveType::None)
        }
      }
      // Для остальных типов - `None`, и токен очищается.
      //
      // todo Сейчас могут попасть лишние т.к. они не объявлены выше.
      _ => StructureType::Primitive(PrimitiveType::None)
    };

    if let Some(newData) = newData {
      self.setData( Bytes::from(newData) );
    }
    result(self, structureType)
    //
  }

  /// Вычисляет `[StructureType`] на основе токена;
  /// 
  /// Но делает это упрощенно по имени токена - т.е. когда мы явно знаем уже тип в строке.
  /// 
  /// Например: токен хранит `U64` ⟶ получаем `StructureType::U64`.
  pub fn getStructureTypeSimple(&self) -> StructureType
  {
    let tokenData: String = self.to_string();
    let structureTypeName: &str = tokenData.as_str();
    
    if PrimitiveType::isPrimitive(structureTypeName) {
      StructureType::Primitive(
        PrimitiveType::fromName(structureTypeName).unwrap() // Здесь точно есть т.к. проверено `is` выше.
      )
    } else
    { // Всё остальное — `custom`.
      StructureType::Primitive(PrimitiveType::Custom(tokenData))
    }
    //
  }

  // ===============================================================================================
}

// =================================================================================================
