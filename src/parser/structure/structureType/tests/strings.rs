use crate::parser::parser::runCode;
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

/// Строка уходит в нативную функцию: и целиком, и по полям.
///
/// `lib.print(str.pointer, str.length)` — ручная распаковка полей.
/// `lib.print(str)` — целиком, chillffi сам раскладывает String на ptr+len.
///
/// C печатает прямо в stdout, поэтому тут проверяется, что оба вызова прошли
/// и код дошёл до конца — через `done := True` после `printc`.
#[test]
fn native() -> ()
{
  // todo libPath тут пишет что unused, но по факту он юзается ниже в макросе.
  let libPath: String = buildLibrary!(
    r#"
    #include <stdio.h>
    #include <stdint.h>
    #include <stddef.h>

    uint8_t* print(const uint8_t* data, size_t length) {
      fwrite(data, 1, length, stdout);
      fflush(stdout);
      return NULL;
    }
    "#,
    "libprint.so"
  );

  let path: String = writeCodeFile!(
    r#"
    printc(str: String) {
      lib: Pointer = importNative("{libPath}")
      lib.print(str.pointer, str.length)
      lib.print(str)
    }

    printc("Hello from RTS 222!")
    done := True
    "#,
    "stringsMain.rt"
  );

  let main: Arc<RwLock<Structure>> = runCode(&format!("m := import(\"{path}\")\n"));

  checkStructure!(main, "m.printc", stype StructureType::Method);
  checkStructure!(main, "m.done", value "True");

  let _ = std::fs::remove_file(std::env::temp_dir().join("stringsMain.rt"));
  let _ = std::fs::remove_file(std::env::temp_dir().join("libprint.so"));
}

// =================================================================================================
