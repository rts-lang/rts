use crate::parser::structure::structureType::union::union;
use crate::parser::testing::checkStructure;
// =================================================================================================

// Тесты на параметры методов.
//
// todo Сюда надо будет тесты с параметрами, без параметров и кучу других.
//  Сейчас здесь по большей части `union`.
// todo Еще нужны тесты на несколько параметров. А для отделения надо
//  использовать `;` а не `,` т.к. смысл в разделении линий внутри `()`.

// =================================================================================================

/// `n: U8 | String`: вариант подбирается по переданному значению.
#[test]
fn picksVariant() -> ()
{
  checkStructure!(
    r#"
    f(n: U8 | String) { = n }
    a := f(300)
    "#,
    "a",
    type UInt,
    stype U8,
    value "255"
  );
  checkStructure!(
    r#"
    f(n: U8 | String) { = n }
    a := f("x")
    "#,
    "a",
    type String,
    stype String,
    value "x"
  );
}

/// Значение не подходит ни к одному варианту - оно становится `None`.
#[test]
fn noVariant() -> ()
{
  checkStructure!(
    r#"
    f(n: U8 | String) { = n }
    a := f(True)
    "#,
    "a",
    type None,
    stype None,
    value ""
  );
}

/// Вариант выбранный прошлым вызовом не влияет на следующий.
#[test]
fn everyCall() -> ()
{
  checkStructure!(
    r#"
    f(n: U8 | String) { = n }
    a := f(300)
    b := f("x")
    "#,
    "a",
    stype U8,
    value "255"
  );
  checkStructure!(
    r#"
    f(n: U8 | String) { = n }
    a := f(300)
    b := f("x")
    "#,
    "b",
    stype String,
    value "x"
  );
  checkStructure!(
    r#"
    f(n: U8 | String) { = n }
    b := f("x")
    a := f(300)
    "#,
    "a",
    stype U8,
    value "255"
  );
}

/// Параметр даёт то же, что объявление с таким же объединением.
#[test]
fn likeDeclaration() -> ()
{
  checkStructure!(
    r#"
    f(n: I8 | U8) { = n }
    a := f(300)
    "#,
    "a",
    value "127"
  );
  checkStructure!(
    "a: I8 | U8 = 300", "a",
    stype I8,
    utype union![I8, U8],
    value "127"
  );
}

/// `None` в объединении параметра отбрасывается: `n: U8 | None` это `n: U8`.
#[test]
fn noneDropped() -> ()
{
  checkStructure!(
    r#"
    f(n: U8 | None) { = n }
    a := f(300)
    "#,
    "a",
    stype U8,
    value "255"
  );
  checkStructure!(
    r#"
    f(n: U8) { = n }
    a := f(300)
    "#,
    "a",
    stype U8,
    value "255"
  );
}

/// Объединение не мешает объявлению остальных параметров через запятую.
#[test]
fn manyParams() -> ()
{
  checkStructure!(
    r#"
    f(x: U8 | String, y: I8) { = x }
    a := f("t"; -300)
    "#,
    "a",
    stype String,
    value "t"
  );
  checkStructure!(
    r#"
    f(x: I8, y: U8 | String) { = x }
    a := f(-300; "t")
    "#,
    "a",
    type Int,
    stype I8,
    value "-128"
  );
}

/// Параметр без типа пропускает любое значение как раньше.
#[test]
fn noType() -> ()
{
  checkStructure!(
    r#"
    f(n) { = n }
    a := f(300)
    "#,
    "a",
    stype U16,
    value "300"
  );
}

// =================================================================================================
