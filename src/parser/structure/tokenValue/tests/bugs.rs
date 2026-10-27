use crate::parser::testing::checkExpression;
// =================================================================================================

// todo Известные баги.

// =================================================================================================

/// Скобки: результат выражения в скобках.
#[test]
#[ignore]
fn brackets() -> ()
{
  checkExpression!(
    "(10-20)",
    type Int,
    value "-10"
  );
  checkExpression!(
    "(10 - 20)",
    type Int,
    value "-10"
  );
  checkExpression!(
    "(1+2)-(3+4)",
    type Int,
    value "-4"
  );
  checkExpression!(
    "10 - (20 - 5)",
    type Int,
    value "-5"
  );
  checkExpression!(
    "10 + (-20)",
    type Int,
    value "-10"
  );
  checkExpression!(
    "10+(-20)",
    type Int,
    value "-10"
  );
  checkExpression!(
    "-(5)",
    type Int,
    value "-5"
  );
  checkExpression!(
    "-(2 + 3) - 4",
    type Int,
    value "-9"
  );
}

/// Вычитание констант.
#[test]
fn subtraction() -> ()
{
  checkExpression!(
    "10-20",
    type Int,
    value "-10"
  );
  checkExpression!(
    "10 - 20",
    type Int,
    value "-10"
  );
  checkExpression!(
    "10 + -20",
    type Int,
    value "-10"
  );
  checkExpression!(
    "10 - 20 - 5",
    type Int,
    value "-15"
  );
}

// =================================================================================================
