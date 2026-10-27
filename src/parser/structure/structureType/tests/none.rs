use crate::parser::structure::structure::Structure;
use crate::parser::testing::{getStructure, runCode};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, RwLock, RwLockReadGuard};
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

/// Читает грани структуры `a` после выполнения кода строкой вида `type=UInt, stype=U8, ...`.
///
/// Если код упал (структуры нет и т.п.) — вернёт `error: сообщение`, чтобы сравнение не пропало.
fn facets(code: &str, names: &[&str]) -> String
{
  let result = catch_unwind(AssertUnwindSafe(||
  {
    let main: Arc<RwLock<Structure>> = runCode!(code);
    let structureLink: Arc<RwLock<Structure>> = getStructure(&main, "a");
    let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();

    let token: Option<(String, String)> = structure.lines.as_ref()
      .and_then(|lines| lines.first())
      .and_then(|line| line.read().unwrap().tokens.clone())
      .and_then(|tokens| tokens.first().map(|token|
        (token.getDataType().to_string(), token.getData().toString().unwrap_or_default())
      ));

    let mut parts: Vec<String> = Vec::new();
    for name in names
    {
      parts.push(match *name
      {
        "type"  => format!("type={}",  token.as_ref().map(|t| t.0.clone()).unwrap_or(String::from("<none>"))),
        "value" => format!("value={}", token.as_ref().map(|t| t.1.clone()).unwrap_or(String::from("<none>"))),
        "stype" => format!("stype={}", structure.dataType.to_string()),
        "utype" => format!("utype={}", structure.utype().to_string()),
        "mut"   => format!("mut={}",   structure.mutable.to_string()),
        other   => panic!("unknown facet '{}'", other)
      });
    }
    parts.join(", ")
  }));

  match result
  {
    Ok(text) => text,
    Err(payload) =>
    {
      let message: String =
        if let Some(text) = payload.downcast_ref::<&str>() { text.to_string() } else
        if let Some(text) = payload.downcast_ref::<String>() { text.clone() } else
        { String::from("unknown panic") };
      format!("error: {}", message)
    }
  }
}

const ALL: &[&str] = &["type", "stype", "utype", "value", "mut"];

/// Запись с `None` в типе должна дать то же, что запись без него.
///
/// Структура `a` в обоих случаях должна быть создана: совпадение двух одинаковых ошибок
/// ничего не доказывает.
fn assertSame(withNone: &str, withoutNone: &str) -> ()
{
  let first: String = facets(withNone, ALL);
  let second: String = facets(withoutNone, ALL);

  assert!(!first.starts_with("error:"), "\n  {:?}\n    -> {}\n", withNone, first);

  assert_eq!(
    first, second,
    "\n  с None:   {:?}\n    -> {}\n  без None: {:?}\n    -> {}\n",
    withNone, first, withoutNone, second
  );
}

// =================================================================================================
// Одиночный `None` в типе - это отсутствие типа.

/// `a: None := 5` — то же, что `a := 5`.
#[test]
fn noneTypeIsNoType() -> ()
{
  assertSame("a: None := 5", "a := 5");
  assertSame("a~: None := 5", "a~ := 5");
  assertSame("a~~: None := 5", "a~~ := 5");
}

/// `a: None = 5` — то же, что `a = 5`: `=` структуру не создаёт.
#[test]
fn noneTypeWithEqualsIsSkipped() -> ()
{
  for (withNone, withoutNone) in [("a: None = 5", "a = 5"), ("a~: None = 5", "a~ = 5")]
  {
    let first: String = facets(withNone, ALL);
    let second: String = facets(withoutNone, ALL);

    // Структуры нет в обоих случаях, и причина одна.
    assert!(first.starts_with("error: Structure 'a' was not created"), "{:?} -> {}", withNone, first);
    assert_eq!(first, second, "{:?} и {:?}", withNone, withoutNone);
  }
}

/// `a: None` — то же, что `a`: структура ждёт первого значения.
#[test]
fn noneTypeForwardDeclaration() -> ()
{
  assertSame("a: None\na = 5", "a\na = 5");
  assertSame("a: None\na = 5\na = 300", "a\na = 5\na = 300");
}

/// Несколько `None` подряд — тоже отсутствие типа.
#[test]
fn onlyNonesAreNoType() -> ()
{
  assertSame("a: None | None := 5", "a := 5");
}

// =================================================================================================
// `None` внутри `union` отбрасывается.

/// `None` в конце, в начале и в середине объединения.
#[test]
fn noneIsDroppedFromUnion() -> ()
{
  assertSame("a: U8 | None = 300", "a: U8 = 300");
  assertSame("a: None | U8 = 300", "a: U8 = 300");
  assertSame("a: I8 | None | U8 = 300", "a: I8 | U8 = 300");
  assertSame("a: I8 | U8 | None = -300", "a: I8 | U8 = -300");
}

/// После отбрасывания из `U8 | None` остаётся обычный `U8`, а не `union` из одного варианта.
#[test]
fn singleTypeLeftAfterDroppingNone() -> ()
{
  let facetsText: String = facets("a: U8 | None = 300", &["stype", "utype", "value"]);
  assert_eq!(facetsText, "stype=U8, utype=U8, value=255");
}

/// `True | False | None` — это `True | False`.
#[test]
fn noneIsDroppedFromLogicUnion() -> ()
{
  assertSame("a: True | False | None = True", "a: True | False = True");
  assertSame("a: True | False | None = False", "a: True | False = False");
}

/// Примитивы-значения рядом с `None`.
#[test]
fn noneIsDroppedNextToPrimitiveValues() -> ()
{
  assertSame("a: 10 | None = 10", "a: 10 = 10");
  assertSame("a: 10 | 20 | None = 20", "a: 10 | 20 = 20");
  assertSame("a: \"x\" | None = \"x\"", "a: \"x\" = \"x\"");
}

/// Объявление без значения, затем присваивание.
#[test]
fn noneIsDroppedForVariableAssign() -> ()
{
  assertSame("a~: U8 | None\na = 300", "a~: U8\na = 300");
  assertSame("a~~: U8 | None\na = 300", "a~~: U8\na = 300");
}

// =================================================================================================
// Хранение и удаление `None` этим не затронуты.

/// `a = None` хранит `None`, структура остаётся жива и сохраняет свой тип.
#[test]
fn storingNoneStillWorks() -> ()
{
  assert_eq!(
    facets("a~ := \"text\"\na = None", &["stype", "value", "mut"]),
    "stype=String, value=, mut=Variable"
  );
}

/// `a := None` удаляет структуру.
#[test]
fn deletingStructureStillWorks() -> ()
{
  assert!(facets("a := 5\na := None", ALL).starts_with("error: Structure 'a' was not created"));
}

/// `None(5)` как приведение по-прежнему даёт пустое значение.
#[test]
fn castToNoneStillWorks() -> ()
{
  assert_eq!(facets("a := None(5)", &["type", "value"]), "type=None, value=");
}

/// Структура без значения и без типа по-прежнему ничего не хранит.
#[test]
fn unknownReferenceStillGivesNone() -> ()
{
  assert_eq!(facets("a := b", &["type", "stype", "value"]), "type=None, stype=None, value=");
}

// =================================================================================================
