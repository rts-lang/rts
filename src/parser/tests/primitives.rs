use crate::parser::structure::primitives::{ArgsPrimitives, MethodsPrimitives};
use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::parser::testing::checkStructure;
use crate::parser::testing::runCode;
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
  for (name, dataType) in PrimitiveType::iter()
  {
    checkStructure!(
      "", name,
      stype StructureType::Primitive(dataType.clone()),
      mut Constant,
      lines 0
    );
  }
}

// =================================================================================================

/// Проверяет структуры `параметры запуска`: у них есть значения (линии).
///
/// В тестах аргументов нет: `argc` равен `0`, а у `argv` нет ни одной линии.
#[test]
fn args() -> ()
{
  for (name, dataType) in ArgsPrimitives 
  {
    checkStructure!(
      "", *name,
      stype StructureType::Primitive(dataType.clone()),
      mut Constant
    );
  }
  checkStructure!("", "argc", value "0", lines 1);
  checkStructure!("", "argv", lines 0);

  // Перезаписать их нельзя.
  checkStructure!(
    "argc := 5", "argc",
    value "0",
    mut Constant
  );
  checkStructure!(
    "argc = 5", "argc",
    value "0",
    mut Constant
  );

  // И удалить через `:= None` тоже.
  checkStructure!(
    "argc := None", "argc",
    value "0",
    mut Constant
  );
  checkStructure!(
    "argv := None", "argv",
    stype List,
    mut Constant
  );
}

// =================================================================================================

/// Проверяет все структуры `примитивы-методы`.
#[test]
fn methods() -> ()
{
  for name in MethodsPrimitives
  {
    checkStructure!(
      "", *name, 
      stype Method,
      mut Constant,
      lines 0
    );
  }
}

/// Попытка перезаписать `примитив`.
#[test]
fn overwrite() -> ()
{
  checkStructure!(
    "type := 5", "type",
    stype Method,
    mut Constant,
    lines 0
  );
  checkStructure!(
    "type := 5", "type",
    stype Method,
    mut Constant,
    lines 0
  ); // todo Дубликат строка?
  checkStructure!(
    "type~ := 5", "type",
    stype Method,
    mut Constant,
    lines 0
  );
  checkStructure!(
    "type~~ := 5", "type",
    stype Method,
    mut Constant,
    lines 0
  );
}

// =================================================================================================

/// Блок `type { a }` не создаёт вторую структуру с именем примитива.
#[test]
fn block() -> ()
{
  // 1. Проверка маркера.
  checkStructure!(
    r#"
    a := 5
    type { a }
    "#,
    "type",
    stype Method,
    mut Constant,
    lines 0
  );

  // 2. Проверка параллельной структуры с тем же именем.
  let mainLink: Arc<RwLock<Structure>> = runCode!(r#"
    a = 5
    type { a }
  "#);
  let main: RwLockReadGuard<_> = mainLink.read().unwrap();
  let children: Vec<Arc<RwLock<Structure>>> =
    main.structures.read().unwrap().clone().unwrap_or_default();

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
  std::fs::write(&path, "a := 10\n").unwrap();
  let pathString: String = path.to_string_lossy().replace('\\', "/");
  let code: String = format!("m := import(\"{pathString}\")\n");
  let mainLink: Arc<RwLock<Structure>> = runCode!(&code);

  checkStructure!(mainLink, "m.a", value "10");

  let _ = std::fs::remove_file(&path);

  for name in MethodsPrimitives
  {
    checkStructure!(
      mainLink, *name,
      stype Method,
      mut Constant,
      lines 0
    );
  }
  for (name, dataType) in PrimitiveType::iter()
  {
    checkStructure!(
      mainLink, name,
      stype StructureType::Primitive(dataType.clone()),
      mut Constant,
      lines 0
    );
  }
}

// =================================================================================================
