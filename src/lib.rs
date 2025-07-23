// /lib

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

include!("prelude.rs");
// =================================================================================================

use std::sync::{Arc, RwLock, RwLockWriteGuard};
use crate::line::Line;
use crate::structure::{Structure, StructureMut, StructureType};
use crate::token::{Token};

pub struct RTS {
  namespace: String,
}

impl RTS {
  /// Создаёт namespace структуру и RTS оболочку
  pub fn new(name: String) -> Self {
    // 
    let mut main: RwLockWriteGuard<'_, Structure> = _main.write().unwrap();
    main.pushStructure(
      Structure::new(
        Some(name.clone()),
        StructureMut::Constant,
        StructureType::List,
        // В линии структуры
        Some(vec![
          Arc::new(RwLock::new(
            Line
            {
              tokens: None,
              indent: None,
              lines:  None,
              parent: None
            }
          ))
        ]),
        Some( _main.clone() ), // Ссылаемся на родителя
      )
    );
    
    //
    RTS {
      namespace: name
    }
  }
  
  /// Добавляет структуру в namespace структуру
  pub fn newStructure(&self, structureName: String, structureMut: StructureMut, structureType: StructureType, structureTokens: Vec<Token>) {
    //
    let mainStructure: RwLockWriteGuard<'_, Structure> = _main.write().unwrap();
    let namespaceStructureLink: Arc<RwLock<Structure>> = mainStructure.getStructureByName(self.namespace.as_str()).unwrap();
    let mut namespaceStructure: RwLockWriteGuard<'_, Structure> = namespaceStructureLink.write().unwrap();
    namespaceStructure.pushStructure(
      Structure::new(
        Some(structureName),
        structureMut,
        structureType,
        // В линии структуры
        Some(vec![
          Arc::new(RwLock::new(
            Line
            {
              tokens: Some(structureTokens),
              indent: None,
              lines:  None,
              parent: None
            }
          ))
        ]),
        Some( namespaceStructureLink.clone() ), // Ссылаемся на родителя
      )
    );
  }

  /// Запускает код
  pub fn run(&self, script: &str) {
    let buffer: Vec<u8> = script.as_bytes().to_vec();
    parseLines( readTokens(buffer, unsafe{_debugMode}) );
  }
}

//type Method = Arc<dyn Fn(Vec<Structure>) -> Structure + Send + Sync>;

/* пример испрользования
  let rts: RTS = RTS::new(String::from("terravox"));

  rts.newStructure(
    String::from("test"),
    StructureMut::Constant,
    StructureType::String,
    vec![
      Token::new(
        TokenType::String,
        Bytes::new(String::from("mod var"))
      )
    ]
  );
  rts.run("println(terravox.test);");
*/