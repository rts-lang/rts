use super::structureType::{PrimitiveKind, StructureType};
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
  /// Является ли токен именем типа в `union`.
  ///
  /// - `a: U8 | String`
  /// - `a: 10 | 20`
  /// - `a: "text" | U8`
  fn isTypeName(token: &Token) -> bool
  {
    match token.getDataType()
    {
      // Идентификатор: U8, I32, Pointer, List и пользовательские типы.
      TokenType::Word => true,

      // Ключевые слова типов, у которых нет одноимённых примитивов.
      TokenType::None | TokenType::Any | TokenType::Link |
      TokenType::True | TokenType::False => true,

      // Ключевые слова, одноимённые с примитивами (`UInt` и `10`, `String` и `"abc"`).
      // Имя типа токенизируется БЕЗ данных, у примитива данные есть — это и есть
      // разница между `a: UInt | Int` (типы) и `a: 1 | 2` (значения-примитивы).
      TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float |
      TokenType::String | TokenType::RawString |
      TokenType::FormattedString | TokenType::FormattedRawString |
      TokenType::Char | TokenType::FormattedChar =>
      token.getData().toString().map(|data: String| data.is_empty()).unwrap_or(true),

      // Операторы, скобки, знаки препинания — типом быть не могут.
      _ => false
    }
  }

  /// Читает тип после `:` в объявлении.
  ///
  /// Например:
  /// - `a: U8`
  /// - `a: U8 | String`
  /// - `a: 10 | 20`
  /// - `a: 10`
  /// - `a: "text" | U8`
  /// - Тип не указан: `None`
  ///
  /// Повторы убираются: `U8 | U8` — это `U8`.
  fn fromPrimitiveToken(token: &Token) -> Option<Self>
  {
    let data: String = token.getData().toString()?;
    if data.is_empty() {
      return None;
    }

    match token.getDataType()
    {
      TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float =>
        Some(Self::Primitive(PrimitiveKind::Number, data)),
      TokenType::String | TokenType::RawString =>
        Some(Self::Primitive(PrimitiveKind::Text, data)),
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
    let Self::Primitive(kind, expected) = self else { return false };
    let Some(data) = token.getData().toString() else { return false };

    match (kind, token.getDataType())
    {
      (
        PrimitiveKind::Number,
        TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float
      ) => Self::numberEquals(expected, &data),
      (
        PrimitiveKind::Text,
        TokenType::String | TokenType::RawString
      ) => *expected == data,
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
      let found: Option<Self> = current.iter().find_map(|token: &Token|
        if Self::isTypeName(token) {
          Some(token.getStructureTypeSimple()) 
        } else {
          Self::fromPrimitiveToken(token)
        }
      );
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
      0 => Self::None, // Ни одного имени типа — тип не указан.
      1 => match variants.into_iter().next().unwrap()
      {
        primitive @ Self::Primitive(..) => Self::Union(vec![primitive]), // Примитив проверяется только через Union.
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
      Self::None => Vec::new(),
      other => vec![other.clone()]
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
      if variant == &StructureType::Any {
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
    StructureType::None
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
    if matches!(variant, StructureType::Primitive(..)) {
      return variant;
    }

    // Не подошло ни к одному варианту — токен очищается.
    if variant == StructureType::None
    {
      token.setDataType(TokenType::None);
      token.setData(None);
      return variant;
    }

    // Тип значения уже совпадает с вариантом — приводить нечего,
    // иначе зажимаем значение в границы варианта.
    if natural != variant && variant != StructureType::Any {
      Self::normalizeToken(token, variant.clone());
    }

    variant
  }
}

// =================================================================================================
