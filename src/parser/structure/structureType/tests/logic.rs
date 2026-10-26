use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::parser::testing::{checkExpression, checkStructure};
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Тесты на `True`/`False`/`None` как типы.
//
// Здесь есть примеры с `=` они работают т.к. указаны типы структур;
// В иных случаях `:=` как при создании.

// =================================================================================================

/// `True` как тип.
#[test]
fn trueType() -> ()
{
  checkStructure!(
    r#"
    a: True = True
    "#,
    "a",
    type TokenType::True,
    stype StructureType::Primitive(PrimitiveType::True),
    value "True"
  );
}

/// `False` как тип.
#[test]
fn falseType() -> ()
{
  checkStructure!(
    r#"
    b: False = False
    "#,
    "b",
    type TokenType::False,
    stype StructureType::Primitive(PrimitiveType::False),
    value "False"
  );
}

/// `None` как тип.
#[test]
#[ignore] // todo Это известный баг? почему оно тут так?
fn noneType() -> ()
{
  checkStructure!(
    r#"
    c: None = None
    "#,
    "c",
    type TokenType::None,
    stype StructureType::Primitive(PrimitiveType::None),
    value ""
  );
}

// =================================================================================================

/// Union `True | False`: значение выбирает `вариант`.
#[test]
fn trueFalseUnion() -> ()
{
  checkStructure!(
    r#"
    d~: True | False = True
    "#,
    "d",
    type TokenType::True,
    stype StructureType::Primitive(PrimitiveType::True),
    value "True"
  );
  checkStructure!(
    r#"
    d~: True | False = True
    d = False
    "#,
    "d",
    type TokenType::False,
    stype StructureType::Primitive(PrimitiveType::False),
    value "False"
  );
}

/// Без аннотации — вывод из значения.
#[test]
fn inferred() -> ()
{
  checkStructure!(
    r#"
    e := True
    "#,
    "e",
    type TokenType::True,
    stype StructureType::Primitive(PrimitiveType::True),
    value "True"
  );
  checkStructure!(
    r#"
    f := False
    "#,
    "f",
    type TokenType::False,
    stype StructureType::Primitive(PrimitiveType::False),
    value "False"
  );
}

/// Forward-declaration без значения — `None`.
#[test]
#[ignore] // todo Это известный баг? почему оно тут так?
fn forwardDecl() -> ()
{
  checkStructure!(
    r#"
    g
    "#,
    "g",
    type TokenType::None,
    stype StructureType::Primitive(PrimitiveType::None),
    value ""
  );
}

// =================================================================================================

// Известные баги.

/// Отрицательное число справа от сравнения: минус склеивается с оператором.
#[test]
#[ignore]
fn negativeRightSide() -> ()
{
  checkExpression!("5 > -5", type TokenType::True);
  checkExpression!("-3 = -3", type TokenType::True);
  checkExpression!("0 > -3", type TokenType::True);
}

/// Числа разных типов: сравниваются по виду, а не по значению.
#[test]
#[ignore]
fn crossTypeComparison() -> ()
{
  checkExpression!("5 = 5.0", type TokenType::True);
  checkExpression!("10 > 2.5", type TokenType::True);
  checkExpression!("5 < 3.0", type TokenType::False);
  checkExpression!("2.5 < 10", type TokenType::True);
}

/// `True`/`False` не числа: перед сравнением с числом их нужно приводить явно.
#[test]
#[ignore]
fn boolNumberComparison() -> ()
{
  checkExpression!("True = 1", type TokenType::False);
  checkExpression!("False = 0", type TokenType::False);
}

// =================================================================================================
