use crate::parser::testing::checkExpression;
use crate::tokenizer::types::tokenType::TokenType;
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
    type TokenType::Int,
    value "-10"
  );
  checkExpression!(
    "(10 - 20)",
    type TokenType::Int,
    value "-10"
  );
  checkExpression!(
    "(1+2)-(3+4)",
    type TokenType::Int,
    value "-4"
  );
  checkExpression!(
    "10 - (20 - 5)",
    type TokenType::Int,
    value "-5"
  );
  checkExpression!(
    "10 + (-20)",
    type TokenType::Int,
    value "-10"
  );
  checkExpression!(
    "10+(-20)",
    type TokenType::Int,
    value "-10"
  );
  checkExpression!(
    "-(5)",
    type TokenType::Int,
    value "-5"
  );
  checkExpression!(
    "-(2 + 3) - 4",
    type TokenType::Int,
    value "-9"
  );
}

/// Вычитание констант.
#[test]
fn subtraction() -> ()
{
  checkExpression!(
    "10-20",
    type TokenType::Int,
    value "-10"
  );
  checkExpression!(
    "10 - 20",
    type TokenType::Int,
    value "-10"
  );
  checkExpression!(
    "10 + -20",
    type TokenType::Int,
    value "-10"
  );
  checkExpression!(
    "10 - 20 - 5",
    type TokenType::Int,
    value "-15"
  );
}

// =================================================================================================
