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
  Ptr,  // указатель (raw)

  // todo Требует удаление для FFI-ABI?
  Method,
  // todo Требует удаление для FFI-ABI?
  List, // todo List<Type>
  
// native
  // todo Требует удаление для FFI-ABI?
  /// Нативные внешние штуки
  Native,

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
    { // primitives
      StructureType::None => String::from("None"),
      StructureType::Any => String::from("Any"),
      StructureType::Link => String::from("Link"),

      StructureType::Bool => String::from("Bool"),

      StructureType::U8 => String::from("U8"),
      StructureType::U16 => String::from("U16"),
      StructureType::U32 => String::from("U32"),
      StructureType::U64 => String::from("U64"),
      StructureType::Usize => String::from("Usize"),
      
      StructureType::I8 => String::from("I8"),
      StructureType::I16 => String::from("I16"),
      StructureType::I32 => String::from("I32"),
      StructureType::I64 => String::from("I64"),
      StructureType::Isize => String::from("Isize"),
      
      StructureType::F32 => String::from("F32"),
      StructureType::F64 => String::from("F64"),
      
      StructureType::Ptr => String::from("Ptr"),

      StructureType::Method => String::from("Method"),
      StructureType::List => String::from("List"),
      
      // native
      StructureType::Native => String::from("Native"),

      // custom
      StructureType::Custom(value) => value.clone(),
    }
  }
}

// =================================================================================================

impl Token
{
  /// Приводит данные токена в рамки требуемого StructureType,
  /// чтобы Structure смог его безопасно хранить;
  /// 
  /// Это делается только при присвоении или при изменении.
  /// 
  /// todo Есть идея ввести на уровне Structure в будущем bool,
  ///  чтобы можно было проверить и оптимизировать эту работу.
  pub fn normalizeToStructure(&self, structureType: StructureType) -> ()
  {
    // Если грубо - это приводит токен в рамки того что может хранить тип структуры

    unimplemented!()
  }
  
  /// Вычисляет тип структуры на основе токена;
  /// Удобно когда нет рамок структуры и нужно понять, 
  /// что в него положили, но при этом в рамках StructureType.
  pub fn getStructureType(&self) -> StructureType 
  {
    // Но нам нужен метод который вычислит тип структуры автоматически на основе токена
    // Его token type + data

    // Пример:
    // Если токен UInt, смотрим значение: U8, U16, U32, U64, Usize
    // Если токен Int, аналогично I8, I16, ...
    // Если Float -> F32 или F64
    
    unimplemented!()
  }
}

impl TokenType
{
  /// Преобразует TokenType под StructureType;
  /// Удобно если есть Custom или известный StructureType как строка.
  pub fn toStructureType(&self) -> StructureType
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
      "Ptr" => StructureType::Ptr,

      // Служебные
      // todo Под вопросом
      "Method" => StructureType::Method,
      "List" => StructureType::List,
      "Native" => StructureType::Native,

      // Всё остальное — кастомное
      _ => StructureType::Custom(data),
    }
    //
  }
}

// =================================================================================================