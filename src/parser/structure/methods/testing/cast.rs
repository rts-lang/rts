use crate::parser::parser::runCode;
use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::parser::testing::checkStructure;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock};
// =================================================================================================

// Тесты на методы приведения к StructureType: `U8(a)`, `I16(a)`, `F32(a)`, `Usize(a)` (Function::cast).
//
// Метод берёт токен и возвращает токен: значение режется под границы типа, структура не создаётся.
// Границы те же, что при объявлении `a: U8 = ...`, их считает `Structure::normalizeToken`.
//
// Структура `a := U8(300)` получает тип обычным путём: по токену, а не по методу.

// =================================================================================================
// Беззнаковые.

/// `U8`: всё что больше — граница типа, отрицательное — 0.
#[test]
fn castU8() -> ()
{
  checkStructure!( "a := U8(300)", "a", value "255" );
  checkStructure!( "a := U8(10)", "a", value "10" );
  checkStructure!( "a := U8(-10)", "a", value "0" );
}

#[test]
fn castU16() -> ()
{
  checkStructure!( "a := U16(70000)", "a", value "65535" );
}

#[test]
fn castU32() -> ()
{
  checkStructure!( "a := U32(5000000000)", "a", value "4294967295" );
}

#[test]
fn castU64() -> ()
{
  checkStructure!( "a := U64(99999999999999999999)", "a", value "18446744073709551615" );
}

#[test]
fn castUsize() -> ()
{
  checkStructure!( "a := Usize(99999999999999999999)", "a", value "18446744073709551615" );
}

// =================================================================================================
// Знаковые.

/// `I8`: границы с обеих сторон.
#[test]
fn castI8() -> ()
{
  checkStructure!( "a := I8(300)", "a", value "127" );
  checkStructure!( "a := I8(-300)", "a", value "-128" );
  checkStructure!( "a := I8(-5)", "a", value "-5" );
}

#[test]
fn castI16() -> ()
{
  checkStructure!( "a := I16(40000)", "a", value "32767" );
}

#[test]
fn castI32() -> ()
{
  checkStructure!( "a := I32(3000000000)", "a", value "2147483647" );
}

#[test]
fn castI64() -> ()
{
  checkStructure!( "a := I64(99999999999999999999)", "a", value "9223372036854775807" );
}

#[test]
fn castIsize() -> ()
{
  checkStructure!( "a := Isize(-99999999999999999999)", "a", value "-9223372036854775808" );
}

// =================================================================================================
// Плавающие.

#[test]
fn castF32() -> ()
{
  checkStructure!( "a := F32(1.5)", "a", value "1.5" );
}

#[test]
fn castF64() -> ()
{
  checkStructure!( "a := F64(3)", "a", value "3" );
}

/// Дробное в целый тип округляется, как при объявлении.
#[test]
fn castFloatToInteger() -> ()
{
  checkStructure!( "a := U8(1.9)", "a", value "2" );
  checkStructure!( "x: U8 = 1.9", "x", value "2" );
}

// =================================================================================================
// Не число.

/// То, что числом не является, даёт базовое значение типа, как `a: U8 = "test"`.
#[test]
fn castNotNumber() -> ()
{
  checkStructure!( r#"a := U8("test")"#, "a", value "0" );
  checkStructure!( r#"x: U8 = "test""#, "x", value "0" );
}

// =================================================================================================
// Те же границы, что при объявлении.

/// Метод и объявление режут значение одинаково.
#[test]
fn sameAsDeclaration() -> ()
{
  checkStructure!( "x: U8 = 300", "x", value "255" );
  checkStructure!( "a := U8(300)", "a", value "255" );

  checkStructure!( "x: I8 = -300", "x", value "-128" );
  checkStructure!( "a := I8(-300)", "a", value "-128" );

  checkStructure!( "x: U8 = -10", "x", value "0" );
  checkStructure!( "a := U8(-10)", "a", value "0" );
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
    type TokenType::UInt,
    stype StructureType::Primitive(PrimitiveType::U8),
    value "255"
  );
  checkStructure!(
    "a := I8(-300)",
    "a",
    type TokenType::Int,
    stype StructureType::Primitive(PrimitiveType::I8),
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
    type TokenType::UInt,
    stype StructureType::Primitive(PrimitiveType::U8),
    value "5"
  );
}

/// Аргумент — выражение: именованная структура подставляется своим значением.
#[test]
fn castOfStructure() -> ()
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
fn castNested() -> ()
{
  checkStructure!( "a := U8(U16(70000))", "a", value "255" );
  checkStructure!( "a := U16(U8(300))", "a", value "255" );
}

/// В выражении метод режет только свой операнд, и сумма считается уже с ним.
///
/// `U8(a)` — 255, `255 + 10` — 265, а `c` получает тип по токену: U16.
#[test]
fn castInExpression() -> ()
{
  checkStructure!(
    r#"
    a: U64 = 1111111
    b: U8 = 10
    c := U8(a) + b
    "#,
    "c",
    type TokenType::UInt,
    stype StructureType::Primitive(PrimitiveType::U16),
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
    type TokenType::UInt,
    stype StructureType::Primitive(PrimitiveType::U32),
    value "1111121"
  );
}

/// Метод стоит в аргументе вызова: параметр получает уже срезанное значение.
#[test]
fn castInArgument() -> ()
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
fn castPointer() -> ()
{
  checkStructure!( "a := Pointer(5)", "a", value "5" );
  checkStructure!( "a := CString(7)", "a", value "7" );
  checkStructure!( "a := Any(5)", "a", value "5" );
}

/// `String` и `RawString`: значение становится строкой этого типа.
#[test]
fn castToString() -> ()
{
  checkStructure!(
    "a := String(5)", "a",
    type TokenType::String,
    stype StructureType::Primitive(PrimitiveType::String),
    value "5"
  );
  checkStructure!(
    "a := RawString(5)", "a",
    type TokenType::RawString,
    stype StructureType::Primitive(PrimitiveType::RawString),
    value "5"
  );
}

/// `None` - это отсутствие значения, а не значение.
#[test]
fn castToNone() -> ()
{
  checkStructure!( "a := None(5)", "a", type TokenType::None, value "" );
}

/// todo desc
#[test]
fn everyPrimitiveIsCast() -> ()
{
  for (name, _) in PrimitiveType::iter()
  {
    let code: String = format!("a := {name}(5)");
    let main: Arc<RwLock<Structure>> = runCode(&code);

    if name == "None"
    { // `None` - это отсутствие значения.
      checkStructure!(main, "a", type TokenType::None, value "");
    } else {
      checkStructure!(main, "a", value "5");
    }
  }
}

// =================================================================================================
