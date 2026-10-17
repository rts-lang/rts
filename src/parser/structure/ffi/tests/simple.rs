use crate::parser::parser::runCodeFile;
use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::StructureType;
use crate::parser::testing::{buildLibrary, checkStructure, writeCodeFile};
use std::sync::{Arc, RwLock};
// =================================================================================================

// todo desc

// =================================================================================================

/// `importNative` + `lib.strlen(data, len)` — возвращает переданную длину.
#[test]
fn simple() -> ()
{
  let libraryPath: String = buildLibrary!(
    r#"
    #include <stdint.h>
    #include <stddef.h>

    size_t strlen(const uint8_t* data, size_t length) {
      (void)data;
      return length;
    }
    "#
  );

  let path: String = writeCodeFile!(
    format!(
      r#"
      strlen(pointer: Pointer; length: Usize) {{
        library: Pointer = importNative("{libraryPath}")
        := library.strlen(pointer; length)
      }}

      length: Usize = strlen("Hello from RTS 222!"; 19)
      "#
    )
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "strlen", stype StructureType::Method);
  checkStructure!(main, "length", stype StructureType::Usize, value "19");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

// =================================================================================================
