use crate::parser::parser::runCode;
use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::PrimitiveType;
use crate::parser::testing::checkStructure;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use crate::tokenizer::types::token::Token;
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
    f(n: U8) {
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
    f(n: U8) {
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
    inc(n: U8) { = n + 1 }
    a := inc(10)
    b := inc(20)
    "#,
    "a", value "11"
  );
  checkStructure!(
    r#"
    inc(n: U8) { = n + 1 }
    a := inc(10)
    b := inc(20)
    "#,
    "b", value "21"
  );
}

/// Рекурсия с `:=` как early return — `? n = 0 { := 0 }` прерывает функцию.
#[test]
fn recursionWithEarlyReturn() -> ()
{
  // sum(0) = 0, sum(1) = 1, sum(2) = 3, sum(3) = 6.
  checkStructure!(
    r#"
    sum(n: U8) {
      ? n = 0 { := 0 }
      = n + sum(n - 1)
    }
    a := sum(3)
    "#,
    "a", value "6"
  );
}

/// `:=` прерывает функцию — код после блока не выполняется.
#[test]
fn earlyReturnSkipsRest() -> ()
{
  checkStructure!(
    r#"
    f(n: U8) {
      ? n = 0 { := 99 }
      = n
    }
    a := f(0)
    "#,
    "a", value "99"
  );
  checkStructure!(
    r#"
    f(n: U8) {
      ? n = 0 { := 99 }
      = n
    }
    b := f(5)
    "#,
    "b", value "5"
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
  let main: Arc<RwLock<Structure>> = runCode("f(n: U8) { }\nf(100)\nf(200)");
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
    crate::parser::structure::structureType::StructureType::Primitive(PrimitiveType::Method),
    None,
    None
  )));
  // Добавим child в оригинал.
  let child = Arc::new(RwLock::new(Structure::new(
    Some(String::from("inner")),
    crate::parser::structure::structure::StructureMut::Constant,
    crate::parser::structure::structureType::StructureType::Primitive(PrimitiveType::U8),
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
      crate::parser::structure::structureType::StructureType::Primitive(PrimitiveType::U8),
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
// Вложенность.

/// `:=` внутри вложенного `?`-блока поднимается до метода.
#[test]
fn nestedBlockReturn() -> ()
{
  checkStructure!(
    r#"
    f(n: U8) {
      ? n > 0 {
        ? n = 1 { := 10 }
      }
      = 99
    }
    a := f(1)
    "#,
    "a", value "10"
  );
  checkStructure!(
    r#"
    f(n: U8) {
      ? n > 0 {
        ? n = 1 { := 10 }
      }
      = 99
    }
    b := f(5)
    "#,
    "b", value "99"
  );
}

/// `:=` внутри вложенной функции поднимается до этой функции, не до внешней.
#[test]
fn nestedFunctionReturn() -> ()
{
  checkStructure!(
    r#"
    outer() {
      inner() {
        := 42
      }
      := inner()
    }
    a := outer()
    "#,
    "a", value "42"
  );
}

/// `:=` в else-ветке прерывает функцию.
#[test]
fn returnInElseBranch() -> ()
{
  checkStructure!(
    r#"
    f(n: U8) {
      ? n > 0 { := 1 }
      ? { := 2 }
    }
    a := f(0)
    "#,
    "a", value "2"
  );
  checkStructure!(
    r#"
    f(n: U8) {
      ? n > 0 { := 1 }
      ? { := 2 }
    }
    b := f(5)
    "#,
    "b", value "1"
  );
}

// =================================================================================================

/// `:=` поднимается до первого `named` блока.
/// 
/// И можно спутать что вот метод или не метод - но это ошибочное мнение.
/// 
/// На самом деле все есть просто структура и любая может быть запущена через `()`.
///
/// `helper { := 7 }` — `:=` пишет `result` в структуру `helper`.
/// `f` не имеет `result` => `a` будет `None`.
///
/// todo: Но еще структура без указания явного типа результата должна возвращать
///  значение, если оно есть. Сейчас `helper` имеет `result`, но `f`
///  его не возвращает — нужен явный `= helper` или `:= helper`.
///  А также чтобы не надо было `->` для helper и оно вернуло `7`.
///  И т.е. этот токен уже автоматически будет как-то рассматриваться по #71.
///  Тут важно еще соблюдать как раз типизацию `StructureType`.
#[test]
fn returnToFirstNamed() -> ()
{
  checkStructure!(
    r#"
    f() {
      helper { := 7 }
    }
    a := f()
    "#,
    "a", value ""
  );
}

// =================================================================================================

/// `:=` лежит в `?` и пишет только в именованные структуры.
///
/// Поэтому он поднимется на top-level (в main) — а там noname и
/// `:=` не должен написать в его `result`.
#[test]
fn topLevelReturn() -> ()
{
  let main: Arc<RwLock<Structure>> = runCode("? True { := 42 }");
  let result: Option<Token> = main.read().unwrap().result.clone();
  assert!(result.is_none());
}

// =================================================================================================
