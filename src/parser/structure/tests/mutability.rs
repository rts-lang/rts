use crate::parser::testing::checkStructure;
// =================================================================================================

// Тесты на мутабельность структур.
//
//
// Есть следующие виды `mut`:
//
// - `Final`: Ожидает первое значение, потом становится `Constant`;
//
// - `Constant`: Значение и тип зафиксированы;
//
// - `Variable` `~`: Меняется только значение;
//
// - `Dynamic` `~~`: Меняются значение и тип.
//
//
// Есть также `mut` - это примитив-метод для проверки;
// В тестах он встроенный `mut`.

// =================================================================================================
// Final

/// `a` без значения; первое `=` задаёт значение, mut → `Constant`.
#[test]
fn firstValue() -> ()
{
  checkStructure!(
    r#"
    a
    a = 10
    "#,
    "a",
    value "10",
    mut Constant
  );
}

/// После Final→Constant второе `=` значение не меняет.
#[test]
fn secondBlocked() -> ()
{
  checkStructure!(
    r#"
    a
    a = 10
    a = 20
    "#,
    "a",
    value "10",
    mut Constant
  );
}

/// `a: U8` — Final с типом; значение приводится, mut → `Constant`.
#[test]
fn typedFirst() -> ()
{
  checkStructure!(
    r#"
    a: U8
    a = 300
    "#,
    "a",
    type UInt,
    stype U8,
    value "255",
    mut Constant
  );
}

// =================================================================================================
// Constant

/// `a := 10` — сразу `Constant`, тип из значения.
#[test]
fn constant() -> ()
{
  checkStructure!(
    "a := 10",
    "a",
    value "10",
    mut Constant
  );
}

/// Повторное `=` у `Constant` значение не меняет.
#[test]
fn constantBlocked() -> ()
{
  checkStructure!(
    r#"
    a := 10
    a = 20
    "#,
    "a",
    value "10",
    mut Constant
  );
}

/// `a: U8 := 300` — приведение к типу, `Constant`.
#[test]
fn constantTyped() -> ()
{
  checkStructure!(
    "a: U8 = 300",
    "a",
    type UInt,
    stype U8,
    value "255",
    mut Constant
  );
}

/// `a: U8 := -10` — знаковый литерал к `U8`, mut `Constant`.
#[test]
fn constantSigned() -> ()
{
  checkStructure!(
    "a: U8 = -10",
    "a",
    stype U8,
    mut Constant
  );
}

// =================================================================================================
// Variable (`~`)

/// `a~` без значения; первое `=` задаёт значение, mut `Variable`.
#[test]
fn variable() -> ()
{
  checkStructure!(
    r#"
    a~
    a = 10
    "#,
    "a",
    value "10",
    mut Variable
  );
}

/// `a~: U8` — тип держится при смене значения.
#[test]
fn variableTyped() -> ()
{
  checkStructure!(
    r#"
    a~: U8
    a = 10
    a = 300
    "#,
    "a",
    type UInt,
    stype U8,
    value "255",
    mut Variable
  );
}

/// `a~ := 10` — значение можно перезаписать.
#[test]
fn variableValue() -> ()
{
  checkStructure!(
    r#"
    a~ := 10
    a = 20
    "#,
    "a",
    value "20",
    mut Variable
  );
}

/// `a~: U8 := 10` — тип и значение при создании.
#[test]
fn variableTypedValue() -> ()
{
  checkStructure!(
    "a~: U8 = 10",
    "a",
    type UInt,
    stype U8,
    value "10",
    mut Variable
  );
}

/// `a~: U8` — перезапись всё равно через приведение к типу.
#[test]
fn variableCast() -> ()
{
  checkStructure!(
    r#"
    a~: U8 = 1
    a = 999
    "#,
    "a",
    type UInt,
    stype U8,
    value "255",
    mut Variable
  );
}

// =================================================================================================
// Dynamic (`~~`)

/// `a~~` без значения; первое `=` задаёт значение, mut `Dynamic`.
#[test]
fn dynamic() -> ()
{
  checkStructure!(
    r#"
    a~~
    a = 10
    "#,
    "a",
    value "10",
    mut Dynamic
  );
}

/// `a~~: U8` — тип можно сменить (число → строка).
#[test]
fn dynamicTyped() -> ()
{
  checkStructure!(
    r#"
    a~~: U8
    a = 10
    a = "text"
    "#,
    "a",
    value "text",
    mut Dynamic
  );
}

/// `a~~ := 10` — смена значения и типа.
#[test]
fn dynamicValue() -> ()
{
  checkStructure!(
    r#"
    a~~ := 10
    a = "text"
    "#,
    "a",
    value "text",
    mut Dynamic
  );
}

/// `a~~: U8 := 10` — тип и значение при создании.
#[test]
fn dynamicTypedValue() -> ()
{
  checkStructure!(
    "a~~: U8 = 10",
    "a",
    type UInt,
    stype U8,
    value "10",
    mut Dynamic
  );
}

/// После typed Dynamic `=` строки меняет dataType.
#[test]
fn dynamicTypeChange() -> ()
{
  checkStructure!(
    r#"
    a~~: U8 = 10
    a = "x"
    "#,
    "a",
    value "x",
    mut Dynamic
  );
}

/// `a~~` — смена только числового значения.
#[test]
fn dynamicReassign() -> ()
{
  checkStructure!(
    r#"
    a~~ := 1
    a = 2
    "#,
    "a",
    value "2",
    mut Dynamic
  );
}

// =================================================================================================
