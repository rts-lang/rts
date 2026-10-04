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
//
// Поэтому нам следует "нормализовать" - т.е. привести к нужной форме 
// токены при хранении в структуре. На что указывает StructureType - 
// что вообще мы должны хранить и в каком виде в Structure.
//
// Это позволит TokenType -> StructureType на уровне типов.

// =================================================================================================

/// Тип данных структуры.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum StructureType
{
  None,
  Any,
  Link,

  /// Отдельный логический тип-литерал (#60 / #65).
  True,
  /// Отдельный логический тип-литерал (#60 / #65).
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
  Union(Vec<Self>)
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

      // Custom.
      Self::Custom(value) => value.clone()
    }
  }
}

// =================================================================================================

impl StructureType
{
  /// Является ли токен именем типа, пригодным для объединения (issue #59).
  ///
  /// Из type-секции после `:` берутся только настоящие имена типов.
  /// Числовые литералы (`a: 1 | 2`) и строковые литералы (`a: "name" | 10`)
  /// — это значения, а не типы, и они не поддерживаются.
  ///
  /// Ключевые слова типов (`UInt`, `Int`, `UFloat`, `Float`, `String`, `RawString`)
  /// токенизируются как токен БЕЗ данных — их имя живёт в самом TokenType.
  /// Одноимённые литералы (`10`, `"abc"`) — это уже значения с данными.
  /// Поэтому «имя типа» отличается от литерала именно наличием данных.
  fn isTypeName(token: &Token) -> bool
  {
    match token.getDataType()
    {
      // Идентификатор: U8, I32, Pointer, List и пользовательские типы.
      TokenType::Word => true,

      // Ключевые слова типов, у которых нет одноимённых литералов.
      TokenType::None | TokenType::Any | TokenType::Link |
      TokenType::True | TokenType::False => true,

      // Ключевые слова, одноимённые с литералами (`UInt` и `10`, `String` и `"abc"`).
      // Имя типа токенизируется БЕЗ данных, у литерала данные есть — это и есть
      // разница между `a: UInt | Int` (типы) и `a: 1 | 2` (значения, не поддерживается).
      TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float |
      TokenType::String | TokenType::RawString |
      TokenType::FormattedString | TokenType::FormattedRawString |
      TokenType::Char | TokenType::FormattedChar =>
      token.getData().toString().map(|data: String| data.is_empty()).unwrap_or(true),

      // Операторы, скобки, знаки препинания — типом быть не могут.
      _ => false
    }
  }

  /// Читает тип из type-секции объявления: `U8`, `String`, `U8 | String` (issue #59).
  ///
  /// Возвращает `StructureType::Union(vec![..])`, если вариантов несколько.
  /// Если после `|` идёт не имя типа, лишние варианты отбрасываются —
  /// union из не-типов не имеет смысла (#59: `a: "name"|10` не поддерживается).
  pub fn fromTypeTokens(typeTokens: &[Token]) -> Self
  {
    let mut variants: Vec<Self> = Vec::new();

    let mut current: Vec<Token> = Vec::new();
    let pushVariant = |variants: &mut Vec<Self>, current: &mut Vec<Token>| -> ()
    {
      if let Some(token) = current.iter().find(|token: &&Token| Self::isTypeName(token))
      {
        let variant: Self = token.getStructureTypeSimple();
        // Повторы и вложенные union'ы схлопываем в плоский список вариантов.
        match variant
        {
          Self::Union(nested) => variants.extend(nested),
          _ => if !variants.contains(&variant) { variants.push(variant); }
        }
      } // Не имя типа — просто пропускаем этот вариант.
      current.clear();
    };

    for token in typeTokens
    {
      // `|` разделяет варианты объединения.
      if *token.getDataType() == TokenType::Inclusion
      { pushVariant(&mut variants, &mut current); continue; }

      // `&` в объединении не несёт смысла, но и не должен ломать разбор.
      if *token.getDataType() == TokenType::Joint { continue; }

      current.push(token.clone());
    }
    pushVariant(&mut variants, &mut current);

    match variants.len()
    {
      0 => Self::None, // Ни одного имени типа — тип не указан.
      1 => variants.into_iter().next().unwrap(), // Один вариант — обычный тип, без Union.
      _ => Self::Union(variants)
    }
  }

  /// Варианты объединения; для не-Union — единственный сам тип.
  ///
  /// Так `Union([U8])` и `U8` ведут себя одинаково, а пустой список — как `None`.
  pub fn variants(&self) -> Vec<Self>
  {
    match self
    {
      Self::Union(variants) => variants.clone(),
      Self::None => Vec::new(),
      other => vec![other.clone()]
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

  /// Является ли вариант числовым (целочисленным или с плавающей точкой).
  pub const fn isNumeric(variant: &Self) -> bool
  {
    matches!(variant,
      Self::U8  | Self::U16  | Self::U32  |
      Self::U64 | Self::Usize |
      Self::I8  | Self::I16  | Self::I32  |
      Self::I64 | Self::Isize |
      Self::F32 | Self::F64
    )
  }

  /// Можно ли привести значение к этому варианту объединения.
  ///
  /// Число приводится к любому числовому варианту (с зажимом в границы типа, #71),
  /// строка — только к строковому. Всё остальное не приводится.
  const fn isConvertible(token: &Token, variant: &Self) -> bool
  {
    match token.getDataType()
    {
      TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float =>
        Self::isNumeric(variant),
      TokenType::String | TokenType::RawString =>
        matches!(variant, Self::String | Self::RawString),
      // Приводить нечего: точное совпадение уже было проверено отдельно.
      _ => false
    }
  }
}

// =================================================================================================

impl Structure
{
  /// Подбирает вариант объединения под значение токена (issue #59).
  ///
  /// Порядок такой же, как при объявлении обычного типа:
  /// 1. значение уже помещается в один из вариантов как есть — берём его;
  /// 2. иначе приводим в первый вариант, в который приведение возможно;
  /// 3. иначе — `None`, как и при неудачном приведении (#71).
  ///
  /// Возвращает выбранный вариант; `None` — значение не подошло ни к одному.
  pub fn matchUnion(token: &mut Token, union: &StructureType) -> StructureType
  {
    let variants: Vec<StructureType> = union.variants();

    // Пустое объединение = тип не указан, значение идёт как есть.
    if variants.is_empty()
    { return token.getStructureType(); }

    // 1. Значение уже имеет подходящий тип — приведение не нужно.
    //    Именно поэтому `a: U8 | I8 = -10` даёт I8, а не зажатое в U8 ноль.
    let natural: StructureType = StructureType::naturalType(token);
    if variants.contains(&natural)
    { return natural; }

    // 2. Ничего не подошло — приводим в первый подходящий вариант.
    for variant in variants.iter()
    {
      if variant == &StructureType::Any
      { return variant.clone(); } // Any принимает что угодно.
    }
    for variant in variants.iter()
    {
      if StructureType::isConvertible(token, variant)
      { return variant.clone(); }
    }

    // 3. Ни один вариант не подошёл и привести нельзя — константное поведение (#71).
    StructureType::None
  }

  /// Приводит токен к одному из вариантов объединения (issue #59).
  ///
  /// Значение, не подходящее ни к одному варианту, становится `None` —
  /// ровно так же, как неудачное приведение к обычному типу.
  pub fn normalizeUnion(token: &mut Token, union: &StructureType) -> StructureType
  {
    let variants: Vec<StructureType> = union.variants();

    if variants.is_empty()
    { return token.getStructureType(); }

    let natural: StructureType = StructureType::naturalType(token);
    let variant: StructureType = Self::matchUnion(token, &StructureType::Union(variants));

    // Не подошло ни к одному варианту — токен очищается.
    if variant == StructureType::None
    {
      token.setDataType(TokenType::None);
      token.setData(None);
      return variant;
    }

    // Тип значения уже совпадает с вариантом — приводить нечего,
    // иначе зажимаем значение в границы варианта.
    if natural != variant && variant != StructureType::Any
    {
      Self::normalizeToken(token, variant.clone());
    }

    variant
  }

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
                  let clamped: String = if rounded < 0.0 
                  { // Приведение f64 в i64 насыщается само, inf тоже.
                    (rounded as i64).max(min).to_string() 
                  } else 
                  { 
                    (rounded as u64).min(max).to_string() 
                  };
                  token.setData( Bytes::from(clamped) );
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
      // Для остальных типов - возвращаем Custom.
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

#[cfg(test)]
mod tests
{
  use crate::tokenizer::types::token::Token;
  use crate::tokenizer::types::tokenType::TokenType;
  use crate::tokenizer::types::line::Line;
  use crate::tokenizer::tools::splitByType::splitByType;
  use std::sync::{Arc, RwLock, RwLockReadGuard};
  use crate::parser::structure::structureType::StructureType;
  use crate::parser::structure::structure::Structure;
  use crate::parser::testing::check;
  // ===============================================================================================

  /// Проверяет тип и значение токена после getStructureType().
  fn checkToken(
    tokenType: TokenType, 
    data: &str, 
    expectedType: StructureType, 
    expectedData: &str
  ) -> ()
  {
    let mut token: Token = Token::new(tokenType, String::from(data));
    let structureType: StructureType = token.getStructureType();
    let tokenData: String = token.getData().toString().unwrap_or_default();
    assert!(
      structureType == expectedType && tokenData == expectedData,
      "For '{}' expected value '{}', got '{}' (type matched: {})",
      data, expectedData, tokenData, structureType == expectedType
    );
  }

  // ===============================================================================================

  /// Числа внутри рамок ABI не меняются.
  #[test]
  fn inRange()
  {
    checkToken(TokenType::UInt,   "255",                  StructureType::U8,  "255");
    checkToken(TokenType::UInt,   "18446744073709551615", StructureType::U64, "18446744073709551615");
    checkToken(TokenType::Int,    "-128",                 StructureType::I8,  "-128");
    checkToken(TokenType::Int,    "-9223372036854775808", StructureType::I64, "-9223372036854775808");
    checkToken(TokenType::UFloat, "1.5",                  StructureType::F32, "1.5");
  }

  /// За рамками ABI тип и значение становятся границей крайнего типа (#71).
  #[test]
  fn saturation() -> ()
  {
    // UInt: > u64 и очень большое число.
    checkToken(TokenType::UInt, "18446744073709551616", StructureType::U64, &u64::MAX.to_string());
    checkToken(TokenType::UInt, "99999999999999999999999999999999999999999999", StructureType::U64, &u64::MAX.to_string());
    // Int: < i64 и очень большое отрицательное число.
    checkToken(TokenType::Int, "-9223372036854775809", StructureType::I64, &i64::MIN.to_string());
    checkToken(TokenType::Int, "-99999999999999999999999999999999999999999999", StructureType::I64, &i64::MIN.to_string());
    // Int: > i64.
    checkToken(TokenType::Int, "9223372036854775808", StructureType::I64, &i64::MAX.to_string());
    // Float: inf после parse.
    checkToken(TokenType::UFloat, "1e309",  StructureType::F64, &format!("{:e}", f64::MAX));
    checkToken(TokenType::Float,  "-1e309", StructureType::F64, &format!("{:e}", f64::MIN));
  }

  /// Проверяет значение токена после normalizeToken() в явный тип.
  fn normalize(
    tokenType: TokenType, 
    data: &str, 
    structureType: StructureType, 
    expectedData: &str
  ) -> ()
  {
    let mut token: Token = Token::new(tokenType, String::from(data));
    Structure::normalizeToken(&mut token, structureType);
    let tokenData: String = token.getData().toString().unwrap_or_default();
    assert!(
      tokenData == expectedData,
      "For '{}' the value '{}' was expected, got '{}'",
      data, expectedData, tokenData
    );
  }

  /// Явный тип зажимает значение в свои границы, даже если число больше u64 и i64 (#71).
  #[test]
  fn normalizeClamp() -> ()
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

  /// Большие числа в float сохраняют величину, а не сжимаются в u64 (#71).
  #[test]
  fn normalizeFloat() -> ()
  {
    let big: &str = "44444444444444444444444444444444444444444444"; // 44 цифры
    normalize(TokenType::UInt, big, StructureType::F32, &f32::MAX.to_string());
    normalize(TokenType::UInt, big, StructureType::F64, &big.parse::<f64>().unwrap().to_string());
    // Бесконечность зажимается в границу типа.
    normalize(TokenType::Float, "-1e309", StructureType::F32, &f32::MIN.to_string());
    normalize(TokenType::UFloat, "1e309", StructureType::F64, &f64::MAX.to_string());
    normalize(TokenType::UFloat, "1e309", StructureType::U8, "255");
  }

  // ===============================================================================================
  
  /// Float в целый тип: округляется и зажимается в границы типа;
  /// 
  /// Отрицательное значение остаётся для знаковых типов и становится 0 для беззнаковых.
  #[test]
  fn normalizeFloatToInteger() -> ()
  {
    normalize(TokenType::UFloat, "5.4", StructureType::I8, "5");
    normalize(TokenType::UFloat, "5.5", StructureType::I8, "6");
    // Беззнаковые
    normalize(TokenType::Float, "-10.0", StructureType::U8, "0");
    normalize(TokenType::Float, "-5.5", StructureType::U8, "0");
    normalize(TokenType::Float, "-0.4", StructureType::U8, "0");
    // Знаковые принимают отрицательное, если оно в диапазоне.
    normalize(TokenType::Float, "-5.5", StructureType::I8, "-6");
    normalize(TokenType::Float, "-5.4", StructureType::I8, "-5");
    normalize(TokenType::Float, "-0.4", StructureType::I8, "0");
    normalize(TokenType::Float, "-1000000.7", StructureType::I64, "-1000001");
    // Вне диапазона - граница типа.
    normalize(TokenType::Float, "-200.5", StructureType::I8, "-128");
    normalize(TokenType::UFloat, "200.5", StructureType::I8, "127");
    normalize(TokenType::UFloat, "300.5", StructureType::U8, "255");
    normalize(TokenType::Float, "-1e309", StructureType::I64, &i64::MIN.to_string());
    normalize(TokenType::UFloat, "1e309", StructureType::I64, &i64::MAX.to_string());
    normalize(TokenType::UFloat, "1e309", StructureType::U64, &u64::MAX.to_string());
  }

  /// Не число - None, токен очищается.
  #[test]
  fn notNumber() -> ()
  {
    checkToken(TokenType::UInt,   "abc", StructureType::None, "");
    checkToken(TokenType::Int,    "-",   StructureType::None, "");
    checkToken(TokenType::UFloat, "NaN", StructureType::None, "");
  }

  // ===============================================================================================

  /// Сравнивает типы через to_string(): у StructureType нет Debug, а печать
  /// заодно показывает, как объединение выглядит в коде.
  fn isType(actual: StructureType, expected: StructureType) -> bool
  {
    assert!(
      actual == expected,
      "Expected type '{}', got '{}'",
      expected.to_string(), actual.to_string()
    );
    true
  }

  /// Разбирает type-секцию объявления в StructureType (issue #59).
  ///
  /// `a: U8` — это по-прежнему обычный одиночный тип, а не Union из одного
  /// элемента: иначе поменялось бы поведение всех существующих объявлений.
  fn parseType(code: &str) -> StructureType
  {
    let mut buffer: Vec<u8> = code.as_bytes().to_vec();
    let lines: Vec< Arc<RwLock<Line>> > = crate::tokenizer::tokenizer::readTokensSimple(&mut buffer);

    for line in lines
    {
      let line: RwLockReadGuard<Line> = line.read().unwrap();
      if let Some(tokens) = &line.tokens
      {
        // Отрезаем всё после `:` — это и есть type-секция.
        let typeTokens: Vec<Token> = match splitByType(tokens.clone(), &[TokenType::Colon])
        {
          parts if parts.len() == 2 =>
          parts[1].tokens.clone().unwrap_or_default(),
          _ => continue
        };
        return StructureType::fromTypeTokens(&typeTokens);
      }
    }
    StructureType::None
  }

  /// Проверяет, что значение легло в объединение: подходящий вариант и результат.
  fn union(
    tokenType: TokenType, 
    data: &str, 
    variants: Vec<StructureType>, 
    expectedType: &StructureType, 
    expectedData: &str
  )
  {
    let mut token: Token = Token::new(tokenType, String::from(data));
    let union: StructureType = StructureType::Union(variants.clone());
    let resultType: StructureType = Structure::normalizeUnion(&mut token, &union);

    let tokenData: String = token.getData().toString().unwrap_or_default();
    assert!(
      resultType == *expectedType && tokenData == expectedData,
      "For '{}' in '{}' the '{}' variant with value '{}' was expected, got '{}' with value '{}'",
      data, union.to_string(), expectedType.to_string(), expectedData, resultType.to_string(), tokenData
    );
  }

  // ===============================================================================================

  /// Один вариант — обычный тип; несколько — Union (issue #59).
  #[test]
  fn unionParse() -> ()
  {
    isType(parseType("a: U8 = 10"), StructureType::U8);
    isType(parseType("a: String = \"x\""), StructureType::String);
    isType(parseType("a: U8 | String = 10"),
      StructureType::Union(vec![StructureType::U8, StructureType::String]));
    isType(parseType("b: U8|String = 10"), // Без пробелов.
      StructureType::Union(vec![StructureType::U8, StructureType::String]));
    isType(parseType("a: I8 | U8 | F64 | None = 1"),
      StructureType::Union(vec![
        StructureType::I8, StructureType::U8, StructureType::F64, StructureType::None
      ]));
    // Ключевые слова-типы без данных - настоящие имена типов.
    isType(parseType("a: UInt | Int = 1"),
      StructureType::Union(vec![
        StructureType::Custom(String::from("UInt")), StructureType::Custom(String::from("Int"))
      ]));
    // Повторы схлопываются.
    isType(parseType("a: U8 | U8 | String = 10"),
      StructureType::Union(vec![StructureType::U8, StructureType::String]));
  }

  /// Литералы - это значения, а не типы: они не поддерживаются (issue #59).
  #[test]
  fn unionLiteralsAreNotTypes() -> ()
  {
    // Ни одного имени типа - тип не указан, объявление ведёт себя как `a = 10`.
    isType(parseType("a: 1 | 2 = 10"), StructureType::None);
    // Строковый литерал отбрасывается, `U8` остаётся единственным вариантом.
    isType(parseType("a: \"name\" | 10 = 10"), StructureType::None);
    isType(parseType("a: \"name\" | U8 = 10"), StructureType::U8);
    // Незакрытый `|` не ломает разбор.
    isType(parseType("a: U8 | = 10"), StructureType::U8);
  }

  /// Значение ложится в тот вариант, в который помещается как есть (issue #59).
  #[test]
  fn unionExactMatch() -> ()
  {
    let u8String: Vec<StructureType> = vec![StructureType::U8, StructureType::String];
    union(TokenType::UInt,   "10",  u8String.clone(), &StructureType::U8,     "10");
    union(TokenType::String, "hi",  u8String.clone(), &StructureType::String, "hi");
    union(TokenType::UFloat, "1.5", vec![StructureType::U8, StructureType::F32],
      &StructureType::F32, "1.5");

    // Из нескольких подходящих вариантов выбирается тот, в который значение.
    // помещается без потерь, а не первый в списке:
    // -10 помещается в I8, поэтому U8 | I8 даёт I8, а не зажатое в U8 ноль.
    union(TokenType::Int, "-10", vec![StructureType::U8, StructureType::I8],
      &StructureType::I8, "-10");
    union(TokenType::UInt, "70000", vec![StructureType::U8, StructureType::U32],
      &StructureType::U32, "70000");
  }

  /// Не подошёл ни один вариант - приводим в первый, куда приведение возможно (#59/#71).
  #[test]
  fn unionConvert() -> ()
  {
    let u8String: Vec<StructureType> = vec![StructureType::U8, StructureType::String];
    // 300 не помещается в U8, но приводится в него с зажимом (#71).
    union(TokenType::UInt, "300",  u8String.clone(), &StructureType::U8, "255");
    // Отрицательное в беззнаковый - тоже приведение, не совпадение.
    union(TokenType::Int,  "-10",  u8String.clone(), &StructureType::U8, "0");
    // Float приводится в целый вариант с округлением.
    union(TokenType::UFloat, "1.5", u8String.clone(), &StructureType::U8, "2");
  }

  /// Ни один вариант не подошёл и привести нельзя - None (issue #59).
  #[test]
  fn unionNone() -> ()
  {
    let u8String: Vec<StructureType> = vec![StructureType::U8, StructureType::String];
    // True/False не приводятся ни к числу, ни к строке.
    union(TokenType::True, "True",  u8String.clone(), &StructureType::None, "");
    union(TokenType::Link, "a.b",   u8String.clone(), &StructureType::None, "");
    // Явный `| None` в объединении: значение не подходит - всё равно None.
    union(TokenType::True, "True",  vec![StructureType::String, StructureType::None],
      &StructureType::None, "");
    // Само None в объединении - законный вариант.
    union(TokenType::None, "", vec![StructureType::String, StructureType::None],
      &StructureType::None, "");
  }

  /// Объединение не должно ломать обычные одиночные типы.
  #[test]
  fn unionSingleVariantBehavesLikeType() -> ()
  {
    // Union из одного варианта - это просто этот тип.
    let one: StructureType = StructureType::Union(vec![StructureType::U8]);
    isType(StructureType::Union(one.variants()), StructureType::Union(vec![StructureType::U8]));
    union(TokenType::UInt, "300", vec![StructureType::U8], &StructureType::U8, "255");
    // Пустое объединение равносильно отсутствию типа.
    assert!(
      StructureType::Union(vec![]).variants().is_empty(),
      "An empty union should produce an empty list of variants"
    );
  }

  /// Union печатается так же, как записывается в коде.
  #[test]
  fn unionToString() -> ()
  {
    assert_eq!(
      StructureType::Union(vec![StructureType::U8, StructureType::String]).to_string(),
      "U8 | String"
    );
    assert_eq!(
      StructureType::Union(vec![StructureType::I8, StructureType::F64, StructureType::None]).to_string(),
      "I8 | F64 | None"
    );
    // Вложенный union схлопывается при разборе, но и сам печатается нормально.
    assert_eq!(StructureType::Union(vec![
      StructureType::Union(vec![StructureType::U8, StructureType::String]),
      StructureType::U16
    ]).to_string(), "U8 | String | U16");
  }

  // ===============================================================================================

  /// Число без знака получает наименьший подходящий ABI тип.
  #[test]
  fn autoUnsigned() -> ()
  {
    // U8
    check!("a~~ = 0", a, type TokenType::UInt, stype StructureType::U8, val "0");
    check!("a = 255", a, type TokenType::UInt, stype StructureType::U8, val "255");
    // U16
    check!("a = 256", a, type TokenType::UInt, stype StructureType::U16, val "256");
    check!("a = 65535", a, type TokenType::UInt, stype StructureType::U16, val "65535");
    // U32
    check!("a = 65536", a, type TokenType::UInt, stype StructureType::U32, val "65536");
    check!("a = 4294967295", a, type TokenType::UInt, stype StructureType::U32, val "4294967295");
    // U64
    check!("a = 4294967296", a, type TokenType::UInt, stype StructureType::U64, val "4294967296");
    check!("a = 18446744073709551615", a, type TokenType::UInt, stype StructureType::U64, val "18446744073709551615");
  }

  /// Число со знаком получает наименьший подходящий ABI тип.
  #[test]
  fn autoSigned() -> ()
  {
    // I8
    check!("a = -1", a, type TokenType::Int, stype StructureType::I8, val "-1");
    check!("a = -128", a, type TokenType::Int, stype StructureType::I8, val "-128");
    // I16
    check!("a = -129", a, type TokenType::Int, stype StructureType::I16, val "-129");
    check!("a = -32768", a, type TokenType::Int, stype StructureType::I16, val "-32768");
    // I32
    check!("a = -32769", a, type TokenType::Int, stype StructureType::I32, val "-32769");
    check!("a = -2147483648", a, type TokenType::Int, stype StructureType::I32, val "-2147483648");
    // I64
    check!("a = -2147483649", a, type TokenType::Int, stype StructureType::I64, val "-2147483649");
    check!("a = -9223372036854775808", a, type TokenType::Int, stype StructureType::I64, val "-9223372036854775808");
  }

  /// Число за крайним ABI типом становится его границей (#71).
  #[test]
  fn autoSaturation() -> ()
  {
    // Больше U64
    check!("a = 18446744073709551616", a, type TokenType::UInt, stype StructureType::U64, val &u64::MAX.to_string());
    // Меньше I64
    check!("a = -9223372036854775809", a, type TokenType::Int, stype StructureType::I64, val &i64::MIN.to_string());
  }

  /// Usize и Isize не выводятся автоматически, только явным типом.
  /// Они зависят от платформы.
  #[test]
  fn explicitPlatform() -> ()
  {
    check!("b: Usize = 18446744073709551616", b, type TokenType::UInt, stype StructureType::Usize, val &usize::MAX.to_string());
    check!("c: Isize = -9223372036854775809", c, type TokenType::Int, stype StructureType::Isize, val &isize::MIN.to_string());
  }

  /// Дробное число получает F32, если помещается, иначе F64.
  /// Значения в `.rt` не сверяются - только type и stype.
  #[test]
  fn autoFloat() -> ()
  {
    // f32
    check!("a = 0.0", a, type TokenType::UFloat, stype StructureType::F32);
    check!("a = 3.4028234663852886e38", a, type TokenType::UFloat, stype StructureType::F32);
    check!("a = -3.4028234663852886e38", a, type TokenType::Float, stype StructureType::F32);
    // f64
    check!("a = 1.7976931348623157e308", a, type TokenType::UFloat, stype StructureType::F64);
    check!("a = -1.7976931348623157e308", a, type TokenType::Float, stype StructureType::F64);
  }

  /// Дробное число за F64 остаётся границей F64 (#71).
  /// Значения в `.rt` не сверяются - только type и stype.
  #[test]
  fn autoFloatSaturation() -> ()
  {
    check!("a = 1.7976931348623157e309", a, type TokenType::UFloat, stype StructureType::F64);
    check!("a = -1.7976931348623157e309", a, type TokenType::Float, stype StructureType::F64);
  }

  // ===============================================================================================

  /// Явный целый тип зажимает значение в свои границы.
  #[test]
  fn castInteger() -> ()
  {
    check!("a: U8 = 300", a, type TokenType::UInt, stype StructureType::U8, val "255");
    check!("b: U8 = -10", b, type TokenType::Int, stype StructureType::U8, val "0");
    check!("c: I8 = -300", c, type TokenType::Int, stype StructureType::I8, val "-128");
    check!("d: U64 = 18446744073709551616", d, type TokenType::UInt, stype StructureType::U64, val "18446744073709551615");
    check!("e: Usize = 18446744073709551616", e, type TokenType::UInt, stype StructureType::Usize, val "18446744073709551615");
    check!("f: Isize = -9223372036854775809", f, type TokenType::Int, stype StructureType::Isize, val "-9223372036854775808");
  }

  /// Число за u64/i64: сначала граница u64/i64, затем зажим в явный тип (#71).
  #[test]
  fn castBigNumbers() -> ()
  {
    check!("g: U8 = 44444444444444444444444444444444444444444444", g, type TokenType::UInt, stype StructureType::U8, val "255");
    check!("h: I8 = 44444444444444444444444444444444444444444444", h, type TokenType::UInt, stype StructureType::I8, val "127");
    check!("i: I64 = -44444444444444444444444444444444444444444444", i, type TokenType::Int, stype StructureType::I64, val "-9223372036854775808");
    check!("j: Usize = 44444444444444444444444444444444444444444444", j, type TokenType::UInt, stype StructureType::Usize, val "18446744073709551615");
    check!("k: Isize = -44444444444444444444444444444444444444444444", k, type TokenType::Int, stype StructureType::Isize, val "-9223372036854775808");
  }

  /// Огромное число в F32 и Float за F64 в F32/U8: зажим в границу целевого типа.
  #[test]
  fn castFloats() -> ()
  {
    check!("l: F32 = 44444444444444444444444444444444444444444444", l, type TokenType::UInt, stype StructureType::F32, val "340282350000000000000000000000000000000");
    check!("m: F32 = -1.7976931348623157e309", m, type TokenType::Float, stype StructureType::F32, val "-340282350000000000000000000000000000000");
    check!("n: U8 = 1.7976931348623157e309", n, type TokenType::UFloat, stype StructureType::U8, val "255");
  }

  /// Float в целый явный тип: округляется и зажимается;
  /// знак сохраняется для знаковых типов, беззнаковые дают 0.
  #[test]
  fn castFloatToInteger() -> ()
  {
    check!("q: I8 = 5.4", q, type TokenType::UFloat, stype StructureType::I8, val "5");
    check!("r: I8 = 5.5", r, type TokenType::UFloat, stype StructureType::I8, val "6");
    check!("s: U8 = -10.0", s, type TokenType::Float, stype StructureType::U8, val "0");
    check!("t: U8 = -5.5", t, type TokenType::Float, stype StructureType::U8, val "0");
    check!("u: I8 = -5.5", u, type TokenType::Float, stype StructureType::I8, val "-6");
    check!("v: I8 = -5.4", v, type TokenType::Float, stype StructureType::I8, val "-5");
    check!("w: I8 = -200.5", w, type TokenType::Float, stype StructureType::I8, val "-128");
    check!("x: I64 = -1000000.7", x, type TokenType::Float, stype StructureType::I64, val "-1000001");
  }

  // ===============================================================================================
}

// =================================================================================================