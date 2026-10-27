use crate::parser::parser::runCodeFile;
use crate::parser::structure::structure::Structure;
use crate::parser::testing::{checkStructure, createDirectory, relativePath, showStructure, writeCodeFile};
use std::sync::{Arc, RwLock};
// =================================================================================================

// Тесты на `import()`; Путь считается от файла, в котором он написан.

// =================================================================================================

/// Простой импорт одного файла.
#[test]
fn simple() -> ()
{
  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  let directory: String = createDirectory!();

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  let testPath: String = writeCodeFile!(
    &directory,
    r#"
    hello() {{
      := 42
    }}

    value: Usize = 4
    "#
  );

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  let testRel: String = relativePath!(&testPath);
  let mainPath: String = writeCodeFile!(
    &directory,
    r#"
    a := import("{testRel}")
    "#
  );

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  let main: Arc<RwLock<Structure>> = runCodeFile(&mainPath);

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  checkStructure!(main, "a.hello", stype Method);
  checkStructure!(main, "a.value", stype Usize, value "4");

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  let _ = std::fs::remove_file(&testPath);
  let _ = std::fs::remove_file(&mainPath);
  let _ = std::fs::remove_dir(&directory);

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
}

// =================================================================================================

/// Дерево импортов по `relative` путям;
/// 
/// Для проверки создаются ручные папки и используются `absolute` пути
/// и далее подгоняются под реальные `relative`.
#[test]
fn nested() -> ()
{
  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  // Структура:
  //   dir/
  //     main         — Импортирует `lib/a` и `lib/deep/c`
  //     lib/
  //       a          — Импортирует `b`, вызывает `b.hello()`
  //       b          — `hello() { := "b: hello" }`
  //       deep/
  //         c        — Импортирует `d`, вызывает `d.hello()`
  //         d        — `hello() { := "d: hello" }`
  
  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  // Создание директорий.
  
  let directory: String = createDirectory!();
  
  let libraryDirectory: String = format!("{directory}/lib");
  std::fs::create_dir_all(&libraryDirectory).unwrap();
  
  let deepDirectory: String = format!("{libraryDirectory}/deep");
  std::fs::create_dir_all(&deepDirectory).unwrap();

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  let bPath: String = writeCodeFile!(
    &libraryDirectory,
    r#"
    hello() {{
      := "b: hello"
    }}
    "#
  );

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  let dPath: String = writeCodeFile!(
    &deepDirectory,
    r#"
    hello() {{
      := "d: hello"
    }}
    "#
  );

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  let dRel: String = relativePath!(&dPath);
  let cPath: String = writeCodeFile!(
    &deepDirectory,
    r#"
    d := import("{dRel}")
    dHello := d.hello()

    hello() {{
      := "c: hello"
    }}
    "#
  );

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  let bRel: String = relativePath!(&bPath);
  let cRelDeep: String = relativePath!(&cPath, "deep");
  let aPath: String = writeCodeFile!(
    &libraryDirectory,
    r#"
    b := import("{bRel}")
    bHello := b.hello()

    hello() {{
      := "a: hello"
    }}

    load() {{
      c := import("{cRelDeep}")
      := c.hello()
    }}
    "#
  );

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  let aRel: String = relativePath!(&aPath, "lib");
  let cRel: String = relativePath!(&cPath, "lib/deep");
  let mainPath: String = writeCodeFile!(
    &directory,
    r#"
    a := import("{aRel}")
    c := import("{cRel}")
    "#
  );

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  let main: Arc<RwLock<Structure>> = runCodeFile(&mainPath);

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  // `a.rt` импортировал `b.rt`; значение метода модуля вернулось в `bHello`.
  showStructure!(main, "a.bHello", stype);
  checkStructure!(
    main, "a.bHello",
    type String, stype String, value "b: hello"
  );
  checkStructure!(main, "a.hello", stype Method);

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  // `c.rt` импортировал `d.rt`; то же для вложенного на уровень глубже.
  showStructure!(main, "c.dHello", stype);
  checkStructure!(
    main, "c.dHello",
    type String, stype String, value "d: hello"
  );
  checkStructure!(main, "c.hello", stype Method);

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  let _ = std::fs::remove_file(&bPath);
  let _ = std::fs::remove_file(&aPath);
  let _ = std::fs::remove_file(&dPath);
  let _ = std::fs::remove_file(&cPath);
  let _ = std::fs::remove_file(&mainPath);
  let _ = std::fs::remove_dir(&deepDirectory);
  let _ = std::fs::remove_dir(&libraryDirectory);
  let _ = std::fs::remove_dir(&directory);
  
  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
}

// =================================================================================================
