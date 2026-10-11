use crate::parser::parser::runCode;
use crate::parser::structure::primitives::{MethodsPrimitives, TypesPrimitives};
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
  for (name, dataType) in TypesPrimitives
  {
    checkStructure!("", *name, stype dataType.clone(), mut StructureMut::Constant, lines 0);
  }
}

// =================================================================================================

/// Проверяет все структуры `примитивы-методы`.
#[test]
fn methods() -> ()
{
  for name in MethodsPrimitives
  {
    checkStructure!("", *name, stype StructureType::Method, mut StructureMut::Constant, lines 0);
  }
}

/// Попытка перезаписать примитив.
#[test]
fn overwrite() -> ()
{
  checkStructure!("type := 5", "type", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  checkStructure!("type = 5", "type", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  checkStructure!("type~ := 5", "type", stype StructureType::Method, mut StructureMut::Constant, lines 0);
  checkStructure!("type~~ := 5", "type", stype StructureType::Method, mut StructureMut::Constant, lines 0);
}

// =================================================================================================

/// Блок `type { a }` не создаёт вторую структуру с именем примитива.
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

  assert_eq!(typeCount, 1);
}

// =================================================================================================

/// `a = import(...)` получает `Module` без имени у временной структуры.
#[test]
fn importModule() -> ()
{
  let path: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/release/native/include/simple/a.rt");
  let code: String = format!("a = import(\"{path}\")\n");
  let main: Arc<RwLock<Structure>> = runCode(&code);
  let a: Arc<RwLock<Structure>> = main.read().unwrap()
    .getStructureByScope("a")
    .expect("a");

  let stype: String = a.read().unwrap().dataType.to_string();
  assert_eq!(stype, "Module");

  let children: Vec<Arc<RwLock<Structure>>> =
    a.read().unwrap().structures.read().unwrap().clone().unwrap_or_default();
  let names: Vec<String> = children.iter()
    .filter_map(|s| s.read().unwrap().name.clone())
    .collect();
  assert!(names.iter().any(|n| n == "hello"));
  assert!(names.iter().any(|n| n == "value"));
  assert!(!names.iter().any(|n| n == "import"));
}

// =================================================================================================
