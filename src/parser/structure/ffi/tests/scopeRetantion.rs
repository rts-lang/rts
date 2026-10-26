use crate::parser::parser::runCodeFile;
use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::parser::testing::{buildLibrary, checkStructure, writeCodeFile};
use std::sync::{Arc, RwLock};
// =================================================================================================

// Тесты на FFI блоки.
//
//  Стоит помнить - что каждый блок = структура.
//  Здесь проверяются именные и безымянные блоки.
//
//  Результат FFI-вызова пишется в переменную, объявленную выше блока:
//  так видно, что блок держит свой scope и может менять внешние структуры.

// =================================================================================================

/// Хелпер: Собирает библиотеку с функцией `hello` для принятия и возврата числа.
fn buildHelloLibrary() -> String
{
  buildLibrary!(
    r#"
    int hello(int number) {
      return number;
    }
    "#
  )
}

// =================================================================================================

/// 1: Просто за блоком;
/// Метод при вызове автоматически обернётся в блок.
#[test]
fn plainCall() -> ()
{
  let libraryPath: String = buildHelloLibrary();

  let path: String = writeCodeFile!(
    r#"
    result~: U32 = 0

    lib: Pointer = importNative("{libraryPath}")
    result = lib.hello(1)
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "result", stype StructureType::Primitive(PrimitiveType::U32), value "1");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

/// 2: Перед методом.
#[test]
fn methodBlock() -> ()
{
  let libraryPath: String = buildHelloLibrary();

  let path: String = writeCodeFile!(
    r#"
    result~: U32 = 0

    [ffi]
    test() {{
      lib: Pointer = importNative("{libraryPath}")
      result = lib.hello(2)
    }}
    test()
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "result", stype StructureType::Primitive(PrimitiveType::U32), value "2");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

/// 3: Перед методом (в одну строку).
#[test]
fn methodBlockOneLine() -> ()
{
  let libraryPath: String = buildHelloLibrary();

  let path: String = writeCodeFile!(
    r#"
    result~: U32 = 0

    [ffi] test2() {{
      lib: Pointer = importNative("{libraryPath}")
      result = lib.hello(3)
    }}
    test2()
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "result", stype StructureType::Primitive(PrimitiveType::U32), value "3");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

/// 4: Безымянный блок.
#[test]
fn unnamedBlock() -> ()
{
  let libraryPath: String = buildHelloLibrary();

  let path: String = writeCodeFile!(
    r#"
    result~: U32 = 0

    [ffi]
    {{
      lib: Pointer = importNative("{libraryPath}")
      result = lib.hello(4)
    }}
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "result", stype StructureType::Primitive(PrimitiveType::U32), value "4");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

/// 5: Безымянный блок (в одну строку).
#[test]
fn unnamedBlockOneLine() -> ()
{
  let libraryPath: String = buildHelloLibrary();

  let path: String = writeCodeFile!(
    r#"
    result~: U32 = 0

    [ffi] {{
      lib: Pointer = importNative("{libraryPath}")
      result = lib.hello(5)
    }}
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "result", stype StructureType::Primitive(PrimitiveType::U32), value "5");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

/// 6: Именованный блок.
#[test]
fn namedBlock() -> ()
{
  let libraryPath: String = buildHelloLibrary();

  let path: String = writeCodeFile!(
    r#"
    result~: U32 = 0

    [ffi]
    name {{
      lib: Pointer = importNative("{libraryPath}")
      result = lib.hello(6)
    }}
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "result", stype StructureType::Primitive(PrimitiveType::U32), value "6");
  checkStructure!(main, "name", stype StructureType::Primitive(PrimitiveType::Method));

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

/// 7: Именованный блок (в одну строку).
#[test]
fn namedBlockOneLine() -> ()
{
  let libraryPath: String = buildHelloLibrary();

  let path: String = writeCodeFile!(
    r#"
    result~: U32 = 0

    [ffi] name {{
      lib: Pointer = importNative("{libraryPath}")
      result = lib.hello(7)
    }}
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "result", stype StructureType::Primitive(PrimitiveType::U32), value "7");
  checkStructure!(main, "name", stype StructureType::Primitive(PrimitiveType::Method));

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

// =================================================================================================
