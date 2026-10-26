use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::PrimitiveType;
use crate::parser::testing::checkStructure;
use crate::parser::testing::runCode;
use std::sync::{Arc, RwLock};
// =================================================================================================

// Тесты на методы приведения к `StructureType`.
//
// Метод берёт токен и возвращает новый токен в границах `StructureType`;
// Границы те же, что при объявлении `a: U8 = ...`, их считает `Structure::normalizeToken`.
//
// Структура `a := U8(300)` получает тип обычным путём: из готового токена, а не от метода.

// =================================================================================================
// Беззнаковые.

/// `U8`: всё что больше — граница типа, отрицательное — `0`.
#[test]
fn u8() -> ()
{
  checkStructure!("a := U8(300)", "a", value "255");
  checkStructure!("a := U8(10)", "a", value "10");
  checkStructure!("a := U8(-10)", "a", value "0");
}

#[test]
fn u16() -> ()
{
  checkStructure!("a := U16(70000)", "a", value "65535");
}

#[test]
fn u32() -> ()
{
  checkStructure!("a := U32(5000000000)", "a", value "4294967295");
}

#[test]
fn u64() -> ()
{
  checkStructure!("a := U64(99999999999999999999)", "a", value "18446744073709551615");
}

#[test]
fn usize() -> ()
{
  checkStructure!("a := Usize(99999999999999999999)", "a", value "18446744073709551615");
}

// =================================================================================================
// Знаковые.

/// `I8`: границы с обеих сторон.
#[test]
fn i8() -> ()
{
  checkStructure!("a := I8(300)", "a", value "127");
  checkStructure!("a := I8(-300)", "a", value "-128");
  checkStructure!("a := I8(-5)", "a", value "-5");
}

#[test]
fn i16() -> ()
{
  checkStructure!("a := I16(40000)", "a", value "32767");
}

#[test]
fn i32() -> ()
{
  checkStructure!("a := I32(3000000000)", "a", value "2147483647");
}

#[test]
fn i64() -> ()
{
  checkStructure!("a := I64(99999999999999999999)", "a", value "9223372036854775807");
}

#[test]
fn isize() -> ()
{
  checkStructure!("a := Isize(-99999999999999999999)", "a", value "-9223372036854775808");
}

// =================================================================================================
// Плавающие.

#[test]
fn f32() -> ()
{
  checkStructure!("a := F32(1.5)", "a", value "1.5");
}

#[test]
fn f64() -> ()
{
  checkStructure!("a := F64(3)", "a", value "3");
}

/// Дробное в целый тип округляется, как при объявлении.
#[test]
fn floatToInteger() -> ()
{
  checkStructure!("a := U8(1.9)", "a", value "2");
  checkStructure!("x: U8 = 1.9", "x", value "2");
}

// =================================================================================================
// Не число.

/// То, что числом не является, даёт базовое значение типа, как `a: U8 = "test"`.
#[test]
fn notNumber() -> ()
{
  checkStructure!(r#"a := U8("test")"#, "a", value "0");
  checkStructure!(r#"x: U8 = "test""#, "x", value "0");
}

// =================================================================================================
// Те же границы, что при объявлении.

/// Метод и объявление режут значение одинаково.
#[test]
fn sameAsDeclaration() -> ()
{
  checkStructure!("x: U8 = 300", "x", value "255");
  checkStructure!("a := U8(300)", "a", value "255");

  checkStructure!("x: I8 = -300", "x", value "-128");
  checkStructure!("a := I8(-300)", "a", value "-128");

  checkStructure!("x: U8 = -10", "x", value "0");
  checkStructure!("a := U8(-10)", "a", value "0");
}

// =================================================================================================
// Где метод применяется.

/// Результат метода — токен: тип данных токена остаётся числовым.
#[test]
fn resultIsToken() -> ()
{
  checkStructure!(
    "a := U8(300)",
    "a",
    type UInt,
    stype U8,
    value "255"
  );
  checkStructure!(
    "a := I8(-300)",
    "a",
    type Int,
    stype I8,
    value "-128"
  );
}

/// `Usize(a)` — такой же метод, как остальные: ссылку на временную структуру он не возвращает.
#[test]
fn usizeIsToken() -> ()
{
  checkStructure!(
    "a := Usize(5)",
    "a",
    type UInt,
    stype U8,
    value "5"
  );
}

/// Аргумент — выражение: именованная структура подставляется своим значением.
#[test]
fn structureAsArgument() -> ()
{
  checkStructure!(
    r#"
    a: U64 = 300
    b := U8(a)
    "#,
    "b", value "255"
  );
}

/// Методы вкладываются: каждый режет значение под свой тип.
#[test]
fn nested() -> ()
{
  checkStructure!("a := U8(U16(70000))", "a", value "255");
  checkStructure!("a := U16(U8(300))", "a", value "255");
}

/// В выражении метод режет только свой операнд, и сумма считается уже с ним.
///
/// - `U8(a)` — `255`;
/// - `255 + 10` — `265`;
/// - а `c` получает тип по токену: `U16`.
#[test]
fn inExpression() -> ()
{
  checkStructure!(
    r#"
    a: U64 = 1111111
    b: U8 = 10
    c := U8(a) + b
    "#,
    "c",
    type UInt,
    stype U16,
    value "265"
  );
}

/// Без метода операнд не режется: тот же пример даёт сумму целиком.
#[test]
fn noCastInExpression() -> ()
{
  checkStructure!(
    r#"
    a: U64 = 1111111
    b: U8 = 10
    c := a + b
    "#,
    "c",
    type UInt,
    stype U32,
    value "1111121"
  );
}

/// Метод стоит в аргументе вызова: параметр получает уже срезанное значение.
#[test]
fn methodArgument() -> ()
{
  checkStructure!(
    r#"
    f(x: U16) {
      = x
    }
    r := f(U8(300))
    "#,
    "r", value "255"
  );
}

// =================================================================================================
// Все примитивы.

/// Примитив без приведения к границам: значение проходит как есть.
///
/// Приведения для таких типов пока нет, поэтому каст ничего не меняет.
#[test]
fn pointer() -> ()
{
  checkStructure!("a := Pointer(5)", "a", value "5");
  checkStructure!("a := CString(7)", "a", value "7");
  checkStructure!("a := Any(5)", "a", value "5");
}

/// `String` и `RawString`: значение становится строкой этого типа.
#[test]
fn toString() -> ()
{
  checkStructure!(
    "a := String(5)", "a",
    type String,
    stype String,
    value "5"
  );
  checkStructure!(
    "a := RawString(5)", "a",
    type RawString,
    stype RawString,
    value "5"
  );
}

/// Здесь `None` - это отсутствие значения, а не значение.
#[test]
fn toNone() -> ()
{
  checkStructure!("a := None(5)", "a", type None, value "");
}

/// todo desc
#[test]
fn everyPrimitiveIsCast() -> ()
{
  for (name, _) in PrimitiveType::iter()
  {
    let code: String = format!("a := {name}(5)");
    let main: Arc<RwLock<Structure>> = runCode!(&code);

    if name == "None"
    { // `None` - это отсутствие значения.
      checkStructure!(main, "a", type None, value "");
    } else {
      checkStructure!(main, "a", value "5");
    }
  }
}

// =================================================================================================
