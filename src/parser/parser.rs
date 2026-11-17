use super::bytes::Bytes;
use super::readStructureLines;
use super::structure::primitives::{ArgsPrimitives, MethodsPrimitives};
use super::structure::structure::{Structure, StructureMut};
use super::structure::structureType::{PrimitiveType, StructureType};
use crate::tokenizer::tokenizer::readTokensSimple;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use crate::{_argc, _argv, _sourcePath};
use std::sync::{Arc, RwLock, RwLockWriteGuard};
// =================================================================================================

// Здесь основная часть `parser`. Основная задача в том, чтобы создать `main`
// `Structure` и предоставить основные методы запуска последовательностей токенов.
//
// Основная структура - это буквально вход в саму программу.
// Также это начальная точка при `import`. Т.е. это самая главная структура,
// хранящая все остальные структуры внутри себя и позволяет запускать себя и их.

// =================================================================================================

/// Создает основную структуру программы.
pub fn createMainStructure(tokenizerLinesLinks: Vec< Arc<RwLock<Line>> >) -> Arc<RwLock<Structure>>
{
  // -----------------------------------------------------------------------------------------------

  // Основная структура; В неё вкладываются остальные;
  //
  // В эту структуру будут переданы стартовые параметры;
  //
  // Неизменяемая; Действует во время всей жизни программы.
  let mainStructureLink: Arc<RwLock<Structure>> =
    Arc::new(RwLock::new(Structure::new(
      None,
      StructureMut::Constant,
      StructureType::Primitive(PrimitiveType::Method),
      None,
      None
    )));
  let mut mainStructure: RwLockWriteGuard<Structure> = mainStructureLink.write().unwrap();

  // Присваиваем линии от Tokenizer; Это то что будет запущено как основная часть.
  mainStructure.lines = Some(tokenizerLinesLinks);

  // -----------------------------------------------------------------------------------------------
  // Параметры запуска (`argc`, `argv`).
  //
  // Значения приходят от запуска, поэтому у них, в отличие от остальных примитивов, есть линии.

  for (name, dataType) in ArgsPrimitives
  {
    let lines: Vec< Arc<RwLock<Line>> > = match *name
    {
      // Одна линия с одним токеном - сколько аргументов.
      "argc" => vec![
        Arc::new(RwLock::new(Line
        {
          tokens: Some(vec![
            Token::new(
              TokenType::UInt,
              Bytes::new(_argc.get().copied().unwrap_or(0).to_string())
            )
          ]),
          lines: None
        }))
      ],

      // По линии на каждый аргумент.
      "argv" => _argv.get().map(|values| values.as_slice()).unwrap_or(&[])
        .iter()
        .map(|arg: &String| Arc::new(RwLock::new(Line
        {
          tokens: Some(vec![
            Token::new(
              TokenType::String,
              Bytes::new( String::from(arg) )
            )
          ]),
          lines: None
        })))
        .collect(),

      _ => Vec::new()
    };

    mainStructure.pushStructure(
      Arc::new(RwLock::new(Structure::new(
        Some(String::from(*name)),
        StructureMut::Constant, // Неизменяемая.
        StructureType::Primitive(dataType.clone()),
        Some(lines),
        Some( mainStructureLink.clone() ) // Ссылаемся на родителя.
      )))
    );
  }

  // -----------------------------------------------------------------------------------------------
  // Примитивы (базовые структуры языка).

  for (name, dataType) in PrimitiveType::iter()
  {
    mainStructure.pushStructure(
      Arc::new(RwLock::new(Structure::new(
        Some(String::from(name)),
        StructureMut::Constant, // Неизменяемая.
        StructureType::Primitive(dataType.clone()),
        None, // Без значения — просто маркер.
        Some( mainStructureLink.clone() ) // Ссылаемся на родителя.
      )))
    );
  }

  for name in MethodsPrimitives
  {
    mainStructure.pushStructure(
      Arc::new(RwLock::new(Structure::new(
        Some(String::from(*name)),
        StructureMut::Constant, // Неизменяемая.
        StructureType::Primitive(PrimitiveType::Method),
        None, // Без значения — просто маркер.
        Some( mainStructureLink.clone() ) // Ссылаемся на родителя.
      )))
    );
  }
  
  // -----------------------------------------------------------------------------------------------
  drop(mainStructure);
  mainStructureLink
}

// =================================================================================================

/// Это основная функция для парсинга строк;
/// 
/// Она разделена на подготовительную часть, и часть запуска readLine().
fn parseLines(tokenizerLinesLinks: Vec< Arc<RwLock<Line>> >) -> Arc<RwLock<Structure>>
{ 
  // Подготовка к запуску.
  let mainStructureLink: Arc<RwLock<Structure>> = createMainStructure(tokenizerLinesLinks);

  // Подготовка закончена, читаем линии;
  //
  // Передаём ссылку на структуру и запускаем.
  readStructureLines(mainStructureLink.clone());
  
  //
  mainStructureLink
}

// =================================================================================================

// todo Я вынес `runCode!` за место метода, который был тут;
//  И вот не понятно должны ли эти методы тоже быть в `testing`, хотя тот же `runCode!`
//  должен ли быт там? Логично что это часть основной инфраструктуры `parser`, поэтому оно
//  пока что так разделено. Это надо решить, возможно оно так и должно быть.

/// Запускает код и возвращает основную структуру.
pub fn runCodeBytes(buffer: &mut Vec<u8>) -> Arc<RwLock<Structure>>
{
  parseLines( readTokensSimple(buffer) )
}

/// Запускает `.rt` файл и возвращает основную структуру.
///
/// В отличие от обёртки `runCode!(r#"m := import("…")"#)` не опирается на
/// `import` — файл читается и выполняется напрямую, как entry в CLI.
/// 
/// todo Тут есть большие вопросы по стилю кода.
pub fn runCodeFile(path: &str) -> Arc<RwLock<Structure>>
{
  assert!(
    path.ends_with(".rt"),
    "runCodeFile: expected a `.rt` file, got `{path}`"
  );
  
  let mut buffer: Vec<u8> = std::fs::read(path)
    .unwrap_or_else(|error| panic!("runCodeFile: read `{path}`: {error}"));
  
  //
  let absolutePath: String = std::fs::canonicalize(path)
    
    .unwrap_or_else(|error| panic!("runCodeFile: canonicalize `{path}`: {error}"))
    .to_string_lossy().replace('\\', "/");
  let previousPath: Option<Arc<String>> =
    _sourcePath.with(|sourcePath| sourcePath.borrow().clone());
  
  _sourcePath.with(|sourcePath|
    *sourcePath.borrow_mut() = Some(Arc::new(absolutePath))
  );
  
  //
  let result: Arc<RwLock<Structure>> = runCodeBytes(&mut buffer);
  
  _sourcePath.with(|sourcePath|
    *sourcePath.borrow_mut() = previousPath
  );
  
  result
}

// =================================================================================================
