use super::structureType::{PrimitiveValueKind, PrimitiveType, StructureType};
use crate::parser::structure::structure::Structure;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;

// =================================================================================================

// Логика `union` для типизации структур.
//
//
// По сути это объединение возможных значений для хранения в структуре;
//
// `Union` может состоять из `вариантов`:
// - `StructureType`, это примитивы;
// - Пользовательских структур; todo Не уверен что оно работает или есть, но должно быть.
// - Значение-римитив [`StructureType::PrimitiveValue`].
//
// Повторяющиеся типы объединяются т.к. это дубликаты.
//
//
// `None` - не тип, а состояние: он автоматически может содержаться у любой структуры
// при `константном` или `неопределенном` поведении, а хранение и удаление `None` описаны отдельно
// (`a = None` хранит значение, `a := None` удаляет структуру).
// Поэтому в `union` и в типизацию структур он не вписывается: парсер просто его отбрасывает;
// `a: None` - это `a`, а `a: U8 | None` - это `a: U8`.
//
// todo `Any` - сейчас не используется. По логике он позволяет содержать любой тип.

// =================================================================================================

impl StructureType
{
  // ===============================================================================================
  
  /// Читает `вариант` типа из одного токена `type-секции` (после `:`).
  ///
  /// Токен в `type-секции` бывает:
  /// - `StructureType`, это примитивы; Пользовательский тип; Эти два - это обычное слово;
  /// - `True`/`False`/`None` — отдельные токены;
  /// - Значение-римитив [`StructureType::PrimitiveValue`].
  /// 
  /// Их можно смешивать или указывать по одиночке при указании типа структуры.
  fn fromTypeToken(token: &Token) -> Option<Self>
  {
    match token.getDataType()
    {
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      // Имя типа: обычное слово — `U8`, `String`, `Any` или пользовательский тип.
      TokenType::Word => Some(token.getStructureTypeSimple()),

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      // `True` и `False` — имена типов, но с данными.
      TokenType::True | TokenType::False => Some(token.getStructureTypeSimple()),
      
      // `None` — токен без данных, но это всё равно имя типа.
      TokenType::None => Some(Self::Primitive(PrimitiveType::None)),

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      // Примитив-значение: число.
      TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float =>
      {
        let data: String = token.getData().toString()?;
        Some(Self::PrimitiveValue(PrimitiveValueKind::Number, data))
      },

      // Примитив-значение: строка.
      // Пустая строка (`""`) — тоже значение, хотя данных у неё нет.
      TokenType::String | TokenType::RawString =>
      {
        let data: String = token.getData().toString().unwrap_or_default();
        Some(Self::PrimitiveValue(PrimitiveValueKind::String, data))
      },

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

      // Все остальное - не входит.
      _ => None

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    }
  }

  // ===============================================================================================

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

  /// Подходит ли значение токена под это `примитив-значение`;
  /// [`StructureType::PrimitiveValue`] требует только точное совпадение.
  ///
  /// Для всех других вариантов всегда `false`.
  fn acceptsValue(&self, token: &Token) -> bool
  {
    let Self::PrimitiveValue(kind, expected) = self else { return false };

    match (kind, token.getDataType())
    {
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      // Number.
      (
        PrimitiveValueKind::Number,
        TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float
      ) => match token.getData().toString()
      {
        Some(data) => Self::numberEquals(expected, &data),
        None => false
      },

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      // String.
      (
        PrimitiveValueKind::String,
        TokenType::String | TokenType::RawString
      ) =>
      { // Пустая строка — это тоже значение, поэтому пустое сравниваем с пустым.
        let data: String = token.getData().toString().unwrap_or_default();
        *expected == data
      },

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      _ => false

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    }
  }

  // ===============================================================================================

  /// Читает тип `StructureType` или `union` типов
  /// для типизации структуры: `U8`, `String`, `U8 | String`.
  ///
  /// Возвращает `StructureType::Union(vec![..])`, если вариантов несколько;
  /// `Варианты` - это имена типов или примитивы: `a: 10 | 20`, `a: "text" | U8`;
  /// Всё остальное отбрасывается.
  ///
  /// Одиночный примитив `a: 10` возвращается как `union` из одного варианта,
  /// иначе он не прошёл бы проверку значения: обычный тип этого не делает.
  ///
  /// `None` - не явный примитив, поэтому как `вариант` он отбрасывается:
  /// `a: None` даёт то же, что и `a` без типа, а `a: U8 | None` - то же, что `a: U8`;
  /// Но фактически `None` может храниться как отсутствие значения;
  /// Более того именно `None` не указывается ещё потому, что он везде отбрасывается.
  pub fn fromTypeTokens(typeTokens: &[Token]) -> Self
  {
    let mut variants: Vec<Self> = Vec::new();

    let mut current: Vec<Token> = Vec::new();
    let pushVariant = |variants: &mut Vec<Self>, current: &mut Vec<Token>| -> ()
    {
      let found: Option<Self> = current.iter().find_map(|token: &Token| Self::fromTypeToken(token))
        // `None` - не явный примитив: его не вписывают ни в `union`, ни в типизацию;
        // такой `вариант` просто выбрасывается, как и всё, что не является типом.
        .filter(|variant: &Self| *variant != Self::Primitive(PrimitiveType::None));
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

  // ===============================================================================================

  /// Варианты объединения; для `не-union` — единственный сам тип.
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

  // ===============================================================================================

  /// Является ли вариант числовым (целочисленным или с плавающей точкой).
  pub const fn isNumeric(variant: &Self) -> bool
  {
    matches!(variant,
      Self::Primitive(PrimitiveType::U8)  | Self::Primitive(PrimitiveType::U16)  |
      Self::Primitive(PrimitiveType::U32)  | Self::Primitive(PrimitiveType::U64) |
      Self::Primitive(PrimitiveType::Usize) |
      //
      Self::Primitive(PrimitiveType::I8)  | Self::Primitive(PrimitiveType::I16)  |
      Self::Primitive(PrimitiveType::I32)  | Self::Primitive(PrimitiveType::I64) |
      Self::Primitive(PrimitiveType::Isize) |
      //
      Self::Primitive(PrimitiveType::F32) | Self::Primitive(PrimitiveType::F64)
    )
  }

  /// Можно ли привести значение к этому `варианту` для `union`.
  ///
  /// Число приводится к любому числовому `варианту` (с зажимом в границы `StructureType`),
  /// строка — только к строковому; Всё остальное не приводится.
  const fn isConvertible(token: &Token, variant: &Self) -> bool
  {
    match token.getDataType()
    {
      //
      TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float =>
        Self::isNumeric(variant),
      //
      TokenType::String | TokenType::RawString =>
        matches!(
          variant,
          Self::Primitive(PrimitiveType::String) | Self::Primitive(PrimitiveType::RawString)
        ),
      // Приводить нечего: точное совпадение уже было проверено отдельно.
      _ => false
    }
  }

  // ===============================================================================================
}

// =================================================================================================

impl Structure
{
  /// Подбирает `вариант` для `union` под значение токена.
  ///
  /// Порядок такой же, как при объявлении обычного типа:
  ///  1. Значение точно совпало с примитивом (`a: 10 | 20`) — берём его, без приведения;
  ///  2. Значение уже помещается в один из `вариантов` как есть — берём его;
  ///  3. Иначе приводим в первый `вариант`, в который приведение возможно;
  ///  4. Иначе — `None`, как и при неудачном приведении.
  ///
  /// Возвращает выбранный вариант (в том числе `Primitive`);
  ///
  /// `None` не нужно указывать в объединении: это скрытый вариант, он выбирается
  /// всегда, когда не подошёл ни один из указанных или при неудаче.
  pub fn matchUnion(token: &mut Token, union: &StructureType) -> StructureType
  {
    let variants: Vec<StructureType> = union.variants();

    // Пустой `union` - тип не указан, значение идёт как есть.
    if variants.is_empty() {
      return token.getStructureType();
    }

    // 0. Примитив-значение: только точное совпадение, и раньше типов - `a: U8 | 10 = 10`
    //    сохраняет именно значение `10`, а не просто "число, помещающееся в U8".
    //    Проверка идёт до `naturalType`, пока токен не зажат границами.
    for variant in variants.iter()
    {
      if variant.acceptsValue(token) {
        return variant.clone();
      }
    }

    // 1. Значение уже имеет подходящий тип — приведение не нужно.
    //    Именно поэтому `a: U8 | I8 = -10` даёт `I8`, а не зажатое в `U8` ноль.
    let natural: StructureType = StructureType::naturalType(token);
    if variants.contains(&natural) {
      return natural;
    }

    // 2. Ничего не подошло — приводим в первый подходящий вариант.
    for variant in variants.iter()
    { // `Any` принимает что угодно.
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

    // 3. Ни один `вариант` не подошёл и привести нельзя — константное поведение на `None`.
    StructureType::Primitive(PrimitiveType::None)
  }

  /// Приводит токен к одному из `вариантов` для `union`.
  ///
  /// Значение, не подходящее ни к одному `варианту`, становится `None` —
  /// ровно так же, как неудачное приведение к обычному типу.
  pub fn normalizeUnion(token: &mut Token, union: &StructureType) -> StructureType
  {
    let variants: Vec<StructureType> = union.variants();

    if variants.is_empty() {
      return token.getStructureType();
    }

    let variant: StructureType = Self::matchUnion(token, &StructureType::Union(variants));
    let natural: StructureType = StructureType::naturalType(token);

    // `Примитив-значение` - выбранный вариант типизации: токен (значение) не трогаем,
    // `dataType`/`stype` становится самим примитивом (`a: 10 | 20 = 10` -> `stype 10`).
    // ABI-форма — через `abiType()` там, где нужен `String/U8/...`.
    if matches!(variant, StructureType::PrimitiveValue(..)) {
      return variant;
    }

    // Не подошло ни к одному `варианту` — токен очищается.
    if variant == StructureType::Primitive(PrimitiveType::None)
    {
      token.setDataType(TokenType::None);
      token.setData(None);
      return variant;
    }

    // Тип значения уже совпадает с `вариантом` — приводить нечего,
    // иначе зажимаем значение в границы `варианта`.
    if natural != variant && variant != StructureType::Primitive(PrimitiveType::Any) {
      Self::normalizeToken(token, variant.clone());
    }

    variant
  }
}

// =================================================================================================
