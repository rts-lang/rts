use crate::parser::structure::structureType::PrimitiveType;
// =================================================================================================

// Реестр структур-примитивов.
//
// Их не стоит называть: `примитивные` - потому что это означает то, что они простые.
// Хотя отчасти это правда, то что они простые, но это неверное определение и название.
//
// Смысл в том что они: `примитив` - т.е. базовая единица языка.

// =================================================================================================

/// Типы-примитивы.
pub const TypesPrimitives: &[(&str, PrimitiveType)] = &[
  //
  ("Any", PrimitiveType::Any),
  ("None", PrimitiveType::None),
  //
  //
  ("True", PrimitiveType::True),
  ("False", PrimitiveType::False),
  //
  ("U8", PrimitiveType::U8),
  ("U16", PrimitiveType::U16),
  ("U32", PrimitiveType::U32),
  ("U64", PrimitiveType::U64),
  //
  ("I8", PrimitiveType::I8),
  ("I16", PrimitiveType::I16),
  ("I32", PrimitiveType::I32),
  ("I64", PrimitiveType::I64),
  //
  ("F32", PrimitiveType::F32),
  ("F64", PrimitiveType::F64),
  //
  ("Usize", PrimitiveType::Usize),
  ("Isize", PrimitiveType::Isize),
  //
  ("Pointer", PrimitiveType::Pointer),
  //
  ("String", PrimitiveType::String),
  ("CString", PrimitiveType::CString),
  ("RawString", PrimitiveType::RawString),
  // todo Под вопросом.
  ("Method", PrimitiveType::Method),
  ("List", PrimitiveType::List)
];

/// Параметры запуска.
///
/// В отличие от остальных у них есть линии (значения), и приходят они от запуска:
/// `argc` - сколько аргументов, `argv` - список самих аргументов.
pub const ArgsPrimitives: &[(&str, PrimitiveType)] = &[
  //
  ("argc", PrimitiveType::Usize),
  ("argv", PrimitiveType::List)
];

/// Методы-примитивы.
pub const MethodsPrimitives: &[&str] = &[
  //
  "type", "stype", "utype", "mut", "len",

  //
  "println", "print", "go", "exit",

  //
  "import", "importNative",
];

/// Имя занято структурой-примитивом.
#[inline]
pub fn isPrimitiveName(name: &str) -> bool
{
  TypesPrimitives.iter().any(|(n, _)| *n == name) ||
  ArgsPrimitives.iter().any(|(n, _)| *n == name) ||
  MethodsPrimitives.iter().any(|n| *n == name)
}

// =================================================================================================
