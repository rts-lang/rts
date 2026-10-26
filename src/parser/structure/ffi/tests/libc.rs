use crate::parser::parser::runCodeFile;
use crate::parser::structure::structure::Structure;
use crate::parser::testing::{checkStructure, writeCodeFile};
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock};
// =================================================================================================

// Тесты на libc.
//
//  Платформозависимые методы больше не встроены в язык, их место - native-пакеты.
//
//  Самый простой пакет - обёртка над `libc`:

// =================================================================================================

/// Число из rand().
#[test]
fn rand() -> ()
{
  let path: String = writeCodeFile!(
    r#"
    libc: Pointer = importNative("libc.so.6")

    a := libc.rand()
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "a", type TokenType::UInt, lines 1);

  let _ = std::fs::remove_file(&path);
}

/// `sleep` 1 ms.
#[test]
fn sleep() -> ()
{
  let path: String = writeCodeFile!(
    r#"
    libc: Pointer = importNative("libc.so.6")

    libc.usleep(1000)
    slept: Usize = 1
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "slept", stype Usize, value "1");

  let _ = std::fs::remove_file(&path);
}

// =================================================================================================
