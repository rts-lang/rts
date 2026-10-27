use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::parser::testing::{checkStructure, primitiveValueNumber, primitiveValueText};
// =================================================================================================

// Тесты на:
//
// - Примитивы: `a: U8 | String = ...`;
//
// - Пользовательские типы: `a: Test1 | Test2 = ...`; todo Не уверен что оно тут полностью есть.
// 
// - Примитив-значение: `a: 10 | 20 = 10`.
//
//
// todo Есть баг еще вроде как с `U8 = None` будет `0`, что ошибка. НО оно не к этому к `StructureType`.

// =================================================================================================

/// Один `вариант` — обычный тип; несколько — `union`.
#[test]
fn unionParse() -> ()
{
  checkStructure!(
    "a: U8 = 10", "a",
    utype U8)
  ;
  checkStructure!(
    "a: String = \"x\"", "a",
    utype String
  );
  checkStructure!(
    "a: U8 | String = 10", "a",
    utype StructureType::Union(vec![
      StructureType::Primitive(PrimitiveType::U8),
      StructureType::Primitive(PrimitiveType::String)
    ])
  );
  checkStructure!(
    "b: U8|String = 10", "b", // Без пробелов.
    utype StructureType::Union(vec![
      StructureType::Primitive(PrimitiveType::U8),
      StructureType::Primitive(PrimitiveType::String)
    ])
  );
  // `None` - не явный примитив: из объединения он отбрасывается.
  checkStructure!(
    "a: I8 | U8 | F64 | None = 1", "a",
    utype StructureType::Union(vec![
      StructureType::Primitive(PrimitiveType::I8),
      StructureType::Primitive(PrimitiveType::U8),
      StructureType::Primitive(PrimitiveType::F64)
    ])
  );

  // Пользовательские структуры.
  checkStructure!(
    "a: Test1 | Test2 = 1", "a",
    utype StructureType::Union(vec![
      StructureType::Primitive(PrimitiveType::Custom(String::from("Test1"))),
      StructureType::Primitive(PrimitiveType::Custom(String::from("Test2")))
    ])
  );

  // Повторы схлопываются.
  checkStructure!(
    "a: U8 | U8 | String = 10", "a",
    utype StructureType::Union(vec![
      StructureType::Primitive(PrimitiveType::U8),
      StructureType::Primitive(PrimitiveType::String)
    ])
  );
}

/// Значение ложится в тот `вариант`, в который помещается как есть.
#[test]
fn unionExactMatch() -> ()
{
  checkStructure!(
    "a: U8 | String = 10", "a",
    stype U8,
    value "10"
  );
  checkStructure!(
    "a: U8 | String = \"hi\"", "a",
    stype String,
    value "hi"
  );
  checkStructure!(
    "a: U8 | F32 = 1.5", "a",
    stype F32,
    value "1.5"
  );

  // Из нескольких подходящих `вариантов` выбирается тот, в который значение помещается без потерь:
  // `-10` помещается в `I8`, поэтому `U8 | I8` даёт `I8`, а не зажатое в `U8` ноль.
  checkStructure!(
    "a: U8 | I8 = -10", "a",
    stype I8,
    value "-10"
  );
  checkStructure!(
    "a: U8 | U32 = 70000", "a",
    stype U32,
    value "70000"
  );
}

/// Не подошёл ни один `вариант` - приводим в первый, куда приведение возможно.
#[test]
fn unionConvert() -> ()
{
  // `300` не помещается в `U8`, но приводится в него с зажимом.
  checkStructure!(
    "a: U8 | String = 300", "a",
    stype U8,
    value "255"
  );

  // Отрицательное в беззнаковый - приведение.
  checkStructure!(
    "a: U8 | String = -10", "a",
    stype U8,
    value "0"
  );

  // `Float` приводится в целый `вариант` с округлением.
  checkStructure!(
    "a: U8 | String = 1.5", "a",
    stype U8,
    value "2"
  );
}

/// `None` для `union`.
#[test]
fn unionNone() -> ()
{
  // `True`/`False` не приводятся ни к числу, ни к строке.
  checkStructure!(
    "a: U8 | String = True", "a",
    stype None,
    value ""
  );

  // Ссылка на несуществующую структуру - тоже `None`.
  checkStructure!(
    "a: U8 | String = b.c", "a",
    stype None,
    value ""
  );

  // `None` - не явный примитив: `| None` отбрасывается и остаётся обычный тип без `union`.
  checkStructure!(
    "a: String | None = \"x\"", "a",
    utype StructureType::Primitive(PrimitiveType::String),
    value "x"
  );
  checkStructure!(
    "a: None | String = \"x\"", "a",
    utype StructureType::Primitive(PrimitiveType::String),
    value "x"
  );
}

/// Примитивы - `варианты` типизации наравне с именами типов (type-секция слева от `=`).
#[test]
fn unionPrimitiveParse() -> ()
{
  let tenTwenty: StructureType = StructureType::Union(vec![
    primitiveValueNumber("10"),
    primitiveValueNumber("20")
  ]);
  
  checkStructure!(
    "a: 10 | 20 = 10", "a",
    utype tenTwenty.clone()
  );
  checkStructure!(
    "a: 10|20 = 10", "a", // Без пробелов.
    utype tenTwenty.clone()
  );
  checkStructure!(
    "a: \"text\" | 10 = 10", "a",
    utype StructureType::Union(vec![primitiveValueText("text"), primitiveValueNumber("10")])
  );
  checkStructure!(
    "a: \"name\" | U8 = 10", "a",
    utype StructureType::Union(vec![
      primitiveValueText("name"),
      StructureType::Primitive(PrimitiveType::U8)
    ])
  );
  checkStructure!(
    "a: U8 | 10 = 10", "a",
    utype StructureType::Union(vec![
      StructureType::Primitive(PrimitiveType::U8),
      primitiveValueNumber("10")
    ])
  );
  checkStructure!(
    "a: 1.5 | 2 = 2", "a",
    utype StructureType::Union(vec![
      primitiveValueNumber("1.5"),
      primitiveValueNumber("2")
    ])
  );

  // Отрицательное число - один токен `Int`.
  checkStructure!(
    "a: -1 | 1 = 1", "a",
    utype StructureType::Union(vec![
      primitiveValueNumber("-1"),
      primitiveValueNumber("1")
    ])
  );

  // Одиночный примитив - `union` из одного `варианта`, иначе значение не проверялось бы.
  checkStructure!(
    "a: 10 = 10", "a",
    utype StructureType::Union(vec![primitiveValueNumber("10")])
  );
  checkStructure!(
    "a: \"x\" = \"x\"", "a",
    utype StructureType::Union(vec![primitiveValueText("x")])
  );

  // Повторы схлопываются.
  checkStructure!(
    "a: 10 | 10 | 20 = 10", "a",
    utype tenTwenty.clone());

  // Незакрытый `|` не ломает разбор.
  checkStructure!(
    "a: U8 | = 10", "a",
    utype U8
  );
  checkStructure!(
    "a: 10 | = 10", "a",
    utype StructureType::Union(vec![primitiveValueNumber("10")])
  );
}

/// Значение, точно совпавшее с примитивом, остаётся как есть;
/// 
/// `stype` - сам примитив, а не natural `U8`/`F32`.
#[test]
fn unionPrimitiveMatch() -> ()
{
  checkStructure!(
    "a: 10 | 20 = 10", "a",
    stype primitiveValueNumber("10"),
    value "10"
  );
  checkStructure!(
    "a: 10 | 20 = 20", "a",
    stype primitiveValueNumber("20"),
    value "20"
  );

  // Числа равны по значению, а не по записи: `Float 10.0` совпал с примитивом `10`,
  // а токен остался `UFloat`.
  checkStructure!(
    "a: 10 | 20 = 10.0", "a",
    type UFloat,
    stype primitiveValueNumber("10"),
    value "10"
  );
  checkStructure!(
    "a: -1 | 1 = -1", "a",
    stype primitiveValueNumber("-1"),
    value "-1"
  );
  checkStructure!(
    "a: -1 | 1 = 1", "a",
    stype primitiveValueNumber("1"),
    value "1"
  );

  //
  checkStructure!(
    "a: \"x\" | 10 = \"x\"", "a",
    stype primitiveValueText("x"),
    value "x"
  );
  checkStructure!(
    "a: \"x\" | 10 = 10", "a",
    stype primitiveValueNumber("10"),
    value "10"
  );
}

/// Значение вне примитивов - `None`: без приведения, без зажима и без явного `None`.
#[test]
fn unionPrimitiveNone() -> ()
{
  // Ближайшее число не подставляется: только точное совпадение.
  checkStructure!(
    "a: 10 | 20 = 15", "a",
    stype None,
    value ""
  );
  checkStructure!(
    "a: 10 | 20 = 300", "a",
    stype None,
    value ""
  );
  checkStructure!(
    "a: 10 | 20 = 10.5", "a",
    stype None,
    value ""
  );

  // Другой вид значения.
  checkStructure!(
    "a: 10 | 20 = \"text\"", "a",
    stype None,
    value ""
  );
  checkStructure!(
    "a: 10 | 20 = \"10\"", "a",
    stype None,
    value ""
  );
  checkStructure!(
    "a: 10 | 20 = True", "a",
    stype None,
    value ""
  );

  // `None` как значение - тоже `None`, хотя в объединении он не указан.
  checkStructure!(
    "a: 10 | 20 = None", "a",
    stype None,
    value ""
  );

  //
  checkStructure!(
    "a: \"x\" | 10 = \"y\"", "a",
    stype None,
    value ""
  );
  checkStructure!(
    "a: \"x\" | 10 = True", "a",
    stype None,
    value ""
  );
  checkStructure!(
    "a: \"x\" | 10 = 11", "a",
    stype None,
    value ""
  );
}

/// Примитив вместе с типом: сначала точный примитив, потом тип как есть, потом приведение.
#[test]
fn unionPrimitiveWithType() -> ()
{
  // Приоритет: примитив раньше типа.
  checkStructure!(
    "a: U8 | 10 = 10", "a",
    stype primitiveValueNumber("10"),
    value "10"
  );

  // Не примитив - работают правила типов: приведение с зажимом.
  checkStructure!(
    "a: U8 | 10 = 11", "a",
    stype U8,
    value "11"
  );
  checkStructure!(
    "a: U8 | 10 = 300", "a",
    stype U8,
    value "255"
  );
  checkStructure!(
    "a: U8 | 10 = -5", "a",
    stype U8,
    value "0"
  );

  // Строковый примитив рядом с типом: матч примитива → `stype` = примитив.
  checkStructure!(
    "a: \"name\" | U8 = \"name\"", "a",
    stype primitiveValueText("name"),
    value "name"
  );
  checkStructure!(
    "a: \"name\" | U8 = \"other\"", "a",
    stype None,
    value ""
  );
  checkStructure!(
    "a: \"name\" | U8 = 300", "a",
    stype U8,
    value "255"
  );
}

/// Смешанный `union`: тип + примитив — `stype` примитив при точном матче.
#[test]
fn unionPrimitiveMixedType() -> ()
{
  checkStructure!(
    "a: String | 10 = 10", "a",
    stype primitiveValueNumber("10"),
    value "10"
  );
  checkStructure!(
    "a: String | 10 = \"hi\"", "a",
    stype String,
    value "hi"
  );
  checkStructure!(
    "a: String | 10 = 11", "a",
    stype None,
    value ""
  );

  //
  checkStructure!(
    "a: String | \"text\" = \"text\"", "a",
    stype primitiveValueText("text"),
    value "text"
  );
  checkStructure!(
    "a: String | \"text\" = \"other\"", "a",
    stype String,
    value "other"
  );
}

/// Примитивы и `примитив-значение` различаются.
#[test]
fn unionTypeNameVsPrimitive() -> ()
{
  // Примитивы.
  checkStructure!(
    "a: U8 = 10", "a",
    utype U8
  );
  checkStructure!(
    "a: Any = 10", "a",
    utype Any
  );
  checkStructure!(
    "a: True = True", "a",
    utype True
  );
  checkStructure!(
    "a: False = False", "a",
    utype False
  );

  // Значения-примитивы.
  checkStructure!(
    "a: 10 = 10", "a",
    utype StructureType::Union(vec![primitiveValueNumber("10")])
  );
  checkStructure!(
    "a: \"text\" = \"text\"", "a",
    utype StructureType::Union(vec![primitiveValueText("text")])
  );

  // Примитив и значение-примитв рядом.
  checkStructure!(
    "a: U8 | 10 = 10", "a",
    utype StructureType::Union(vec![
      StructureType::Primitive(PrimitiveType::U8),
      primitiveValueNumber("10")
    ])
  );
}

/// Пустая строка в type-секции - это `значение-примитив`, а не примитив `String`.
#[test]
fn unionEmptyStringPrimitive() -> ()
{
  // Одиночный примитив - `union` из одного `варианта`, как и любой другой примитив.
  checkStructure!(
    "a: \"\" = \"\"", "a",
    stype primitiveValueText(""),
    utype StructureType::Union(vec![primitiveValueText("")])
  );
  checkStructure!(
    "a: \"\" | U8 = \"\"", "a",
    stype primitiveValueText(""),
    utype StructureType::Union(vec![
      primitiveValueText(""),
      StructureType::Primitive(PrimitiveType::U8)
    ])
  );

  // Не подошло ни к одному `варианту` - `None`.
  checkStructure!(
    "a: \"\" | U8 = \"x\"", "a",
    stype None,
    value ""
  );

  // Имя типа `String` - по-прежнему тип, а не значение.
  checkStructure!(
    "a: String = \"\"", "a",
    stype String
  );
}

// =================================================================================================

/// `variants()` раскладывает тип в список `вариантов`.
#[test]
fn unionVariants() -> ()
{
  let union: StructureType = StructureType::Union(vec![
    StructureType::Primitive(PrimitiveType::U8),
    StructureType::Primitive(PrimitiveType::String)
  ]);
  assert!(union.variants() == vec![
    StructureType::Primitive(PrimitiveType::U8),
    StructureType::Primitive(PrimitiveType::String)
  ]);

  // Одиночный тип - один `вариант`.
  assert!(
    StructureType::Primitive(PrimitiveType::U8).variants() == 
    vec![StructureType::Primitive(PrimitiveType::U8)]
  );

  // Пустое объединение и `None` - пустой список, то есть тип не указан.
  assert!(
    StructureType::Union(vec![])
      .variants().is_empty()
  );
  assert!(
    StructureType::Primitive(PrimitiveType::None)
      .variants().is_empty()
  );
}

/// `Union` печатается так же, как записывается в коде.
#[test]
fn unionToString() -> ()
{
  assert_eq!(StructureType::Union(vec![
    StructureType::Primitive(PrimitiveType::U8),
    StructureType::Primitive(PrimitiveType::String)
  ]).to_string(), "U8 | String");
  assert_eq!(StructureType::Union(vec![
    StructureType::Primitive(PrimitiveType::I8),
    StructureType::Primitive(PrimitiveType::F64),
    StructureType::Primitive(PrimitiveType::None)
  ]).to_string(), "I8 | F64 | None");

  // Вложенный `union` схлопывается при разборе, но и сам печатается нормально.
  assert_eq!(StructureType::Union(vec![
    StructureType::Union(vec![
      StructureType::Primitive(PrimitiveType::U8),
      StructureType::Primitive(PrimitiveType::String)
    ]),
    StructureType::Primitive(PrimitiveType::U16)
  ]).to_string(), "U8 | String | U16");

  // Примитивы: число как есть, строка в кавычках.
  assert_eq!(StructureType::Union(vec![
    primitiveValueNumber("10"),
    primitiveValueNumber("20")
  ]).to_string(), "10 | 20");
  assert_eq!(StructureType::Union(vec![
    primitiveValueText("text"),
    StructureType::Primitive(PrimitiveType::U8)
  ]).to_string(), "\"text\" | U8");
}

/// `abiType` сводит примитив к storage-форме.
#[test]
fn primitiveAbiType() -> ()
{
  assert!(
    primitiveValueText("text").abiType() ==
    StructureType::Primitive(PrimitiveType::String)
  );
  assert!(
    primitiveValueNumber("10").abiType() ==
    StructureType::Primitive(PrimitiveType::U8)
  );
  assert!(
    primitiveValueNumber("-1").abiType() ==
    StructureType::Primitive(PrimitiveType::I8)
  );
  
  //
  assert!(
    StructureType::Primitive(PrimitiveType::U8).abiType() ==
    StructureType::Primitive(PrimitiveType::U8)
  );
}

// =================================================================================================

/// `Dynamic ~~` + `union`: тип меняется свободно, объединение не ограничивает.
#[test]
fn unionDynamic() -> ()
{
  checkStructure!(
    "j~~: U8 | String = 10", "j",
    type UInt,
    stype U8,
    value "10"
  );
  checkStructure!(
    r#"
    j~~: U8 | String = 10
    j = 1.5
    "#,
    "j",
    type UFloat,
    stype F32,
    value "1.5"
  );
  checkStructure!(
    r#"
    j~~: U8 | String = 10
    j = 99999
    "#,
    "j",
    type UInt,
    stype U32,
    value "99999"
  );
}

/// `Variable ~` + примитивы в `union`: набор остаётся и ограничивает.
#[test]
fn unionVariablePrimitives() -> ()
{
  checkStructure!(
    r#"
    t~: 10 | 20 = 10
    t = 20
    "#,
    "t",
    type UInt,
    stype primitiveValueNumber("20"),
    value "20"
  );
  checkStructure!(
    r#"
    t~: 10 | 20 = 10
    t = 30
    "#,
    "t",
    stype None,
    value ""
  );
  checkStructure!(
    r#"
    t~: 10 | 20 = 10
    t = 10
    "#,
    "t",
    type UInt,
    stype primitiveValueNumber("10"),
    value "10"
  );
}

/// `Variable ~` + строковый примитив в `union`: переприсваивание.
#[test]
fn unionVariableStringPrimitive() -> ()
{
  checkStructure!(
    r#"
    v~: String | "text" = "text"
    "#,
    "v",
    type String,
    stype primitiveValueText("text"),
    value "text"
  );
  checkStructure!(
    r#"
    v~: String | "text" = "text"
    v = "other"
    "#,
    "v",
    type String,
    stype String,
    value "other"
  );
}

// =================================================================================================
