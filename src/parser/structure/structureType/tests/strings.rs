use crate::parser::parser::{runCode, runCodeFile};
use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::StructureType;
use crate::parser::testing::{buildLibrary, checkStructure, writeCodeFile};
use std::sync::{Arc, RwLock};
// =================================================================================================

// Это общие тесты на три вида строк: String, CString и RawString.

// =================================================================================================

/// Пустая строка в каждом из трёх типов строк.
///
/// todo type() здесь None, а не String: с явным типом пустая строка теряет тип токена.
///  Хотя структура получает правильный тип. Без явного типа (`a := ""`) токен остаётся String.
#[test]
fn types() -> ()
{
  checkStructure!(
    r#"a: String = """#,
    "a",
    stype StructureType::String,
    value ""
  );
  // CString = Pointer (алиас).
  checkStructure!(
    r#"b: CString = """#,
    "b",
    stype StructureType::Pointer,
    value ""
  );
  checkStructure!(
    r#"c: RawString = """#,
    "c",
    stype StructureType::RawString,
    value ""
  );
}

// =================================================================================================

/// Строка уходит в native: и целиком, и через поля.
///
/// `lib.strlen(str.pointer, str.length)` — ручная распаковка полей.
/// `lib.strlen(str)` — целиком, `chillffi` сам раскладывает String на ptr+len.
#[test]
fn native() -> ()
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
    r#"
    strlen(string: String) {{
      library: Pointer = importNative("{libraryPath}")
      := library.strlen(string.pointer; string.length) + library.strlen(string)
    }}

    string: String = "Hello from RTS 222!"
    result: Usize = strlen(string)
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "strlen", stype StructureType::Method);
  checkStructure!(main, "string", stype StructureType::String, value "Hello from RTS 222!");
  checkStructure!(main, "result", stype StructureType::Usize, value "38");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

// =================================================================================================
