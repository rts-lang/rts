use crate::parser::parser::runCode;
use crate::parser::structure::structure::Structure;
use crate::parser::testing::checkStructure;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

// Тесты на изоляцию вызовов методов через глубокое клонирование.
//
// Каждый вызов метода работает с собственной копией — state не протекает
// между вызовами, изменения body снаружи видны только следующему вызову.

// =================================================================================================
// Изоляция state между вызовами.

/// Два вызова возвращают разные значения; раньше `r` протекала.
#[test]
fn noStateLeakBetweenCalls() -> ()
{
  checkStructure!(
    r#"
    f(n: UInt) {
      r := n + 1
      = r
    }
    a := f(1)
    b := f(5)
    "#,
    "a", value "2"
  );
  checkStructure!(
    r#"
    f(n: UInt) {
      r := n + 1
      = r
    }
    a := f(1)
    b := f(5)
    "#,
    "b", value "6"
  );
}

/// Метод, изменяющий свой параметр, не влияет на следующий вызов.
#[test]
fn mutationNotLeaked() -> ()
{
  checkStructure!(
    r#"
    inc(n: UInt) { = n + 1 }
    a := inc(10)
    b := inc(20)
    "#,
    "a", value "11"
  );
  checkStructure!(
    r#"
    inc(n: UInt) { = n + 1 }
    a := inc(10)
    b := inc(20)
    "#,
    "b", value "21"
  );
}

/// Локальная переменная внутри блока условия не видна снаружи после вызова.
#[test]
fn localInBlockNotVisible() -> ()
{
  let main: Arc<RwLock<Structure>> = runCode
  (
    "f() { ? True { inner := 5 } }\nf()"
  );
  let mainGuard = main.read().unwrap();
  let found = mainGuard.getStructureByLink("inner");
  assert!(found.is_none(), "inner must not leak to outer scope");
}

// =================================================================================================
// Параметры изолированы.

/// Параметры метода не сохраняют значение между вызовами.
#[test]
fn paramsResetBetweenCalls() -> ()
{
  let main: Arc<RwLock<Structure>> = runCode("f(n: UInt) { }\nf(100)\nf(200)");
  let mainGuard: RwLockReadGuard<Structure> = main.read().unwrap();
  let f: Arc<RwLock<Structure>> = mainGuard.getStructureByScope("f").expect("f");
  let fGuard: RwLockReadGuard<Structure> = f.read().unwrap();
  let structures = fGuard.structures.read().unwrap();
  let params: Vec<_> = structures.as_ref().map(|v| v.clone()).unwrap_or_default();
  
  // Первый child — параметр `n`. После f(200) там должно быть 200.
  if let Some(p) = params.first()
  {
    let pGuard = p.read().unwrap();
    if let Some(lines) = &pGuard.lines
    {
      if let Some(firstLine) = lines.first()
      {
        let lineGuard = firstLine.read().unwrap();
        if let Some(tokens) = &lineGuard.tokens
        {
          if let Some(firstToken) = tokens.first()
          {
            let val = firstToken.getData().toString();
            assert_eq!(val.as_deref(), Some("200"));
          }
          //
        }
      }
      //
    }
  }
  //
}

// =================================================================================================
// Прямой тест deepClone.

/// Глубокое клонирование: изменение клона не влияет на оригинал.
#[test]
fn deepCloneIsolates() -> ()
{
  let original: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
    Some(String::from("f")),
    crate::parser::structure::structure::StructureMut::Variable,
    crate::parser::structure::structureType::StructureType::Method,
    None,
    None
  )));
  // Добавим child в оригинал.
  let child = Arc::new(RwLock::new(Structure::new(
    Some(String::from("inner")),
    crate::parser::structure::structure::StructureMut::Constant,
    crate::parser::structure::structureType::StructureType::U8,
    None,
    None
  )));
  original.write().unwrap().pushStructure(child);

  // Клонируем.
  let cloned: Structure = original.read().unwrap().deepClone();

  // Меняем clone: добавим ещё один child.
  cloned.structures.write().unwrap()
    .as_mut().unwrap().push(Arc::new(RwLock::new(Structure::new(
      Some(String::from("new_child")),
      crate::parser::structure::structure::StructureMut::Constant,
      crate::parser::structure::structureType::StructureType::U8,
      None,
      None
    ))));

  // Оригинал не изменился — там всё ещё один child.
  let originalChildren = original.read().unwrap()
    .structures.read().unwrap()
    .as_ref().map(|v| v.len()).unwrap_or(0);
  assert_eq!(originalChildren, 1, "original must have 1 child");

  // У клона — два child.
  let clonedChildren = cloned.structures.read().unwrap()
    .as_ref().map(|v| v.len()).unwrap_or(0);
  assert_eq!(clonedChildren, 2, "clone must have 2 children");
}

// =================================================================================================
