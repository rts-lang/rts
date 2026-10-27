use crate::parser::structure::link::tests::{checkEmpty, checkToken, childStructure, emptyMain, lineOf, linkOf, linkOfIn};
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock};
// =================================================================================================

// Тесты на ссылки по имени.
// 
// `a`, `a.b` - это путь по структурам.
//
// Голова ссылки ищется вверх по области видимости, остаток пути - строго вниз.

// =================================================================================================

/// Чтение значения: у структуры одна линия - отдаём её значение.
#[test]
fn readValue() -> ()
{
  let result: Token = linkOf("a := 10", "a", None);
  checkToken(&result, &TokenType::UInt, "10");
}

/// Чтение структуры из нескольких линий - остаётся ссылка, а не значение.
#[test]
fn readMultiLineStructureIsLink() -> ()
{
  let result: Token = linkOf("a {\n  10\n  20\n}\n", "a", None);
  checkToken(&result, &TokenType::Link, "a");
}

/// Вызов метода с телом: `hello()` возвращает значение.
#[test]
fn callMethod() -> ()
{
  let result: Token = linkOf("hello() {\n  := 42\n}\n", "hello", Some(vec![]));
  checkToken(&result, &TokenType::UInt, "42");
}

/// Вызов однострочного метода: `hello() { := 42 }`.
///
/// Раньше тело из одной строки попадало в ветку чтения и считалось
/// в области вызывающего - значение терялось.
#[test]
fn callOneLineMethod() -> ()
{
  let result: Token = linkOf("hello() { := 42 }\n", "hello", Some(vec![]));
  checkToken(&result, &TokenType::UInt, "42");
}

/// Вызов метода с аргументом: аргумент доходит до тела метода.
#[test]
fn callMethodWithArgument() -> ()
{
  let result: Token = linkOf(
    "add(x: UInt) {\n  := x + 1\n}\n",
    "add",
    Some(vec![Token::new(TokenType::UInt, "5")])
  );
  checkToken(&result, &TokenType::UInt, "6");
}

/// Несколько аргументов склеиваются в одну линию - используется только первый.
///
/// Так устроен вызов по ссылке: аргументы приходят уже вычисленными токенами
/// и кладутся в одну линию, а тело метода ждёт отдельную линию на каждый аргумент.
/// Обычный вызов (`add(2, 3)` в коде) собирает линии правильно - расходится только
/// вызов по ссылке. Поведение зафиксировано тестом, чтобы его изменение
/// было осознанным.
#[test]
fn callMethodWithSeveralArgumentsKeepsFirst() -> ()
{
  let result: Token = linkOf(
    "add(x: UInt, y: UInt) {\n  := x + y\n}\n",
    "add",
    Some(vec![
      Token::new(TokenType::UInt, "2"),
      Token::new(TokenType::UInt, "3")
    ])
  );
  checkToken(&result, &TokenType::UInt, "2");
}

/// Вызов значения, а не метода: значения вызвать нельзя - пусто.
#[test]
fn callValueIsNotMethod() -> ()
{
  let result: Token = linkOf("a := 10", "a", Some(vec![]));
  checkEmpty(&result);
}

/// Нет такой структуры - пусто.
#[test]
fn notFound() -> ()
{
  let result: Token = linkOf("a := 10", "nope", None);
  checkEmpty(&result);
}

/// Цепочка из нескольких имён: `a.b.c` читается сквозь вложенные структуры.
///
/// Дерево собирается руками: вложенные объявления из кода структурами
/// не создаются, пока тело блока не исполнится. Промежуточные структуры
/// обязаны иметь тело - продолжение ищется только через структуры с линиями.
#[test]
fn multiSegmentChain() -> ()
{
  let main: Arc<RwLock<Structure>> = emptyMain();
  let a: Arc<RwLock<Structure>> =
    childStructure("a", StructureType::Primitive(PrimitiveType::None), vec![lineOf(vec![])], &main);
  let b: Arc<RwLock<Structure>> =
    childStructure("b", StructureType::Primitive(PrimitiveType::None), vec![lineOf(vec![])], &a);
  childStructure(
    "c",
    StructureType::Primitive(PrimitiveType::U8),
    vec![lineOf(vec![Token::new(TokenType::UInt, "7")])],
    &b
  );

  let mut segments: Vec<String> = Structure::parseLink("a.b.c");
  let result: Token = main.read().unwrap().linkExpression(None, &mut segments, None);
  checkToken(&result, &TokenType::UInt, "7");
}

/// Вызов метода через цепочку: `a.b.f()` исполняется в области объявления.
#[test]
fn multiSegmentChainCall() -> ()
{
  let main: Arc<RwLock<Structure>> = emptyMain();
  let a: Arc<RwLock<Structure>> =
    childStructure("a", StructureType::Primitive(PrimitiveType::None), vec![lineOf(vec![])], &main);
  let b: Arc<RwLock<Structure>> =
    childStructure("b", StructureType::Primitive(PrimitiveType::None), vec![lineOf(vec![])], &a);
  childStructure(
    "f",
    StructureType::Primitive(PrimitiveType::Method),
    vec![lineOf(vec![Token::new(TokenType::Creation, ""), Token::new(TokenType::UInt, "42")])],
    &b
  );

  let mut segments: Vec<String> = Structure::parseLink("a.b.f");
  let result: Token = main.read().unwrap().linkExpression(None, &mut segments, Some(vec![]));
  checkToken(&result, &TokenType::UInt, "42");
}

/// Разрыв цепочки: среднего звена нет - пусто, а не чтение того, что нашлось.
#[test]
fn multiSegmentChainBreak() -> ()
{
  let main: Arc<RwLock<Structure>> = emptyMain();
  let a: Arc<RwLock<Structure>> =
    childStructure("a", StructureType::Primitive(PrimitiveType::None), vec![lineOf(vec![])], &main);
  childStructure(
    "c",
    StructureType::Primitive(PrimitiveType::U8),
    vec![lineOf(vec![Token::new(TokenType::UInt, "7")])],
    &a
  );

  let mut segments: Vec<String> = Structure::parseLink("a.b.c");
  let result: Token = main.read().unwrap().linkExpression(None, &mut segments, None);
  checkEmpty(&result);
}

/// Голова ссылки ищется вверх: из вложенной структуры видно имя уровнем выше.
#[test]
fn scopeUp() -> ()
{
  let result: Token = linkOfIn("a := 10\nb {\n  := a\n}\n", "b", "a", None);
  checkToken(&result, &TokenType::UInt, "10");
}

/// Чтение метода без скобок вызова - это не значение, а тело метода.
///
/// Отдельно оговорённый случай: чтение и вызов различаются только по скобкам.
#[test]
fn readMethodWithoutCall() -> ()
{
  let result: Token = linkOf("hello() {\n  := 42\n}\n", "hello", None);
  checkToken(&result, &TokenType::Creation, "");
}

/// Вызов метода, у которого нет родителя: выполнять тело негде - пусто.
///
/// Метод лежит среди детей структуры, но родителем у него не выбран никто -
/// так и ведут себя структуры, созданные вне обычного разбора кода.
#[test]
fn callMethodWithoutParent() -> ()
{
  let main: Arc<RwLock<Structure>> = emptyMain();
  let method: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
    Some(String::from("hello")),
    StructureMut::Constant,
    StructureType::Primitive(PrimitiveType::Method),
    Some(vec![Arc::new(RwLock::new(Line {
      tokens: Some(vec![Token::new(TokenType::Creation, "")]),
      lines: None
    }))]),
    None
  )));
  main.read().unwrap().pushStructure(method);

  let mut segments: Vec<String> = vec![String::from("hello")];
  let result: Token = main.read().unwrap().linkExpression(None, &mut segments, Some(vec![]));
  checkEmpty(&result);
}

/// У структуры нет линий - читать нечего.
#[test]
fn readStructureWithoutLines() -> ()
{
  let main: Arc<RwLock<Structure>> = emptyMain();
  let empty: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
    Some(String::from("a")),
    StructureMut::Constant,
    StructureType::Primitive(PrimitiveType::None),
    None,
    Some(main.clone())
  )));
  main.read().unwrap().pushStructure(empty);

  let mut segments: Vec<String> = vec![String::from("a")];
  let result: Token = main.read().unwrap().linkExpression(None, &mut segments, None);
  checkEmpty(&result);
}

// =================================================================================================
