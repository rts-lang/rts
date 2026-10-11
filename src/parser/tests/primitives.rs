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

/// `m = import(...)` из `/tmp`: есть `m.a`; примитивы на `main` на месте.
#[test]
fn importModule() -> ()
{
  let path: std::path::PathBuf = std::env::temp_dir().join("importModule.rt");
  std::fs::write(&path, "a = 10\n").unwrap();
  let pathStr: String = path.to_string_lossy().replace('\\', "/");
  let code: String = format!("m = import(\"{pathStr}\")\n");
  let main: Arc<RwLock<Structure>> = runCode(&code);

  checkStructure!(main, "m.a", value "10");

  let _ = std::fs::remove_file(&path);

  for name in MethodsPrimitives
  {
    checkStructure!(main, *name, stype StructureType::Method, mut StructureMut::Constant, lines 0);
  }
  for (name, dataType) in TypesPrimitives
  {
    checkStructure!(main, *name, stype dataType.clone(), mut StructureMut::Constant, lines 0);
  }
}

// =================================================================================================
