use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::StructureType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use crate::parser::parser::runCode;
// =================================================================================================

// `Примитивные` структуры встраиваются в `main`.
//
// По сути это просто самые:
// - Простые;
// - Нужные для основы языка;
// - Самые неизменяемые.
//
// Они строятся из `StructureType` списка, но не для всех - а только самые базовые.
//
// Также есть `примитивы-методы`. todo Надо добавить.

// =================================================================================================

/// todo desc + можно ли его заменить на что-то уже существующее?
fn assertBuiltin(name: &str, expected: StructureType) -> ()
{
  let main: Arc<RwLock<Structure>> = runCode("");
  let guard: RwLockReadGuard<_> = main.read().unwrap();
  let link: Arc<RwLock<Structure>> = guard.getStructureByScope(name)
    .unwrap_or_else(|| panic!("builtin `{name}` not found as structure"));
  let structure: RwLockReadGuard<Structure> = link.read().unwrap();
  
  assert!(
    structure.mutable == StructureMut::Constant,
    "{name} must be Constant, got {}",
    structure.mutable.to_string()
  );
  assert!(
    structure.dataType == expected,
    "{name} dataType: expected {}, got {}",
    expected.to_string(),
    structure.dataType.to_string()
  );
  assert!(
    structure.name.as_deref() == Some(name),
    "{name} name mismatch: {:?}",
    structure.name
  );
}

// =================================================================================================

/// Проверяет все структуры `примитивы`.
#[test]
fn structures() -> ()
{
  let cases: &[(&str, StructureType)] = &[
    //
    ("U8",    StructureType::U8),
    ("U16",   StructureType::U16),
    ("U32",   StructureType::U32),
    ("U64",   StructureType::U64),
    //
    ("I8",    StructureType::I8),
    ("I16",   StructureType::I16),
    ("I32",   StructureType::I32),
    ("I64",   StructureType::I64),
    //
    ("F32",   StructureType::F32),
    ("F64",   StructureType::F64),
    //
    ("Usize", StructureType::Usize),
    ("Isize", StructureType::Isize),
    
    //
    ("Pointer", StructureType::Pointer),
    
    //
    ("String", StructureType::String),
    ("RawString", StructureType::RawString),
    ("CString", StructureType::Pointer)
  ];
  for (name, expected) in cases
  {
    assertBuiltin(name, expected.clone());
  }
}

// =================================================================================================
