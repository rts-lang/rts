use crate::parser::structure::structureType::{StructureType, PrimitiveKind};
use crate::parser::testing::checkStructure;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// Это тесты на объединения `a: U8 | String` (issue #59) и примитивы как варианты `a: 10 | 20`.
//
// todo Я бы раскрыл все параметры т.к. они длинные и смешиваются.
//
// todo Есть баг еще вроде как с U8 = None будет 0, что ошибка. НО оно не к этому к StructureType.

// =================================================================================================

/// Числовой примитив как вариант типизации.
fn number(data: &str) -> StructureType
{
  StructureType::Primitive(PrimitiveKind::Number, String::from(data))
}

/// Строковый примитив как вариант типизации.
fn text(data: &str) -> StructureType
{
  StructureType::Primitive(PrimitiveKind::Text, String::from(data))
}

// =================================================================================================

/// Один вариант — обычный тип; несколько — Union (issue #59).
#[test]
fn unionParse() -> ()
{
  checkStructure!("a: U8 = 10", a, utype StructureType::U8);
  checkStructure!("a: String = \"x\"", a, utype StructureType::String);
  checkStructure!(
    "a: U8 | String = 10", a,
    utype StructureType::Union(vec![StructureType::U8, StructureType::String])
  );
  checkStructure!(
    "b: U8|String = 10", b, // Без пробелов.
    utype StructureType::Union(vec![StructureType::U8, StructureType::String])
  );
  checkStructure!(
    "a: I8 | U8 | F64 | None = 1", a,
    utype StructureType::Union(vec![
      StructureType::I8, StructureType::U8, StructureType::F64, StructureType::None
    ])
  );

  // Ключевые слова-типы без данных - настоящие имена типов.
  checkStructure!(
    "a: UInt | Int = 1", a,
    utype StructureType::Union(vec![
      StructureType::Custom(String::from("UInt")), StructureType::Custom(String::from("Int"))
    ])
  );

  // Повторы схлопываются.
  checkStructure!(
    "a: U8 | U8 | String = 10", a,
    utype StructureType::Union(vec![StructureType::U8, StructureType::String])
  );
}

/// Значение ложится в тот вариант, в который помещается как есть (issue #59).
#[test]
fn unionExactMatch() -> ()
{
  checkStructure!("a: U8 | String = 10", a, stype StructureType::U8, value "10");
  checkStructure!("a: U8 | String = \"hi\"", a, stype StructureType::String, value "hi");
  checkStructure!("a: U8 | F32 = 1.5", a, stype StructureType::F32, value "1.5");

  // Из нескольких подходящих вариантов выбирается тот, в который значение
  // помещается без потерь, а не первый в списке:
  // -10 помещается в I8, поэтому U8 | I8 даёт I8, а не зажатое в U8 ноль.
  checkStructure!("a: U8 | I8 = -10", a, stype StructureType::I8, value "-10");
  checkStructure!("a: U8 | U32 = 70000", a, stype StructureType::U32, value "70000");
}

/// Не подошёл ни один вариант - приводим в первый, куда приведение возможно (#59/#71).
#[test]
fn unionConvert() -> ()
{
  // 300 не помещается в U8, но приводится в него с зажимом (#71).
  checkStructure!("a: U8 | String = 300", a, stype StructureType::U8, value "255");

  // Отрицательное в беззнаковый - приведение.
  checkStructure!("a: U8 | String = -10", a, stype StructureType::U8, value "0");

  // Float приводится в целый вариант с округлением.
  checkStructure!("a: U8 | String = 1.5", a, stype StructureType::U8, value "2");
}

/// Ни один вариант не подошёл и привести нельзя - None (issue #59).
#[test]
fn unionNone() -> ()
{
  // True/False не приводятся ни к числу, ни к строке.
  checkStructure!("a: U8 | String = True", a, stype StructureType::None, value "");

  // Ссылка на несуществующую структуру - тоже None.
  checkStructure!("a: U8 | String = b.c", a, stype StructureType::None, value "");

  // Явный `| None` в объединении: значение не подходит - всё равно None.
  checkStructure!("a: String | None = True", a, stype StructureType::None, value "");

  // Само None в объединении - законный вариант.
  checkStructure!("a: String | None = None", a, stype StructureType::None, value "");
}

/// Примитивы - варианты типизации наравне с именами типов (type-секция слева от `=`).
#[test]
fn unionPrimitiveParse() -> ()
{
  let tenTwenty: StructureType = StructureType::Union(vec![number("10"), number("20")]);
  checkStructure!("a: 10 | 20 = 10", a, utype tenTwenty.clone());
  checkStructure!("a: 10|20 = 10", a, utype tenTwenty.clone()); // Без пробелов.
  checkStructure!(
    "a: \"text\" | 10 = 10", a,
    utype StructureType::Union(vec![text("text"), number("10")])
  );
  checkStructure!(
    "a: \"name\" | U8 = 10", a,
    utype StructureType::Union(vec![text("name"), StructureType::U8])
  );
  checkStructure!(
    "a: U8 | 10 = 10", a,
    utype StructureType::Union(vec![StructureType::U8, number("10")])
  );
  checkStructure!("a: 1.5 | 2 = 2", a, utype StructureType::Union(vec![number("1.5"), number("2")]));

  // Отрицательное число - один токен Int.
  checkStructure!("a: -1 | 1 = 1", a, utype StructureType::Union(vec![number("-1"), number("1")]));

  // Одиночный примитив - Union из одного варианта, иначе значение не проверялось бы.
  checkStructure!("a: 10 = 10", a, utype StructureType::Union(vec![number("10")]));
  checkStructure!("a: \"x\" = \"x\"", a, utype StructureType::Union(vec![text("x")]));

  // Повторы схлопываются.
  checkStructure!("a: 10 | 10 | 20 = 10", a, utype tenTwenty.clone());

  // Незакрытый `|` не ломает разбор.
  checkStructure!("a: U8 | = 10", a, utype StructureType::U8);
  checkStructure!("a: 10 | = 10", a, utype StructureType::Union(vec![number("10")]));
}

/// Значение, точно совпавшее с примитивом, остаётся как есть;
/// `stype` - сам примитив, а не natural U8/F32.
#[test]
fn unionPrimitiveMatch() -> ()
{
  checkStructure!("a: 10 | 20 = 10", a, stype number("10"), value "10");
  checkStructure!("a: 10 | 20 = 20", a, stype number("20"), value "20");

  // Числа равны по значению, а не по записи: Float `10.0` совпал с примитивом `10`,
  // а токен остался UFloat.
  checkStructure!("a: 10 | 20 = 10.0", a, type TokenType::UFloat, stype number("10"), value "10");
  checkStructure!("a: -1 | 1 = -1", a, stype number("-1"), value "-1");
  checkStructure!("a: -1 | 1 = 1", a, stype number("1"), value "1");

  checkStructure!("a: \"x\" | 10 = \"x\"", a, stype text("x"), value "x");
  checkStructure!("a: \"x\" | 10 = 10", a, stype number("10"), value "10");
}

/// Значение вне примитивов - `None` (#71): без приведения, без зажима и без явного `None`.
#[test]
fn unionPrimitiveNone() -> ()
{
  // Ближайшее число не подставляется: только точное совпадение.
  checkStructure!("a: 10 | 20 = 15", a, stype StructureType::None, value "");
  checkStructure!("a: 10 | 20 = 300", a, stype StructureType::None, value "");
  checkStructure!("a: 10 | 20 = 10.5", a, stype StructureType::None, value "");

  // Другой вид значения.
  checkStructure!("a: 10 | 20 = \"text\"", a, stype StructureType::None, value "");
  checkStructure!("a: 10 | 20 = \"10\"", a, stype StructureType::None, value "");
  checkStructure!("a: 10 | 20 = True", a, stype StructureType::None, value "");

  // `None` как значение - тоже `None`, хотя в объединении он не указан.
  checkStructure!("a: 10 | 20 = None", a, stype StructureType::None, value "");

  checkStructure!("a: \"x\" | 10 = \"y\"", a, stype StructureType::None, value "");
  checkStructure!("a: \"x\" | 10 = True", a, stype StructureType::None, value "");
  checkStructure!("a: \"x\" | 10 = 11", a, stype StructureType::None, value "");
}

/// Примитив вместе с типом: сначала точный примитив, потом тип как есть, потом приведение.
#[test]
fn unionPrimitiveWithType() -> ()
{
  // Приоритет: примитив раньше типа.
  checkStructure!("a: U8 | 10 = 10", a, stype number("10"), value "10");

  // Не примитив - работают правила типов: приведение с зажимом (#71).
  checkStructure!("a: U8 | 10 = 11", a, stype StructureType::U8, value "11");
  checkStructure!("a: U8 | 10 = 300", a, stype StructureType::U8, value "255");
  checkStructure!("a: U8 | 10 = -5", a, stype StructureType::U8, value "0");

  // Строковый примитив рядом с типом: матч примитива → stype = примитив.
  checkStructure!("a: \"name\" | U8 = \"name\"", a, stype text("name"), value "name");
  checkStructure!("a: \"name\" | U8 = \"other\"", a, stype StructureType::None, value "");
  checkStructure!("a: \"name\" | U8 = 300", a, stype StructureType::U8, value "255");
}

/// Смешанный union: тип + примитив — stype примитив при точном матче.
#[test]
fn unionPrimitiveMixedType() -> ()
{
  checkStructure!("a: String | 10 = 10", a, stype number("10"), value "10");
  checkStructure!("a: String | 10 = \"hi\"", a, stype StructureType::String, value "hi");
  checkStructure!("a: String | 10 = 11", a, stype StructureType::None, value "");

  checkStructure!("a: String | \"text\" = \"text\"", a, stype text("text"), value "text");
  checkStructure!("a: String | \"text\" = \"other\"", a, stype StructureType::String, value "other");
}

// =================================================================================================

// Дальше чистые функции StructureType: из кода до них нет пути, поэтому без макроса.

/// `variants()` раскладывает тип в список вариантов.
#[test]
fn unionVariants() -> ()
{
  let union: StructureType = StructureType::Union(vec![StructureType::U8, StructureType::String]);
  assert!(union.variants() == vec![StructureType::U8, StructureType::String]);

  // Одиночный тип - один вариант.
  assert!(StructureType::U8.variants() == vec![StructureType::U8]);

  // Пустое объединение и `None` - пустой список, то есть тип не указан.
  assert!(StructureType::Union(vec![]).variants().is_empty());
  assert!(StructureType::None.variants().is_empty());
}

/// Union печатается так же, как записывается в коде.
#[test]
fn unionToString() -> ()
{
  assert_eq!(StructureType::Union(
    vec![StructureType::U8, StructureType::String]
  ).to_string(), "U8 | String");
  assert_eq!(StructureType::Union(
    vec![StructureType::I8, StructureType::F64, StructureType::None]
  ).to_string(), "I8 | F64 | None");

  // Вложенный union схлопывается при разборе, но и сам печатается нормально.
  assert_eq!(StructureType::Union(vec![
    StructureType::Union(vec![StructureType::U8, StructureType::String]),
    StructureType::U16
  ]).to_string(), "U8 | String | U16");

  // Примитивы: число как есть, строка в кавычках.
  assert_eq!(StructureType::Union(vec![number("10"), number("20")]).to_string(), "10 | 20");
  assert_eq!(StructureType::Union(vec![text("text"), StructureType::U8]).to_string(), "\"text\" | U8");
}

/// abiType сводит примитив к storage-форме.
#[test]
fn primitiveAbiType() -> ()
{
  assert!(text("text").abiType() == StructureType::String);
  assert!(number("10").abiType() == StructureType::U8);
  assert!(number("-1").abiType() == StructureType::I8);
  assert!(StructureType::U8.abiType() == StructureType::U8);
}

// =================================================================================================
