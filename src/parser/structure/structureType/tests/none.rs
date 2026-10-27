use crate::parser::structure::structure::StructureMut;
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::parser::testing::{checkStructure, primitiveValueNumber, primitiveValueText};
// =================================================================================================

// Тесты на `None` в качестве типизации.
//
// `None` - не явный примитив: он может появиться у любой структуры сам из-за
// константного и неопределённое поведение.
//
// 
// Хранение и удаление `None` описаны отдельно:
//
// - `a = None` - Хранит значение, структура жива;
//
// - `a := None` - Удаляет структуру.
//
//
// Поэтому в типизацию структуры и в `union` его не вписывают: парсер просто его отбрасывает.
//
// - `a: None` - То же самое, что просто `a`;
//
// - `a: U8 | None` - То же самое, что просто `a: U8`.

// =================================================================================================
// Одиночный `None` в типе - это отсутствие типа.

/// `a: None := 5` - то же, что `a := 5`: тип выводится из значения.
#[test]
fn noType() -> ()
{
  checkStructure!(
    "a: None := 5", "a",
    type UInt,
    stype U8,
    value "5",
    mut StructureMut::Constant
  );
  checkStructure!(
    "a := 5", "a",
    type UInt,
    stype U8,
    value "5",
    mut StructureMut::Constant
  );

  // Для каждой изменяемости.
  checkStructure!(
    "a~: None := 5", "a",
    type UInt,
    stype U8,
    value "5",
    mut StructureMut::Variable
  );
  checkStructure!(
    "a~ := 5", "a",
    type UInt,
    stype U8,
    value "5",
    mut StructureMut::Variable
  );
  checkStructure!(
    "a~~: None := 5", "a",
    type UInt,
    stype U8,
    value "5",
    mut StructureMut::Dynamic
  );
  checkStructure!(
    "a~~ := 5", "a",
    type UInt,
    stype U8,
    value "5",
    mut StructureMut::Dynamic
  );
}

/// `a: None = 5` - то же, что `a = 5`: `=` структуру не создаёт.
#[test]
#[should_panic(expected = "Structure 'a' was not created")]
fn equalsSkipped() -> ()
{
  checkStructure!(
    "a: None = 5", "a",
    value "5"
  );
}

/// То же для `a~: None = 5`.
#[test]
#[should_panic(expected = "Structure 'a' was not created")]
fn variableEqualsSkipped() -> ()
{
  checkStructure!(
    "a~: None = 5", "a",
    value "5"
  );
}

/// `a: None` - то же, что `a`: структура ждёт первого значения и становится `Constant`.
#[test]
fn forward() -> ()
{
  checkStructure!(
    r#"
    a: None
    a = 5
    a = 300
    "#,
    "a",
    type UInt,
    stype U8,
    value "5",
    mut StructureMut::Constant
  );
  checkStructure!(
    r#"
    a
    a = 5
    a = 300
    "#,
    "a",
    type UInt,
    stype U8,
    value "5",
    mut StructureMut::Constant
  );
}

/// Несколько `None` подряд - тоже отсутствие типа.
#[test]
fn onlyNones() -> ()
{
  checkStructure!(
    "a: None | None := 5", "a",
    type UInt,
    stype U8,
    value "5",
    mut StructureMut::Constant
  );
}

// =================================================================================================
// `None` внутри `union` отбрасывается.

/// `None` в конце, в начале и в середине объединения.
#[test]
fn droppedFromUnion() -> ()
{
  let i8OrU8 = || StructureType::Union(vec![
    StructureType::Primitive(PrimitiveType::I8),
    StructureType::Primitive(PrimitiveType::U8)
  ]);

  checkStructure!(
    "a: I8 | U8 | None = 300", "a",
    stype I8,
    utype i8OrU8(),
    value "127"
  );
  checkStructure!(
    "a: None | I8 | U8 = 300", "a",
    stype I8,
    utype i8OrU8(),
    value "127"
  );
  checkStructure!(
    "a: I8 | None | U8 = 300", "a",
    stype I8,
    utype i8OrU8(),
    value "127"
  );
  checkStructure!(
    "a: I8 | U8 = 300", "a",
    stype I8,
    utype i8OrU8(),
    value "127"
  );

  // С отрицательным значением.
  checkStructure!(
    "a: I8 | U8 | None = -300", "a",
    type Int,
    stype I8,
    utype i8OrU8(),
    value "-128"
  );
  checkStructure!(
    "a: I8 | U8 = -300", "a",
    type Int,
    stype I8,
    utype i8OrU8(),
    value "-128"
  );
}

/// После отбрасывания из `U8 | None` остаётся обычный `U8`, а не `union` из одного варианта.
#[test]
fn singleLeft() -> ()
{
  checkStructure!(
    "a: U8 | None = 300", "a",
    stype U8,
    utype U8,
    value "255"
  );
  checkStructure!(
    "a: None | U8 = 300", "a",
    stype U8,
    utype U8,
    value "255"
  );
  checkStructure!(
    "a: U8 = 300", "a",
    stype U8,
    utype U8,
    value "255"
  );
}

/// `True | False | None` - это `True | False`.
#[test]
fn droppedFromLogic() -> ()
{
  let trueOrFalse = || StructureType::Union(vec![
    StructureType::Primitive(PrimitiveType::True),
    StructureType::Primitive(PrimitiveType::False)
  ]);

  checkStructure!(
    "a: True | False | None = True", "a",
    type True,
    stype True,
    utype trueOrFalse(),
    value "True"
  );
  checkStructure!(
    "a: True | False = True", "a",
    type True,
    stype True,
    utype trueOrFalse(),
    value "True"
  );
  checkStructure!(
    "a: True | False | None = False", "a",
    type False,
    stype False,
    utype trueOrFalse(),
    value "False"
  );
  checkStructure!(
    "a: True | False = False", "a",
    type False,
    stype False,
    utype trueOrFalse(),
    value "False"
  );
}

/// Примитивы-значения рядом с `None`: остаётся `union` без него.
#[test]
fn droppedNextToValues() -> ()
{
  checkStructure!(
    "a: 10 | None = 10", "a",
    stype primitiveValueNumber("10"),
    utype StructureType::Union(vec![primitiveValueNumber("10")]),
    value "10"
  );
  checkStructure!(
    "a: 10 = 10", "a",
    stype primitiveValueNumber("10"),
    utype StructureType::Union(vec![primitiveValueNumber("10")]),
    value "10"
  );
  checkStructure!(
    "a: 10 | 20 | None = 20", "a",
    stype primitiveValueNumber("20"),
    utype StructureType::Union(vec![primitiveValueNumber("10"),
      primitiveValueNumber("20")]),
    value "20"
  );
  checkStructure!(
    "a: \"x\" | None = \"x\"", "a",
    stype primitiveValueText("x"),
    utype StructureType::Union(vec![primitiveValueText("x")]),
    value "x"
  );
}

/// Объявление без значения, затем присваивание: `None` отбрасывается так же.
#[test]
fn droppedOnAssign() -> ()
{
  checkStructure!(
    r#"
    a~: U8 | None
    a = 300
    "#,
    "a",
    type UInt,
    stype U8,
    utype U8,
    value "255",
    mut StructureMut::Variable
  );
  checkStructure!(
    r#"
    a~: U8
    a = 300
    "#,
    "a",
    type UInt,
    stype U8,
    utype U8,
    value "255",
    mut StructureMut::Variable
  );

  // `~~` свободно меняет тип: `300` уже `U16`.
  checkStructure!(
    r#"
    a~~: U8 | None
    a = 300
    "#,
    "a",
    type UInt,
    stype U16,
    utype U16,
    value "300",
    mut StructureMut::Dynamic
  );
  checkStructure!(
    r#"
    a~~: U8
    a = 300
    "#,
    "a",
    type UInt,
    stype U16,
    utype U16,
    value "300",
    mut StructureMut::Dynamic
  );
}

// =================================================================================================
