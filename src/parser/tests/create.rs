use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::parser::testing::checkStructure;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock};
// =================================================================================================

// Это тесты на оператор создания структуры `:=` и на разницу между ним и `=`.
//
// - `:=` => Создание в текущей области. Вверх не ищет никогда:
//          `limit := 5` внутри структуры создает новую структуру `limit`, внешнее не трогает.
//
// - `=` => Присваивание по области видимости: сначала свои структуры, потом до самого верха.
//          Именно поэтому `a = 20` внутри `? { }` меняет объявленное выше `a`,
//          а `counter = counter + 1` внутри структуры доходит до `counter` в main.
//
// - `a.b` => Ссылка (TokenType::Link): голова `a` ищется вверх по области видимости, как голое имя;
//            а путь после неё (`.b`) идёт строго вниз по найденной структуре.
//
//  todo Названия методов слишком большие.
//
//  todo Отсюда надо будет вырезать Link тесты которые не касаются create в другой файл тестов потом.

// =================================================================================================
// `:=` — создание локально.

/// `:=` в теле функции заводит своё имя и НЕ трогает внешнее.
#[test]
fn creationInFunctionStaysLocal() -> ()
{
  checkStructure!(
    r#"
    limit := 10
    check() {
      limit := 5
    }
    check()
    "#,
    "limit", value "10"
  );
}

/// `:=` внутри блока условия точно так же остаётся локальным.
/// Блок условия - это отдельная область, хотя и видит внешнее.
#[test]
fn creationInConditionBlockStaysLocal() -> ()
{
  checkStructure!(
    r#"
    b~ := 10
    ? True {
      b~ := 20
    }
    "#,
    "b", value "10"
  );
}

/// Тип можно указать между именем и `:=`: `a: U8 := 300`.
/// Значит `:=` не мешает разбору типа - он идёт после двоеточия,
/// и значение приводится к объявленному типу (300 не влезает в U8).
#[test]
fn typeBeforeCreationOperator() -> ()
{
  checkStructure!(
    r#"
    a: U8 := 300
    "#,
    "a",
    type TokenType::UInt,
    stype StructureType::Primitive(PrimitiveType::U8),
    value "255"
  );
}

/// Объединение типов - тоже между именем и `:=`: `t~: 10 | 20 := 10`.
#[test]
fn unionBeforeCreationOperator() -> ()
{
  checkStructure!(
    r#"
    t~: 10 | 20 := 10
    t = 20
    "#,
    "t", value "20"
  );
}

// =================================================================================================
// `=` — поиск вверх.

/// `=` в теле функции доходит до внешней структуры и меняет её.
/// Это обратный путь: слева структура, объявленная выше функции.
#[test]
fn assignFromFunctionReachesOuter() -> ()
{
  checkStructure!(
    r#"
    counter~ := 0
    tick() {
      counter = counter + 1
    }
    tick()
    tick()
    "#,
    "counter", value "2"
  );
}

/// `=` внутри блока условия меняет объявленное выше - блок работает
/// в области объемлющей структуры (issue #39).
#[test]
fn assignFromConditionBlockReachesOuter() -> ()
{
  checkStructure!(
    r#"
    a~ := 10
    ? True {
      a = 20
    }
    "#,
    "a", value "20"
  );
}

/// Чтение тоже идёт вверх: из тела функции видна структура, объявленная выше.
/// Это и есть тот случай, который в 231206 работал, а на текущем head потерялся.
#[test]
fn readFromFunctionReachesOuter() -> ()
{
  checkStructure!(
    r#"
    top~ := 77
    got~ := 0
    peek() {
      got = top~
    }
    peek()
    "#,
    "got", value "77"
  );
}

/// `argc` живёт в самой верхней структуре и должен быть виден из тела функции.
#[test]
fn argcIsVisibleFromFunction() -> ()
{
  checkStructure!(
    r#"
    got~ := 0
    grab() {
      got = argc
    }
    grab()
    "#,
    "got", value "0"
  );
}

// =================================================================================================
// Объявления без значения.

/// `a` - Final без значения: тип и значение ждут первого `=`.
/// Первое значение задаёт тип, после этого структура становится Const.
#[test]
fn finalWithoutValueTakesFirstValue() -> ()
{
  checkStructure!(
    r#"
    c
    c = 5
    "#,
    "c",
    type TokenType::UInt,
    stype StructureType::Primitive(PrimitiveType::U8), // Тип выведен из первого значения - 5 влезает в U8.
    value "5"
  );
}

/// `a~` - Variable без значения: значение и тип выводятся при первом присваивании.
/// Раньше такое объявление падало на `rightValue.unwrap()`.
#[test]
fn variableWithoutValueIsAllowed() -> ()
{
  checkStructure!(
    r#"
    v~
    v~ = 7
    "#,
    "v", value "7"
  );
}

// =================================================================================================
// `a.b` — путь ссылки ищется только вниз, а голова `a` - вверх (см. `linkHeadSearchesUpward`).
//
// Проверяем напрямую на структурах: разрешение значений ссылок (`a.b` в выражении)
// в проекте ещё не доведено, а вот правило поиска должно быть зафиксировано.

/// `getStructureByLink` не выходит за пределы своей структуры: имя, которое лежит
/// у родителя, через ссылку не видно.
#[test]
fn linkSearchesOnlyDownward() -> ()
{
  let root: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
    Some(String::from("root")),
    StructureMut::Constant,
    StructureType::Primitive(PrimitiveType::Method),
    None,
    None
  )));

  let leaf: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
    Some(String::from("leaf")),
    StructureMut::Constant,
    StructureType::Primitive(PrimitiveType::Method),
    None,
    Some(root.clone())
  )));

  root.write().unwrap()
    .pushStructure(leaf.clone());

  // Ссылка ищет вниз: `leaf` у себя не находит.
  assert!(
    leaf.read().unwrap().getStructureByLink("leaf").is_none(),
    "Link must not look above itself"
  );

  // Обычный поиск по области - находит, потому что поднимается к родителю.
  assert!(
    leaf.read().unwrap().getStructureByScope("leaf").is_some(),
    "Scope search must look up to the parent"
  );
}

/// Ссылка находит то, что лежит ниже: `a.b`, где `b` - ребёнок `a`.
#[test]
fn linkFindsNestedStructure() -> ()
{
  let root: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
    Some(String::from("root")),
    StructureMut::Constant,
    StructureType::Primitive(PrimitiveType::Method),
    None,
    None
  )));

  let a: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
    Some(String::from("a")),
    StructureMut::Constant,
    StructureType::Primitive(PrimitiveType::Method),
    None,
    Some(root.clone())
  )));

  let b: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
    Some(String::from("b")),
    StructureMut::Constant,
    StructureType::Primitive(PrimitiveType::Method),
    None,
    Some(a.clone())
  )));

  a.write().unwrap()
    .pushStructure(b.clone());
  root.write().unwrap()
    .pushStructure(a.clone());

  // `a.b` - корень `a`, дальше вниз до `b`.
  assert!(
    a.read().unwrap().getStructureByLink("b").is_some(),
    "Link must find nested structure below"
  );

  // А из `b` вверх `a` не видна - ссылка вниз, а не вверх.
  assert!(
    b.read().unwrap().getStructureByLink("a").is_none(),
    "Link must not climb back up"
  );
}

/// Голова ссылки ищется вверх: `f.n` внутри другого метода и внутри блока `? { }`
/// поднимается до `main`, находит `f` и от неё идёт вниз к `n`.
#[test]
fn linkHeadSearchesUpward() -> ()
{
  // Из вложенной структуры.
  checkStructure!(
    r#"
    f(n: UInt) {
      = n
    }
    f(1)
    g() {
      = f.n
    }
    r := g()
    "#,
    "r", value "1"
  );

  // Из блока `? { }` внутри метода.
  checkStructure!(
    r#"
    f(n: UInt) {
      = n
    }
    f(1)
    h() {
      s~ := 0
      ? True {
        s = f.n
      }
      = s
    }
    r := h()
    "#,
    "r", value "1"
  );
}

// =================================================================================================
// Регрессия: дедлок на RwLock.

/// Раньше `linearStructure` брал write-блокировку на объемлющую структуру ДО
/// вычисления правой части, и поиск имени вверх по области видимости упирался
/// в повторный read() на ту же структуру — `std::sync::RwLock` не перевзводим,
/// и процесс вставал навсегда (0% CPU, sleeping).
///
/// Минимальный случай: функция объявляет локальную структуру, а её результат
/// присваивается. Здесь одновременно проверяется и то, что дедлока нет,
/// и то, что локальная структура действительно создалась.
#[test]
fn functionCreatingLocalStructureDoesNotDeadlock() -> ()
{
  // Внутри функции три операции подряд: создать локальную `a`, прочитать её,
  // присвоить наружу. Каждая из них в старом коде попадала в окно, где
  // объемлющая структура уже была залочена на запись.
  checkStructure!(
    r#"
    outer~ := 0
    f() {
      a := 1
      outer = a
    }
    f()
    "#,
    "outer", value "1"
  );
}

/// То же, но локальная структура объявлена через `:=` и отдельно проверяется,
/// что чтение вверх после её создания тоже не встаёт на блокировке.
#[test]
fn functionCreationWithOuterReadDoesNotDeadlock() -> ()
{
  checkStructure!(
    r#"
    outer~ := 5
    got~ := 0
    f() {
      inner~ := 1
      got = outer~
    }
    f()
    "#,
    "got", value "5"
  );
}

// =================================================================================================
// `:= None` — явная очистка структуры (#62).
//
// Только литерал `None` справа от `:=` удаляет структуру из текущей области.
// Любое выражение (`:= method()`, `:= b`) — даже если оно в рантайме вернуло
// `None` — создаёт/перезаписывает структуру со значением `None` (#71).
// `= None` не удаляет: `=` записывает `None` как значение.
//
// Примитивы (`type`, `String`, ...) никогда не удаляются — защита через
// `isPrimitiveName` в `linearStructure` и safety-net внутри `removeStructure`.

/// `:= None` удаляет локальную структуру: последующее обращение паникует
/// с "Structure 'a' was not created" — то, что выдаёт `getStructure`.
#[test]
#[should_panic(expected = "Structure 'a' was not created")]
fn creationNoneLiteralRemovesStructure() -> ()
{
  checkStructure!(
    r#"
    a~ := 5
    a := None
    "#,
    "a", value "5"
  );
}

/// `:= method()`, где `method` вернул `None`, НЕ удаляет структуру —
/// она остаётся существовать со значением `None` (#71: `None` как значение).
///
/// Это разделение намерений: программист написал `None` явно — хочет убрать.
/// Программа вернула `None` — структура остаётся, просто со значением `None`.
#[test]
fn creationWithNoneReturningMethodKeepsStructure() -> ()
{
  // `maybe()` всегда возвращает `None`: правая часть — выражение (вызов),
  // значит, идём по обычному пути создания, и `a` остаётся со значением `None`.
  checkStructure!(
    r#"
    maybe() { = None }
    a~ := maybe()
    "#,
    "a", value ""
  );
}

/// `= None` записывает `None` как значение, структура остаётся жива.
/// Это симметрично с `:= None` (который удаляет): `=` управляет значением,
/// `:=` управляет существованием.
#[test]
fn assignNoneKeepsStructureWithNoneValue() -> ()
{
  checkStructure!(
    r#"
    a~ := "text"
    a = None
    "#,
    "a", value ""
  );
}

/// `:= None` на примитиве (`type`) не удаляет его — защита через
/// `isPrimitiveName` в `linearStructure` и `removeStructure`. После `:= None`
/// примитив остаётся на месте: `type` всё ещё `Method`.
#[test]
fn creationNoneOnPrimitiveIsIgnored() -> ()
{
  checkStructure!(
    r#"
    type := None
    "#,
    "type", stype StructureType::Primitive(PrimitiveType::Method), mut StructureMut::Constant, lines 0
  );
}

// =================================================================================================
