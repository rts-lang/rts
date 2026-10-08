use crate::parser::testing::checkExpression;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Потолки Value + calculate: TokenType бесконечен, но мы ограничены математикой Rust.
//
// Здесь не тестируются структуры - т.к. это часть TokenType а не StructureType.
//
// А также это полная работа парсера, а не просто calculate.
//
// 4 класса токенов для чисел, у каждого свои min и max значения:
//   UInt    0 ... usize::MAX;
//   Int     isize::MIN ... -1.
//   UFloat  0 ... f64::MAX;
//   Float   f64::MIN ... -0.
//
// По сути `(1e308 + 1e308) - (1e308 + 1e308)` это:
//   1. 1e309 - граница, нормализация до f64::MAX.
//   2. (1e308 + 1e308) - результат выше, нормализация до f64::MAX.
//   3. Вычитание одинаковых в данном примере = 0 по итогу.
//
// todo Скобка справа считается только после `-`: `2 + (2 + 2)` даёт 2, `8 / (2 + 2)` даёт 8.
//  Поэтому справа везде `-`. Также `5 - -5` даёт 5: два оператора подряд пока не читаются.
//
// todo надо проверить: -4 / -2 даёт токен Int '2', хотя это UInt. Результат верный, но не тип).

// =================================================================================================

/// UInt: 0 ... usize::MAX.
#[test]
fn limitsUInt() -> ()
{
  let max: usize = usize::MAX;
  // Примитив ровно в потолок остаётся как есть.
  checkExpression!(
    &max.to_string(),
    type TokenType::UInt,
    value &max.to_string()
  );
  // Примитив на 1 больше потолка обрубается сразу.
  checkExpression!(
    &(max as u128 + 1).to_string(),
    type TokenType::UInt,
    value &max.to_string()
  );
  checkExpression!(
    "99999999999999999999999",
    type TokenType::UInt,
    value &max.to_string()
  );
  // 1. usize::MAX + 1 обрубается до usize::MAX.
  // 2. Обе скобки равны потолку.
  // 3. Вычитание = 0.
  checkExpression!(
    &format!("({max} + 1) - ({max} + 1)"),
    type TokenType::UInt,
    value "0"
  );
  // Примитив обрубается ещё до сложения.
  checkExpression!(
    "(99999999999999999999999 + 1) - (99999999999999999999999 + 1)",
    type TokenType::UInt,
    value "0"
  );
  // Результат не округляется до итога: потолок остаётся потолком.
  checkExpression!(
    &format!("({max} + 1) + 0"),
    type TokenType::UInt,
    value &max.to_string()
  );
  checkExpression!(
    "99999999999999999999999 / 1",
    type TokenType::UInt,
    value &max.to_string()
  );
}

/// Int: isize::MIN ... -1.
#[test]
fn limitsInt() -> ()
{
  let min: isize = isize::MIN;
  // Примитив ровно в потолок остаётся как есть.
  checkExpression!(
    &min.to_string(),
    type TokenType::Int,
    value &min.to_string()
  );
  // Примитив на 1 меньше потолка обрубается сразу.
  checkExpression!(
    &(min as i128 - 1).to_string(),
    type TokenType::Int,
    value &min.to_string()
  );
  checkExpression!(
    "-99999999999999999999999",
    type TokenType::Int,
    value &min.to_string()
  );
  // isize::MIN - 1 обрубается до isize::MIN, затем + 1.
  checkExpression!(
    &format!("({min} - 1) + 1"),
    type TokenType::Int,
    value &(min + 1).to_string()
  );
}

/// UFloat: 0 ... f64::MAX.
#[test]
fn limitsUFloat() -> ()
{
  // todo У f64::MAX две записи: примитив внутри потолка и результат операций - как `to_string()`,
  //  а обрубленный примитив - как `{:e}` (так inf пишет и StructureType). Значение то же.
  //
  // Примитив ровно в потолок остаётся как есть.
  checkExpression!(
    "1.7976931348623157e308",
    type TokenType::UFloat,
    value &f64::MAX.to_string()
  );
  // Первый примитив за потолком (парсится в inf) обрубается до f64::MAX.
  checkExpression!(
    "1.7976931348623159e308",
    type TokenType::UFloat,
    value &format!("{:e}", f64::MAX)
  );
  // 1. Примитив 1e309 за потолком обрубается до f64::MAX.
  checkExpression!(
    "1e309",
    type TokenType::UFloat,
    value &format!("{:e}", f64::MAX)
  );
  // 2. Вычитание = 0.
  checkExpression!(
    "1e309 - 1e309",
    type TokenType::UFloat,
    value "0"
  );
  // 1. 1e308 + 1e308 обрубается до f64::MAX (а не inf).
  // 2. Обе скобки равны потолку.
  // 3. Вычитание = 0 (а не inf - inf = NaN).
  checkExpression!(
    "(1e308 + 1e308) - (1e308 + 1e308)",
    type TokenType::UFloat,
    value "0"
  );
  // Обрубается на каждом шаге: f64::MAX - 1e308, а не inf - 1e308.
  checkExpression!(
    "(1e308 + 1e308) - 1e308",
    type TokenType::UFloat,
    value &(f64::MAX - 1e308).to_string()
  );
  checkExpression!(
    "(1e308 + 1e308) + 1",
    type TokenType::UFloat,
    value &f64::MAX.to_string()
  );
}

/// Float: f64::MIN ... -0.
#[test]
fn limitsFloat() -> ()
{
  // Примитив ровно в потолок остаётся как есть.
  checkExpression!(
    "-1.7976931348623157e308",
    type TokenType::Float,
    value &f64::MIN.to_string()
  );
  // Первый примитив за потолком (парсится в -inf) обрубается до f64::MIN.
  checkExpression!(
    "-1.7976931348623159e308",
    type TokenType::Float,
    value &format!("{:e}", f64::MIN)
  );
  // Примитив 1e309 за нижним потолком обрубается сразу.
  checkExpression!(
    "-1e309",
    type TokenType::Float,
    value &format!("{:e}", f64::MIN)
  );
  // -1e308 - 1e308 обрубается до f64::MIN, вычитание = 0.
  checkExpression!(
    "(-1e308 - 1e308) - (-1e308 - 1e308)",
    type TokenType::Float,
    value "0"
  );
  // f64::MIN + 1e308, а не -inf + 1e308.
  checkExpression!(
    "(-1e308 - 1e308) + 1e308",
    type TokenType::Float,
    value &(f64::MIN + 1e308).to_string()
  );
  // UFloat - UFloat с отрицательным результатом остаётся Float.
  checkExpression!(
    "1.0 - 2.0",
    type TokenType::Float,
    value "-1"
  );
}

// =================================================================================================
// Известные расхождения: ожидания верные по потолкам, но пока падают. Запуск: `-- --ignored`.

/// UInt больше isize::MAX в смешанной операции с Int обрубается до isize::MAX (Value::UInt + Value::Int).
///
/// Примитив после `-` читается как Int (`- 1` = `+ -1`), и UInt уходит через toI64().
#[test]
#[ignore = "UInt > isize::MAX с Int-примитивом обрубается до isize::MAX"]
fn limitsUIntMixedWithInt() -> ()
{
  checkExpression!(
    &format!("{} - 1", usize::MAX),
    value &(usize::MAX - 1).to_string()
  );
}

/// Отрицание `isize::MIN` снимает `-` и даёт |isize::MIN|: оно не влезает в isize
/// и обрубается до isize::MAX, поэтому `MIN - MIN` даёт -1.
#[test]
#[ignore = "isize::MIN - isize::MIN даёт -1 (отрицание isize::MIN вне isize)"]
fn limitsIntNegateMin() -> ()
{
  checkExpression!(
    &format!("({min} - 1) - ({min} - 1)", min = isize::MIN),
    value "0"
  );
}

/// `*` отключён в expression (todo в structure.rs): правая часть пропадает.
/// Кейсы выбраны так, чтобы без умножения они не проходили.
#[test]
#[ignore = "`*` отключён в expression"]
fn limitsMultiply() -> ()
{
  checkExpression!(
    &format!("{} * 3", isize::MAX),
    type TokenType::UInt,
    value &usize::MAX.to_string()
  );
  checkExpression!(
    &format!("-{} * 2", isize::MAX),
    type TokenType::Int,
    value &isize::MIN.to_string()
  );
  checkExpression!(
    "1e308 * 10",
    type TokenType::UFloat,
    value &f64::MAX.to_string()
  );
}

// =================================================================================================
