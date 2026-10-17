use crate::parser::parser::runCodeFile;
use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::StructureType;
use crate::parser::testing::{checkStructure, createDirectory, relativePath, writeCodeFile};
use std::sync::{Arc, RwLock};
// =================================================================================================

// Тесты на `import()`: путь считается от файла, в котором он написан.

// =================================================================================================

/// `import("./N.rt")` — простой импорт одного файла.
#[test]
fn simple() -> ()
{
  let directory: String = createDirectory!();

  let testPath: String = writeCodeFile!(
    &directory,
    r#"
    hello() {
      := 42
    }

    value: Usize = 4
    "#
  );

  let mainPath: String = writeCodeFile!(
    &directory,
    format!(
      r#"
      a := import("{}")
      "#,
      relativePath!(&testPath)
    )
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&mainPath);

  checkStructure!(main, "a.hello", stype StructureType::Method);
  checkStructure!(main, "a.value", stype StructureType::Usize, value "4");

  let _ = std::fs::remove_file(&testPath);
  let _ = std::fs::remove_file(&mainPath);
  let _ = std::fs::remove_dir(&directory);
}

// =================================================================================================

/// Вложенный `import()`; Путь считается от файла, где он написан, а не от main.
///
/// todo: `bHello := b.hello()` возвращает пустую строку вместо `"b: hello"`.
///  В `:=` return при вызове метода на модуле — модуль имеет `name = None`,
///  и `:=` не находит именную структуру для записи result.
#[test]
fn nested() -> ()
{
  let directory: String = createDirectory!();

  // Структура:
  //   dir/
  //     main         — Импортирует `lib/a` и `lib/deep/c`
  //     lib/
  //       a          — Импортирует `b`, вызывает `b.hello()`
  //       b          — `hello() { := "b: hello" }`
  //       deep/
  //         c        — Импортирует `d`, вызывает `d.hello()`
  //         d        — `hello() { := "d: hello" }`

  let libraryDirectory: String = format!("{directory}/lib");
  let deepDirectory: String = format!("{libraryDirectory}/deep");
  std::fs::create_dir_all(&libraryDirectory).unwrap();
  std::fs::create_dir_all(&deepDirectory).unwrap();

  let bPath: String = writeCodeFile!(
    &libraryDirectory,
    r#"
    hello() {
      := "b: hello"
    }
    "#
  );

  let dPath: String = writeCodeFile!(
    &deepDirectory,
    r#"
    hello() {
      := "d: hello"
    }
    "#
  );

  let cPath: String = writeCodeFile!(
    &deepDirectory,
    format!(
      r#"
      d := import("{}")
      dHello := d.hello()

      hello() {{
        := "c: hello"
      }}
      "#,
      relativePath!(&dPath)
    )
  );

  let aPath: String = writeCodeFile!(
    &libraryDirectory,
    format!(
      r#"
      b := import("{}")
      bHello := b.hello()

      hello() {{
        := "a: hello"
      }}

      load() {{
        c := import("{}")
        := c.hello()
      }}
      "#,
      relativePath!(&bPath),
      relativePath!(&cPath, "deep")
    )
  );

  let mainPath: String = writeCodeFile!(
    &directory,
    format!(
      r#"
      a := import("{}")
      c := import("{}")
      "#,
      relativePath!(&aPath, "lib"),
      relativePath!(&cPath, "lib/deep")
    )
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&mainPath);

  // `a.rt` импортировал `b.rt`.
  checkStructure!(main, "a.bHello", value "b: hello");
  checkStructure!(main, "a.hello", stype StructureType::Method);

  // `c.rt` импортировал `d.rt`.
  checkStructure!(main, "c.dHello", value "d: hello");
  checkStructure!(main, "c.hello", stype StructureType::Method);

  let _ = std::fs::remove_file(&bPath);
  let _ = std::fs::remove_file(&aPath);
  let _ = std::fs::remove_file(&dPath);
  let _ = std::fs::remove_file(&cPath);
  let _ = std::fs::remove_file(&mainPath);
  let _ = std::fs::remove_dir(&deepDirectory);
  let _ = std::fs::remove_dir(&libraryDirectory);
  let _ = std::fs::remove_dir(&directory);
}

// =================================================================================================
