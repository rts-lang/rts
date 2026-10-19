use crate::parser::structure::link::tests::{checkToken, emptyMain, pointerStructure};
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::StructureType;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock};
// =================================================================================================

// Динамическая библиотека: `lib.method`.

// =================================================================================================

/// Ссылка на метод библиотеки даёт токен с путём к библиотеке и именем метода.
#[test]
fn nativeCallToken() -> ()
{
  let main: Arc<RwLock<Structure>> = emptyMain();
  let lib: Arc<RwLock<Structure>> = pointerStructure("lib", "./libhello.so", &main);
  main.read().unwrap().pushStructure(lib);

  let mut segments: Vec<String> = Structure::parseLink("lib.hello");
  let result: Token = main.read().unwrap().linkExpression(None, &mut segments, None);

  // Токен-вложение: одна линия с двумя строками - путь и имя метода.
  let lines: Vec< Arc<RwLock<Line>> > = result.lines.clone().unwrap_or_default();
  assert!(lines.len() == 1, "expected one nesting line, got {}", lines.len());
  let tokens: Vec<Token> = lines[0].read().unwrap().tokens.clone().unwrap_or_default();
  assert!(tokens.len() == 2, "expected library path and method name, got {}", tokens.len());
  checkToken(&tokens[0], &TokenType::String, "./libhello.so");
  checkToken(&tokens[1], &TokenType::String, "hello");
}

/// Битая библиотека: в первой линии не строка с путём - ссылка пустая.
#[test]
fn nativeCallWithoutLibraryPath() -> ()
{
  let main: Arc<RwLock<Structure>> = emptyMain();
  let lib: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
    Some(String::from("lib")),
    StructureMut::Constant,
    StructureType::Pointer,
    Some(vec![Arc::new(RwLock::new(Line {
      tokens: Some(vec![Token::new(TokenType::Word, "notAPath")]),
      lines: None
    }))]),
    Some(main.clone())
  )));
  main.read().unwrap().pushStructure(lib);

  let mut segments: Vec<String> = Structure::parseLink("lib.hello");
  let result: Token = main.read().unwrap().linkExpression(None, &mut segments, None);

  let lines: Vec< Arc<RwLock<Line>> > = result.lines.clone().unwrap_or_default();
  assert!(lines.is_empty(), "expected no nesting lines");
  checkToken(&result, &TokenType::None, "");
}

// =================================================================================================
