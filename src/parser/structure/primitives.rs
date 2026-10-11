use crate::parser::structure::structureType::StructureType;
// =================================================================================================

// Реестр структур-примитивов.
//
// Их не стоит называть: `примитивные` - потому что это означает то, что они простые.
// Хотя отчасти это правда, то что они простые, но это неверное определение и название.
//
// Смысл в том что они: `примитив` - т.е. базовая единица языка.

// =================================================================================================

/// Типы-примитивы.
pub const TypesPrimitives: &[(&str, StructureType)] = &[
  //
  ("U8", StructureType::U8),
  ("U16", StructureType::U16),
  ("U32", StructureType::U32),
  ("U64", StructureType::U64),
  //
  ("I8", StructureType::I8),
  ("I16", StructureType::I16),
  ("I32", StructureType::I32),
  ("I64", StructureType::I64),
  //
  ("F32", StructureType::F32),
  ("F64", StructureType::F64),
  //
  ("Usize", StructureType::Usize),
  ("Isize", StructureType::Isize),
  //
  ("Pointer", StructureType::Pointer),
  //
  ("String", StructureType::String),
  ("RawString", StructureType::RawString),
  //
  ("CString", StructureType::Pointer),
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
  MethodsPrimitives.iter().any(|n| *n == name)
}

// =================================================================================================
