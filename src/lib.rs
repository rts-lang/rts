#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

include!("prelude.rs");

// =================================================================================================

#[cfg(not(feature = "analyzer"))]
use std::sync::{Arc, RwLock, RwLockWriteGuard};
#[cfg(not(feature = "analyzer"))]
use crate::parser::bytes::Bytes;
#[cfg(not(feature = "analyzer"))]
use crate::parser::parser::{parseLines, MainStructure};
#[cfg(not(feature = "analyzer"))]
use crate::parser::structure::structure::{Structure, StructureMut};
#[cfg(not(feature = "analyzer"))]
use crate::parser::structure::structureType::StructureType;
#[cfg(not(feature = "analyzer"))]
use crate::tokenizer::tokenizer::readTokensSimple;
#[cfg(not(feature = "analyzer"))]
use crate::tokenizer::types::line::Line;
#[cfg(not(feature = "analyzer"))]
use crate::tokenizer::types::token::Token;

// todo удалить mods
pub mod tokenizer;
pub mod parser;
#[cfg(all(not(target_family = "wasm"), not(feature = "analyzer")))]
mod logger;
#[cfg(feature = "analyzer")]
mod analyzer;

// =================================================================================================

/// Основная структура-прокладка для создания оболочки между RTS-lib и другим проектом;
#[cfg(not(feature = "analyzer"))]
pub struct RTS 
{
  namespace: String
}

#[cfg(not(feature = "analyzer"))]
impl RTS 
{
  /// Создаёт namespace структуру и RTS оболочку
  pub fn new(name: String) -> Self 
  {
    // 
    {
      let main: RwLockWriteGuard<Structure> = MainStructure.write().unwrap();
      main.pushStructure(
        Arc::new(RwLock::new(Structure::new(
          Some(name.clone()),
          StructureMut::Constant,
          StructureType::List,
          // В линии структуры
          Some(vec![
            Arc::new(RwLock::new(
              Line
              {
                tokens: None,
                lines: None
              }
            ))
          ]),
          Some( MainStructure.clone() ) // Ссылаемся на родителя
        )))
      );
    }
    
    //
    Self {
      namespace: name
    }
  }
  
  /// Добавляет структуру в namespace структуру
  pub fn newStructure(
    &self, 
    structureName: String, 
    structureMut: StructureMut, 
    structureType: StructureType, 
    structureTokens: Vec<Token>
  ) 
  {
    //
    let namespaceStructureLink: Arc<RwLock<Structure>> = {
      let mainStructure: RwLockWriteGuard<Structure> = MainStructure.write().unwrap();
      mainStructure.getStructureByScope(self.namespace.as_str()).unwrap()
    };
    let namespaceStructure: RwLockWriteGuard<Structure> = namespaceStructureLink.write().unwrap();
    namespaceStructure.pushStructure(
      Arc::new(RwLock::new(Structure::new(
        Some(structureName),
        structureMut,
        structureType,
        // В линии структуры
        Some(vec![
          Arc::new(RwLock::new(
            Line
            {
              tokens: Some(structureTokens),
              lines: None
            }
          ))
        ]),
        Some( namespaceStructureLink.clone() ) // Ссылаемся на родителя
      )))
    );
  }

  /// Запускает код
  pub fn run(&self, script: &str) 
  {
    let mut buffer: Vec<u8> = script.as_bytes().to_vec();
    parseLines( readTokensSimple(&mut buffer) );
  }
  
  pub fn getNative(method: extern "C" fn(args: &[Token])) -> Bytes 
  {
    let ptr: *const () = method as *const ();
    let rawBytes: Vec<u8> = (ptr as usize).to_le_bytes().to_vec();
    //println!("raw bytes ({}): {:?}", rawBytes.len(), rawBytes);
    Bytes::new(rawBytes)
  }
}
