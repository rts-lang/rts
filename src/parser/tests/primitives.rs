use crate::parser::parser::runCode;
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::StructureType;
use crate::parser::testing::checkStructure;
use std::sync::{Arc, RwLock, RwLockReadGuard};
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
// Также есть `примитивы-методы`: функции и процедуры.

// =================================================================================================

/// Проверяет все структуры `примитивы`.
#[test]
fn structures() -> ()
{
  //
  checkStructure!("", "U8", stype StructureType::U8, mut StructureMut::Constant, lines 0);
  checkStructure!("", "U16", stype StructureType::U16, mut StructureMut::Constant, lines 0);
  checkStructure!("", "U32", stype StructureType::U32, mut StructureMut::Constant, lines 0);
  checkStructure!("", "U64", stype StructureType::U64, mut StructureMut::Constant, lines 0);
  //
  checkStructure!("", "I8", stype StructureType::I8, mut StructureMut::Constant, lines 0);
  checkStructure!("", "I16", stype StructureType::I16, mut StructureMut::Constant, lines 0);
  checkStructure!("", "I32", stype StructureType::I32, mut StructureMut::Constant, lines 0);
  checkStructure!("", "I64", stype StructureType::I64, mut StructureMut::Constant, lines 0);
  //
  checkStructure!("", "F32", stype StructureType::F32, mut StructureMut::Constant, lines 0);
  checkStructure!("", "F64", stype StructureType::F64, mut StructureMut::Constant, lines 0);
  //
  checkStructure!("", "Usize", stype StructureType::Usize, mut StructureMut::Constant, lines 0);
  checkStructure!("", "Isize", stype StructureType::Isize, mut StructureMut::Constant, lines 0);
  //
  checkStructure!("", "Pointer", stype StructureType::Pointer, mut StructureMut::Constant, lines 0);
  //
  checkStructure!("", "String", stype StructureType::String, mut StructureMut::Constant, lines 0);
  checkStructure!("", "RawString", stype StructureType::RawString, mut StructureMut::Constant, lines 0);
  checkStructure!("", "CString", stype StructureType::Pointer, mut StructureMut::Constant, lines 0);
}

// =================================================================================================

/// Проверяет все структуры `примитивы-методы`.
/// 
/// todo Тут бы проход по массиву.
#[test]
fn methods() -> ()
{
  //
  checkStructure!("", "type", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  checkStructure!("", "stype", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  checkStructure!("", "utype", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  checkStructure!("", "mut", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  checkStructure!("", "len", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  //
  checkStructure!("", "println", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  checkStructure!("", "print", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  checkStructure!("", "go", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  checkStructure!("", "exit", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  //
  checkStructure!("", "import", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  checkStructure!("", "importNative", stype StructureType::Method, mut StructureMut::Constant, lines 0);
}

/// Попытка перезаписать примитив.
/// 
/// todo По идее лучше расширить на все примитивы? Или 1 достаточно будет?
#[test]
fn overwrite() -> ()
{
  checkStructure!("type := 5", "type", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  checkStructure!("type = 5", "type", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  checkStructure!("type~ := 5", "type", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  checkStructure!("type~~ := 5", "type", stype StructureType::Method, mut StructureMut::Constant, lines 0);
}

// =================================================================================================

/// todo desc
#[test]
fn block() -> ()
{
  // 1. Проверка маркера.
  checkStructure!("a = 5\ntype { a }\n", "type", stype StructureType::Method, mut StructureMut::Constant, lines 0);

  // 2. Проверка параллельной структуры с тем же именем.
  let main: Arc<RwLock<Structure>> = runCode("a = 5\ntype { a }\n");
  let guard: RwLockReadGuard<_> = main.read().unwrap();
  let children: Vec<Arc<RwLock<Structure>>> =
    guard.structures.read().unwrap().clone().unwrap_or_default();

  let typeCount: usize = children.iter()
    .filter(|s| s.read().unwrap().name.as_deref() == Some("type"))
    .count();

  assert_eq!(
    typeCount,
    1,
    "reserved name must not create a parallel structure: expected 1 structure named 'type', got {typeCount}"
  );
}

// =================================================================================================
