use crate::parser::parser::runCode;
use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::StructureType;
use crate::parser::testing::{buildLib, checkStructure};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
// =================================================================================================

// Это тесты на три вида строк: String, CString и RawString.
//
// Аннотация типа сама решает каст.

// =================================================================================================

/// Пустая строка в каждом из трёх типов строк.
///
/// todo type() здесь None, а не String: с явным типом пустая строка теряет тип токена.
/// Хотя структура получает правильный тип. Без явного типа (`a := ""`) токен остаётся String.
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

/// Строка уходит в нативную функцию (release/native/strings): и целиком, и по полям.
///
/// `lib.print(str.pointer, str.length)` — ручная распаковка полей.
/// `lib.print(str)` — целиком, chillffi сам раскладывает String на ptr+len.
///
/// C печатает прямо в stdout, поэтому тут проверяется, что оба вызова прошли
/// и код дошёл до конца — через `done := True` после `printc`.
#[test]
fn native() -> ()
{
  let lib: String = buildLib("release/native/strings/print.c", "libstringsPrint.so");
  let path: PathBuf = std::env::temp_dir().join("stringsMain.rt");
  std::fs::write(&path, format!(
    r#"printc(str: String) {{
  lib: Pointer = importNative("{lib}")
  lib.print(str.pointer, str.length)
  lib.print(str)
}}

printc("Hello from RTS 222!")
done := True"#
  )).unwrap();
  let pathStr: String = path.to_string_lossy().replace('\\', "/");
  let main: Arc<RwLock<Structure>> = runCode(&format!("m := import(\"{pathStr}\")\n"));

  checkStructure!(main, "m.printc", stype StructureType::Method);
  checkStructure!(main, "m.done", value "True");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&lib);
}

// =================================================================================================
