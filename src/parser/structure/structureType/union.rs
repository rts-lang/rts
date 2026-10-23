use super::structureType::{PrimitiveKind, PrimitiveType, StructureType};
use crate::parser::structure::structure::Structure;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;

// =================================================================================================

// Здесь логика `union` для типизации структур.
//
// `union` может состоять из `StructureType`, примитивов, custom структур.
//
// Применение `union` к структуре позволяет содержать в ней эти значения.
//
//
// Повторяющиеся типы объединяются т.к. это дубликаты.
//
//
// `None` автоматически может содержаться при `константном` или `неопределенном` поведении;
// Поэтому он никогда не указывается в `union` или в качестве типизации для структур.
// todo Хотя вроде как `a: None = ...` имеет смысл и можно указывать? Поэтому или можно убрать?
//  Имеется ввиду, что ожидание `None` может быть явное без удаления структуры и поэтому так.
//
// todo `Any` - сейчас не используется. По логике он позволяет содержать любой тип.

// =================================================================================================

impl StructureType
{
  /// Читает вариант типа из одного токена `type-секции` (после `:`).
  ///
  /// Токен в `type-секции` бывает трёх видов:
  /// - `StructureType`, пользовательский тип — это обычное слово;
  /// - `True`/`False`/`None` — отдельные токены;
  /// - Значение-римитив: `10`, `"text"` — число или строка, у которых есть данные.
  /// 
  /// Их можно смешивать или указывать по одиночке при указании типа структуры.
  fn fromTypeToken(token: &Token) -> Option<Self>
  {
    match token.getDataType()
    {
      // Имя типа: обычное слово — `U8`, `String`, `Any` или пользовательский тип.
      TokenType::Word => Some(token.getStructureTypeSimple()),

      // `True` и `False` — имена типов, но с данными.
      TokenType::True | TokenType::False => Some(token.getStructureTypeSimple()),

      // `None` — токен без данных, но это всё равно имя типа.
      TokenType::None => Some(Self::Primitive(PrimitiveType::None)),

      // Примитив-значение: число.
      TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float =>
      {
        let data: String = token.getData().toString()?;
        Some(Self::PrimitiveValue(PrimitiveKind::Number, data))
      },

      // Примитив-значение: строка.
      // Пустая строка (`""`) — тоже значение, хотя данных у неё нет.
      TokenType::String | TokenType::RawString =>
      {
        let data: String = token.getData().toString().unwrap_or_default();
        Some(Self::PrimitiveValue(PrimitiveKind::Text, data))
      },

      // Операторы, скобки, знаки препинания, ссылки — типом быть не могут.
      _ => None
    }
  }

  /// Числа равны по значению, а не по записи: `10`, `10.0` и `010` - одно значение.
  fn numberEquals(left: &str, right: &str) -> bool
  {
    if let (Ok(left), Ok(right)) = (left.parse::<i128>(), right.parse::<i128>())
    {
      return left == right;
    }

    match (left.parse::<f64>(), right.parse::<f64>())
    {
      (Ok(left), Ok(right)) => left == right,
      _ => left == right
    }
  }

  /// Подходит ли значение токена под этот вариант-примитив: только точное совпадение.
  ///
  /// Для всех вариантов, кроме `Primitive`, всегда `false`.
  fn acceptsValue(&self, token: &Token) -> bool
  {
    let Self::PrimitiveValue(kind, expected) = self else { return false };

    match (kind, token.getDataType())
    {
      (
        PrimitiveKind::Number,
        TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float
      ) => match token.getData().toString()
      {
        Some(data) => Self::numberEquals(expected, &data),
        None => false
      },
      (
        PrimitiveKind::Text,
        TokenType::String | TokenType::RawString
      ) =>
      { // Пустая строка — это тоже значение, поэтому пустое сравниваем с пустым.
        let data: String = token.getData().toString().unwrap_or_default();
        *expected == data
      },
      _ => false
    }
  }

  /// Читает тип `StructureType` или `union` типов
  /// для типизации структуры: `U8`, `String`, `U8 | String`.
  ///
  /// Возвращает `StructureType::Union(vec![..])`, если вариантов несколько;
  /// Варианты - это имена типов или примитивы: `a: 10 | 20`, `a: "text" | U8`.
  /// Всё остальное в варианте (операторы, скобки) отбрасывается.
  ///
  /// Одиночный примитив `a: 10` возвращается как `Union` из одного варианта,
  /// иначе он не прошёл бы проверку значения: обычный тип этого не делает.
  pub fn fromTypeTokens(typeTokens: &[Token]) -> Self
  {
    let mut variants: Vec<Self> = Vec::new();

    let mut current: Vec<Token> = Vec::new();
    let pushVariant = |variants: &mut Vec<Self>, current: &mut Vec<Token>| -> ()
    {
      let found: Option<Self> = current.iter().find_map(|token: &Token| Self::fromTypeToken(token));
      if let Some(variant) = found
      {
        // Повторы и вложенные union'ы схлопываем в плоский список вариантов.
        match variant
        {
          Self::Union(nested) => variants.extend(nested),
          _ => if !variants.contains(&variant) {
            variants.push(variant);
          }
        }
      } // Ни имя типа, ни примитив — просто пропускаем этот вариант.
      current.clear();
    };

    for token in typeTokens
    {
      // `|` разделяет варианты объединения.
      if *token.getDataType() == TokenType::Inclusion
      {
        pushVariant(&mut variants, &mut current);
        continue;
      }

      // `&` в объединении не несёт смысла, но и не должен ломать разбор.
      if *token.getDataType() == TokenType::Joint {
        continue;
      }

      current.push(token.clone());
    }
    pushVariant(&mut variants, &mut current);

    match variants.len()
    {
      0 => Self::Primitive(PrimitiveType::None), // Ни одного имени типа — тип не указан.
      1 => match variants.into_iter().next().unwrap()
      {
        primitive @ Self::PrimitiveValue(..) => Self::Union(vec![primitive]), // Примитив проверяется только через Union.
        single => single // Один вариант — обычный тип, без Union.
      },
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
      Self::Primitive(PrimitiveType::None) => Vec::new(),
      other => vec![other.clone()]
    }
  }

  /// Является ли вариант числовым (целочисленным или с плавающей точкой).
  pub const fn isNumeric(variant: &Self) -> bool
  {
    matches!(variant,
      Self::Primitive(PrimitiveType::U8)  | Self::Primitive(PrimitiveType::U16)  | Self::Primitive(PrimitiveType::U32)  |
      Self::Primitive(PrimitiveType::U64) | Self::Primitive(PrimitiveType::Usize) |
      Self::Primitive(PrimitiveType::I8)  | Self::Primitive(PrimitiveType::I16)  | Self::Primitive(PrimitiveType::I32)  |
      Self::Primitive(PrimitiveType::I64) | Self::Primitive(PrimitiveType::Isize) |
      Self::Primitive(PrimitiveType::F32) | Self::Primitive(PrimitiveType::F64)
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
        matches!(variant, Self::Primitive(PrimitiveType::String) | Self::Primitive(PrimitiveType::RawString)),
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
  /// 0. значение точно совпало с примитивом (`a: 10 | 20`) — берём его, без приведения;
  /// 1. значение уже помещается в один из вариантов как есть — берём его;
  /// 2. иначе приводим в первый вариант, в который приведение возможно;
  /// 3. иначе — `None`, как и при неудачном приведении (#71).
  ///
  /// `None` не нужно указывать в объединении: это скрытый вариант, он выбирается
  /// всегда, когда не подошёл ни один из указанных.
  ///
  /// Возвращает выбранный вариант (в том числе `Primitive`); `None` — значение не подошло ни к одному.
  pub fn matchUnion(token: &mut Token, union: &StructureType) -> StructureType
  {
    let variants: Vec<StructureType> = union.variants();

    // Пустое объединение = тип не указан, значение идёт как есть.
    if variants.is_empty() {
      return token.getStructureType();
    }

    // 0. Примитив: только точное совпадение, и раньше типов - `a: U8 | 10 = 10`
    //    сохраняет именно значение 10, а не просто "число, помещающееся в U8".
    //    Проверка идёт до naturalType, пока токен не зажат границами (#71).
    for variant in variants.iter()
    {
      if variant.acceptsValue(token) {
        return variant.clone();
      }
    }

    // 1. Значение уже имеет подходящий тип — приведение не нужно.
    //    Именно поэтому `a: U8 | I8 = -10` даёт I8, а не зажатое в U8 ноль.
    let natural: StructureType = StructureType::naturalType(token);
    if variants.contains(&natural) {
      return natural;
    }

    // 2. Ничего не подошло — приводим в первый подходящий вариант.
    for variant in variants.iter()
    { // Any принимает что угодно.
      if variant == &StructureType::Primitive(PrimitiveType::Any) {
        return variant.clone();
      } 
    }
    for variant in variants.iter()
    {
      if StructureType::isConvertible(token, variant) {
        return variant.clone();
      }
    }

    // 3. Ни один вариант не подошёл и привести нельзя — константное поведение (#71).
    StructureType::Primitive(PrimitiveType::None)
  }

  /// Приводит токен к одному из вариантов объединения (issue #59).
  ///
  /// Значение, не подходящее ни к одному варианту, становится `None` —
  /// ровно так же, как неудачное приведение к обычному типу.
  pub fn normalizeUnion(token: &mut Token, union: &StructureType) -> StructureType
  {
    let variants: Vec<StructureType> = union.variants();

    if variants.is_empty() {
      return token.getStructureType();
    }

    let variant: StructureType = Self::matchUnion(token, &StructureType::Union(variants));
    let natural: StructureType = StructureType::naturalType(token);

    // Примитив - выбранный вариант типизации: токен (значение) не трогаем,
    // dataType/stype становится самим примитивом (`a: 10 | 20 = 10` -> stype 10).
    // ABI-форма — через abiType() там, где нужен String/U8/...
    if matches!(variant, StructureType::PrimitiveValue(..)) {
      return variant;
    }

    // Не подошло ни к одному варианту — токен очищается.
    if variant == StructureType::Primitive(PrimitiveType::None)
    {
      token.setDataType(TokenType::None);
      token.setData(None);
      return variant;
    }

    // Тип значения уже совпадает с вариантом — приводить нечего,
    // иначе зажимаем значение в границы варианта.
    if natural != variant && variant != StructureType::Primitive(PrimitiveType::Any) {
      Self::normalizeToken(token, variant.clone());
    }

    variant
  }
}

// =================================================================================================
