use crate::parser::testing::checkStructure;
// =================================================================================================

// Тесты на приравнивание условного блока (структуры) с правой стороны
// к структуре с левой стороны.

// =================================================================================================

/// `? True { = 20 }` — блок выполняется, `result` становится значением.
#[test]
fn blockExpression() -> ()
{
  checkStructure!(
    r#"
    a := ? True { = 20 }
    "#,
    "a", value "20"
  );
}

/// `? False { = 99 }` — блок не выполняется, `a` получает `None`.
#[test]
fn blockExpressionFalse() -> ()
{
  checkStructure!(
    r#"
    a := ? False { = 99 }
    "#,
    "a", value ""
  );
}

/// Условие — не только `True`, но и выражение: `n = 5`.
#[test]
fn blockExpressionCondition() -> ()
{
  checkStructure!(
    r#"
    n := 5
    a := ? n = 5 { = 10 }
    "#,
    "a", value "10"
  );
}

/// Пустой блок возвращает `None` — нет `=`, нет `result`.
#[test]
fn blockExpressionEmpty() -> ()
{
  checkStructure!(
    r#"
    a := ? True {}
    "#,
    "a", value ""
  );
}

// =================================================================================================

// `elif`/`else` у блока-выражения: единая логика `if-else` (executeConditionChain)
// работает одинаково для оператора (`? c { }`) и для выражения (`a := ? c { }`).

/// `else` у выражения: условие ложно — значением становится результат `else`.
#[test]
fn blockExpressionElse() -> ()
{
  checkStructure!(
    r#"
    n := 3
    a := ? n = 5 { = 1 }
    ? { = 2 }
    "#,
    "a", value "2"
  );
}

/// `else` у выражения: условие истинно — `else` не выполняется
/// ни как значение, ни как побочный эффект.
#[test]
fn blockExpressionElseSkipped() -> ()
{
  checkStructure!(
    r#"
    n := 5
    a := ? n = 5 { = 1 }
    ? { = 2 }
    "#,
    "a", value "1"
  );
  checkStructure!(
    r#"
    m~: U8 = 0
    a := ? True { = 1 }
    ? { m = 9 }
    "#,
    "m", value "0"
  );
}

/// `elif`/`else` у выражения: берётся первая подходящая ветка.
#[test]
fn blockExpressionElif() -> ()
{
  checkStructure!(
    r#"
    n := 1
    a := ? n = 1 { = 10 }
    ? n = 2 { = 20 }
    ? { = 30 }
    "#,
    "a", value "10"
  );
  checkStructure!(
    r#"
    n := 2
    a := ? n = 1 { = 10 }
    ? n = 2 { = 20 }
    ? { = 30 }
    "#,
    "a", value "20"
  );
  checkStructure!(
    r#"
    n := 3
    a := ? n = 1 { = 10 }
    ? n = 2 { = 20 }
    ? { = 30 }
    "#,
    "a", value "30"
  );
}

/// Выражение внутри `result` вложенного блока: `= ? True { = 1 }`.
#[test]
fn blockExpressionNested() -> ()
{
  checkStructure!(
    r#"
    a := ? True { = ? True { = 1 } }
    "#,
    "a", value "1"
  );
}

/// `elif`/`else` у вложенного выражения читаются в линиях того блока, где оно лежит.
#[test]
fn blockExpressionNestedElse() -> ()
{
  checkStructure!(
    r#"
    a := ? True {
      = ? False { = 1 }
      ? { = 2 }
    }
    "#,
    "a", value "2"
  );
}

/// `:=` внутри блока-выражения возвращает из метода, остальное не читается.
#[test]
fn blockExpressionEarlyReturn() -> ()
{
  checkStructure!(
    r#"
    f() {
      a := ? True { := 7 }
      = 1
    }
    r := f()
    "#,
    "r", value "7"
  );
}

/// `else` завершает цепочку: `?` после него — это новая цепочка, а не `elif`.
#[test]
fn elseEndsChain() -> ()
{
  checkStructure!(
    r#"
    n := 1
    m~: U8 = 0
    ? n = 1 { m = 1 }
    ? { m = 2 }
    ? n = 1 { m = 3 }
    "#,
    "m", value "3"
  );
}

/// `else` в начале цепочки тоже завершает её: `?` после него — новая цепочка.
#[test]
fn elseFirstEndsChain() -> ()
{
  checkStructure!(
    r#"
    n := 1
    m~: U8 = 0
    ? { m = 1 }
    ? n = 1 { m = 2 }
    "#,
    "m", value "2"
  );
}

/// Правая часть читается полностью: `?` + условие заменяются результатом блока,
/// остальные токены остаются в выражении (`1 + (блок)` = 3).
#[test]
fn blockExpressionPrefix() -> ()
{
  checkStructure!(
    r#"
    a := 1 + ? True { = 2 }
    "#,
    "a", value "3"
  );
}

/// То же с `elif`/`else`: результат ветки подставляется в выражение.
#[test]
fn blockExpressionPrefixElse() -> ()
{
  checkStructure!(
    r#"
    b := 10 - ? False { = 2 }
    ? { = 4 }
    "#,
    "b", value "6"
  );
}

// =================================================================================================
