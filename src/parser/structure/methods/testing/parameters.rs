use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::PrimitiveType;
use crate::parser::structure::structureType::union::union;
use crate::parser::testing::checkStructure;
use crate::parser::testing::runCode;
use std::sync::{Arc, RwLock};
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
// Единое приведение с методом `P(...)`.

/// Параметр приводится тем же приведением, что и метод приведения `P(...)`.
#[test]
fn likeCastMethod() -> ()
{
  // Число в строковом параметре становится строкой: как `String(5)`.
  checkStructure!(
    r#"
    f(x: String) { = x }
    a := f(5)
    "#,
    "a",
    type String,
    stype String,
    value "5"
  );
  checkStructure!(
    r#"
    f(x: RawString) { = x }
    a := f(5)
    "#,
    "a",
    type RawString,
    stype RawString,
    value "5"
  );

  // Отрицательное в беззнаковом параметре - граница типа, как `U8(-10)`.
  checkStructure!(
    r#"
    f(x: U8) { = x }
    a := f(-10)
    "#,
    "a",
    type UInt,
    stype U8,
    value "0"
  );

  // `None`-параметр - отсутствие значения, как `None(5)`.
  checkStructure!(
    r#"
    f(x: None) { = x }
    a := f(5)
    "#,
    "a",
    type None,
    stype None,
    value ""
  );
}

/// Для каждого примитива параметр с его типом даёт то же значение,
/// что и метод приведения: `f(p: P) { = p } a := f(5)` - это `a := P(5)`.
#[test]
fn everyPrimitiveParamIsCast() -> ()
{
  for (name, _) in PrimitiveType::iter()
  {
    let code: String = format!("f(p: {name}) {{ = p }} a := f(5)");
    let main: Arc<RwLock<Structure>> = runCode!(&code);

    if name == "None"
    { // `None` - это отсутствие значения.
      checkStructure!(main, "a", type None, value "");
    } else
    if name == "F32" || name == "F64"
    { // `float` примитивы: значение `5` форматируется через `{:?}`,
      // который сохраняет точку для целых `float` ⟶ `5.0`.
      checkStructure!(main, "a", value "5.0");
    } else {
      checkStructure!(main, "a", value "5");
    }
  }
}

// =================================================================================================
