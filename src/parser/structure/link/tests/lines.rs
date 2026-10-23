use crate::parser::structure::link::tests::{checkEmpty, checkToken, childStructure, emptyMain, lineOf, linkOf, linkOfLine, linkOfRootLine};
use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock};
// =================================================================================================

// Ссылка по номеру линии: `a[0]` - это `lines[0]` структуры.
//
// Номер относится только к той структуре, в которой ссылка начата.

// =================================================================================================

/// Чтение линии со значением.
#[test]
fn readLine() -> ()
{
  let result: Token = linkOfLine("a := 10", "a", 0, None);
  checkToken(&result, &TokenType::UInt, "10");
}

/// Номер линии за пределами линий структуры - пусто.
#[test]
fn lineOutOfBounds() -> ()
{
  let result: Token = linkOfLine("a := 10", "a", 5, None);
  checkEmpty(&result);
}

/// Числовая голова без структуры: линию читать не из чего - пусто.
#[test]
fn lineNumberWithoutStructure() -> ()
{
  let result: Token = linkOf("a := 10", "0", None);
  checkEmpty(&result);
}

/// Динамическая голова `[выражение]` вычисляется в номер линии: `a.[0]`.
#[test]
fn dynamicHeadExpression() -> ()
{
  let result: Token = linkOf("a := 10", "a.[0]", None);
  checkToken(&result, &TokenType::UInt, "10");
}

/// Продолжение после номера линии: `a.0.b` - имя из линии ищется среди детей `a`.
///
/// Дерево собирается руками: объявление `b` внутри `a { }` из кода структурой
/// не создаётся, пока тело блока не исполнится.
#[test]
fn lineLinkContinuation() -> ()
{
  let main: Arc<RwLock<Structure>> = emptyMain();
  let a: Arc<RwLock<Structure>> = childStructure(
    "a",
    StructureType::Primitive(PrimitiveType::None),
    vec![lineOf(vec![Token::new(TokenType::Word, "b")])],
    &main
  );
  childStructure(
    "b",
    StructureType::Primitive(PrimitiveType::U8),
    vec![lineOf(vec![Token::new(TokenType::UInt, "5")])],
    &a
  );

  let mut segments: Vec<String> = Structure::parseLink("a.0.b");
  let result: Token = main.read().unwrap().linkExpression(None, &mut segments, None);
  checkToken(&result, &TokenType::UInt, "5");
}

/// Продолжение после номера линии, когда имени нет среди детей: `a.0.b` ищется выше, в self.
#[test]
fn lineLinkContinuationFromSelf() -> ()
{
  let result: Token = linkOf("b := 5\na {\n  b\n}\n", "a.0.b", None);
  checkToken(&result, &TokenType::UInt, "5");
}

/// Линия со словом: если такого слова нет среди детей - остаётся ссылка.
#[test]
fn readWordLineIsLink() -> ()
{
  let result: Token = linkOfLine("a {\n  b\n}\n", "a", 0, None);
  checkToken(&result, &TokenType::Link, "b");
}

/// Линия со словом, и такое слово есть среди детей - читается значение ребёнка.
#[test]
fn readWordLineThroughChild() -> ()
{
  let result: Token = linkOfRootLine("a := 10", 0, None);
  checkToken(&result, &TokenType::UInt, "10");
}

/// Тело метода - это тоже линия: читается как есть, без вызова.
#[test]
fn readMethodBodyLine() -> ()
{
  let result: Token = linkOfLine("hello() {\n  := 42\n}\n", "hello", 0, None);
  checkToken(&result, &TokenType::Creation, "");
}

/// Вызов по номеру линии: `a[0]()` зовёт метод, имя которого стоит в этой линии.
#[test]
fn callByLineNumber() -> ()
{
  let result: Token = linkOfLine("b {\n  hello() { := 42 }\n}\n", "b", 0, Some(vec![]));
  checkToken(&result, &TokenType::Word, "hello");
}

/// Вызов по номеру линии аргументы не прокидывает: они теряются.
///
/// Числовая ссылка собирает вызов иначе, чем ссылка по имени: в неё
/// аргументы не попадают вовсе. Поведение зафиксировано тестом,
/// чтобы изменение было осознанным.
#[test]
fn callByLineNumberKeepsNoArguments() -> ()
{
  let result: Token = linkOfLine(
    "b {\n  add(x: UInt) { := x + 1 }\n}\n",
    "b",
    0,
    Some(vec![Token::new(TokenType::UInt, "5")])
  );
  checkToken(&result, &TokenType::Word, "add");
}

// =================================================================================================
