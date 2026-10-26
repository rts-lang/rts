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
    "a: True = True", "a",
    type TokenType::True,
    stype True,
    value "True"
  );
}

/// `False` как тип.
#[test]
fn falseType() -> ()
{
  checkStructure!(
    "b: False = False", "b",
    type TokenType::False,
    stype False,
    value "False"
  );
}

/// `None` как тип.
#[test]
#[ignore] // todo Это известный баг? почему оно тут так?
fn noneType() -> ()
{
  checkStructure!(
    "c: None = None", "c",
    type TokenType::None,
    stype None,
    value ""
  );
}

// =================================================================================================

/// Union `True | False`: значение выбирает `вариант`.
#[test]
fn trueFalseUnion() -> ()
{
  checkStructure!(
    "d~: True | False = True", "d",
    type TokenType::True,
    stype True,
    value "True"
  );
  checkStructure!(
    r#"
    d~: True | False = True
    d = False
    "#,
    "d",
    type TokenType::False,
    stype False,
    value "False"
  );
}

/// Без аннотации — вывод из значения.
#[test]
fn inferred() -> ()
{
  checkStructure!(
    "e := True", "e",
    type TokenType::True,
    stype True,
    value "True"
  );
  checkStructure!(
    "f := False", "f",
    type TokenType::False,
    stype False,
    value "False"
  );
}

/// Forward-declaration без значения — `None`.
#[test]
#[ignore] // todo Это известный баг? почему оно тут так?
fn forwardDecl() -> ()
{
  checkStructure!(
    "g", "g",
    type TokenType::None,
    stype None,
    value ""
  );
}

// =================================================================================================
// todo Известные баги.

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
