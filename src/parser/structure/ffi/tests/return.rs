use crate::parser::parser::runCodeFile;
use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::StructureType;
use crate::parser::testing::{buildLibrary, checkStructure, writeCodeFile};
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock};
// =================================================================================================

// Тесты на возврат результата FFI-вызова.
//
//  Вариант 1: тип слева указан — он же тип возврата C-функции,
//             правая часть кастуется к левой.
//  Вариант 2: тип слева не указан — левая часть получает тип от правой:
//             то же авто-приведение по величине значения, что и для примитивов
//             `Usize` в результате не бывает.

// =================================================================================================

/// Хелпер: Собирает библиотеку с функциями возврата.
fn buildReturnsLibrary() -> String
{
  buildLibrary!(
    r#"
    #include <stdint.h>
    #include <stddef.h>

    // Функции без аргументов — по одной на каждый ABI-тип результата
    uint8_t  retU8(void)    { return 200; }
    uint16_t retU16(void)   { return 60000; }
    uint32_t retU32(void)   { return 4000000000u; }
    uint64_t retU64(void)   { return 18446744073709551615ull; }
    size_t   retUsize(void) { return 123456789; }
    int8_t   retI8(void)    { return -100; }
    int16_t  retI16(void)   { return -30000; }
    int32_t  retI32(void)   { return -2000000000; }
    int64_t  retI64(void)   { return -9000000000000000000ll; }
    float    retF32(void)   { return -1.5f; }
    double   retF64(void)   { return 2.5; }
    void     retVoid(void)  {}

    // Аргументы и результат
    uint16_t addU8(uint8_t a, uint8_t b)  { return (uint16_t)(a + b); }
    double   mulF64(double a, double b)   { return a * b; }
    "#
  )
}

// =================================================================================================

/// Вариант 1: `a: Type = lib.f()`.
#[test]
fn typedReturns() -> ()
{
  let libraryPath: String = buildReturnsLibrary();

  let path: String = writeCodeFile!(
    r#"
    lib: Pointer = importNative("{libraryPath}")

    a1: U8    = lib.retU8()
    a2: U16   = lib.retU16()
    a3: U32   = lib.retU32()
    a4: U64   = lib.retU64()
    a5: Usize = lib.retUsize()
    a6: I8    = lib.retI8()
    a7: I16   = lib.retI16()
    a8: I32   = lib.retI32()
    a9: I64   = lib.retI64()
    b1: F32   = lib.retF32()
    b2: F64   = lib.retF64()
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "a1", stype StructureType::U8, value "200");
  checkStructure!(main, "a2", stype StructureType::U16, value "60000");
  checkStructure!(main, "a3", stype StructureType::U32, value "4000000000");
  checkStructure!(main, "a4", stype StructureType::U64, value "18446744073709551615");
  checkStructure!(main, "a5", stype StructureType::Usize, value "123456789");
  checkStructure!(main, "a6", stype StructureType::I8, value "-100");
  checkStructure!(main, "a7", stype StructureType::I16, value "-30000");
  checkStructure!(main, "a8", stype StructureType::I32, value "-2000000000");
  checkStructure!(main, "a9", stype StructureType::I64, value "-9000000000000000000");
  checkStructure!(main, "b1", stype StructureType::F32, value "-1.5");
  checkStructure!(main, "b2", stype StructureType::F64, value "2.5");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

/// Вариант 1 с системными библиотеками `libc`/`libm`.
#[test]
fn typedReturnsFromSystemLibraries() -> ()
{
  let path: String = writeCodeFile!(
    r#"
    libc: Pointer = importNative("libc.so.6")
    libm: Pointer = importNative("libm.so.6")

    s1: Usize = libc.strnlen("hello world")
    s2: I32   = libc.abs(-5)
    s3: I32   = libc.close(999)
    s4: F64   = libm.sqrt(16.0)
    s5: F64   = libm.floor(-2.5)
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "s1", stype StructureType::Usize, value "11");
  checkStructure!(main, "s2", stype StructureType::I32, value "5");
  checkStructure!(main, "s3", stype StructureType::I32, value "-1");
  checkStructure!(main, "s4", stype StructureType::F64, value "4");
  checkStructure!(main, "s5", stype StructureType::F64, value "-3");

  let _ = std::fs::remove_file(&path);
}

// =================================================================================================

/// Вариант 2: `a = lib.f()`.
///
/// Тип структуры выводится из значения — так же, как для `a = 10` (native/types,
/// issue #63): округление по величине, не по фактической ширине C-функции.
/// Usize здесь не появляется никогда — ветка U64 перехватывает значение раньше
/// (`u64::MAX == usize::MAX` на 64-бит, см. structureType.rs).
///
/// На ABI-уровне `chillffi` при этом всегда читает полный регистр (usize) —
/// отсюда точность гарантирована только для результатов размером со слово:
/// `size_t`, `uint64_t`, указатели. Для знаковых, узких (8/16/32 бит) и `float`
/// результатов указывайте тип слева.

/// todo desc
#[test]
fn inferredReturns() -> ()
{
  let libraryPath: String = buildReturnsLibrary();

  let path: String = writeCodeFile!(
    r#"
    lib: Pointer = importNative("{libraryPath}")
    libc: Pointer = importNative("libc.so.6")

    c1 := lib.retUsize()
    c2 := lib.retU64()
    c3 := libc.strnlen("hello world")
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "c1", stype StructureType::U32, value "123456789");
  checkStructure!(main, "c2", stype StructureType::U64, value "18446744073709551615");
  checkStructure!(main, "c3", stype StructureType::U8, value "11");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

/// Аргументы и результат.
#[test]
fn argumentsAndResult() -> ()
{
  let libraryPath: String = buildReturnsLibrary();

  let path: String = writeCodeFile!(
    r#"
    lib: Pointer = importNative("{libraryPath}")

    d1: U16 = lib.addU8(200; 100)
    d2: F64 = lib.mulF64(1.5; 4.0)
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "d1", stype StructureType::U16, value "300");
  checkStructure!(main, "d2", stype StructureType::F64, value "6");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

/// Результат внутри выражения.
#[test]
fn resultInsideExpression() -> ()
{
  let libraryPath: String = buildReturnsLibrary();

  let path: String = writeCodeFile!(
    r#"
    lib: Pointer = importNative("{libraryPath}")
    libc: Pointer = importNative("libc.so.6")

    e1: Usize = lib.retUsize() + 1
    e2: I32 = lib.retI32() + 1
    e3 := libc.strnlen("abc") + 1
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "e1", stype StructureType::Usize, value "123456790");
  checkStructure!(main, "e2", stype StructureType::I32, value "-1999999999");
  checkStructure!(main, "e3", stype StructureType::U8, value "4");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

/// Тип слева применяется ко всем FFI-вызовам верхнего уровня выражения.
///
/// todo В `a: I32 = f() + g()` оба вызова читаются как I32, отдельный тип на каждый вызов не задать.
#[test]
fn typeAppliesToAllCalls() -> ()
{
  let path: String = writeCodeFile!(
    r#"
    libc: Pointer = importNative("libc.so.6")

    t1: I32 = libc.abs(-2) + libc.abs(-3)
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "t1", stype StructureType::I32, value "5");

  let _ = std::fs::remove_file(&path);
}

/// Аргумент из константы.
#[test]
fn argumentFromConstant() -> ()
{
  let path: String = writeCodeFile!(
    r#"
    libc: Pointer = importNative("libc.so.6")

    y: I32 = libc.close(999)
    t2: I32 = libc.abs(y)
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "t2", stype StructureType::I32, value "1");

  let _ = std::fs::remove_file(&path);
}

/// Присваивание в существующую структуру.
#[test]
fn assignmentToExistingStructure() -> ()
{
  let libraryPath: String = buildReturnsLibrary();

  let path: String = writeCodeFile!(
    r#"
    lib: Pointer = importNative("{libraryPath}")

    r1~ : I32 = 0
    r1 = lib.retI32()
    r2~~ := 0
    r2 = lib.retUsize()
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "r1", stype StructureType::I32, value "-2000000000");
  checkStructure!(main, "r2", stype StructureType::U32, value "123456789");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

/// Вызов-оператор: результат не нужен.
///
/// `marker` в конце - чтобы было видно, что файл дочитан до конца.
#[test]
fn callAsStatement() -> ()
{
  let libraryPath: String = buildReturnsLibrary();

  let path: String = writeCodeFile!(
    r#"
    lib: Pointer = importNative("{libraryPath}")

    lib.retVoid()
    lib.retI32()

    marker: Usize = 1
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "marker", stype StructureType::Usize, value "1");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

/// Pointer: адрес читается как число.
///
/// todo Вне [ffi] блока scope временный, поэтому полученный адрес после вызова использовать нельзя.
#[test]
fn pointerResult() -> ()
{
  let path: String = writeCodeFile!(
    r#"
    libc: Pointer = importNative("libc.so.6")

    p: Pointer = libc.malloc(16)
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "p", stype StructureType::Pointer);

  let _ = std::fs::remove_file(&path);
}

/// Ошибки: неизвестный метод и тип без поддержки результата дают None → default типа.
#[test]
fn unknownMethod() -> ()
{
  let libraryPath: String = buildReturnsLibrary();

  let path: String = writeCodeFile!(
    r#"
    lib: Pointer = importNative("{libraryPath}")

    z1: I32 = lib.noSuchFunction()
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "z1", stype StructureType::I32, value "0");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

/// Внутри `[ffi]` блока (scope retention).
#[test]
fn insideFfiBlock() -> ()
{
  let libraryPath: String = buildReturnsLibrary();

  let path: String = writeCodeFile!(
    r#"
    [ffi]
    block {{
      inner: Pointer = importNative("{libraryPath}")
      f1: I32 = inner.retI32()
      f2 := inner.retUsize()
    }}
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "block.f1", stype StructureType::I32, value "-2000000000");
  checkStructure!(main, "block.f2", stype StructureType::U32, value "123456789");

  let _ = std::fs::remove_file(&path);
  let _ = std::fs::remove_file(&libraryPath);
}

/// Результат FFI-вызова как return функции (`= value`).
///
/// todo Библиотека, объявленная снаружи функции, внутри неё не видна: `= libc.strnlen(s)` даёт None.
#[test]
fn resultAsFunctionReturn() -> ()
{
  let path: String = writeCodeFile!(
    r#"
    lenOf(s: String) {{
      libc: Pointer = importNative("libc.so.6")
      = libc.strnlen(s)
    }}

    w1 := lenOf("hello")
    w2: Usize = lenOf("hello world")
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  checkStructure!(main, "w1", stype StructureType::U8, value "5");
  checkStructure!(main, "w2", stype StructureType::Usize, value "11");

  let _ = std::fs::remove_file(&path);
}

/// F32-аргументы: ожидается 4, у нас 0.
///
/// todo float-токен всегда уходит как F64.
#[test]
#[ignore = "todo: F32-аргументы - float-токен всегда уходит как F64"]
fn f32Arguments() -> ()
{
  let path: String = writeCodeFile!(
    r#"
    libm: Pointer = importNative("libm.so.6")

    f1: F32 = libm.sqrtf(16.0)
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  // Ожидается 4, а приходит 0.
  checkStructure!(main, "f1", stype StructureType::F32, value "4");

  let _ = std::fs::remove_file(&path);
}

/// True/False в результате: ожидается True, у нас None.
///
/// todo issue #65.
#[test]
#[ignore = "todo: True/False в результате (issue #65) - у нас None"]
fn booleanResult() -> ()
{
  let path: String = writeCodeFile!(
    r#"
    libc: Pointer = importNative("libc.so.6")

    b1: True | False = libc.abs(1)
    "#
  );

  let main: Arc<RwLock<Structure>> = runCodeFile(&path);

  // Ожидается True, а приходит None.
  checkStructure!(
    main, "b1",
    type TokenType::True, stype StructureType::True, value "True"
  );

  let _ = std::fs::remove_file(&path);
}

// =================================================================================================
//
//  Известные проблемы и что не проверено:
//
//  todo CString-аргументы. Ожидается 5, у нас 5, 6, 6 от запуска к запуску:
//  String уходит как (ptr, len) без NUL, а FfiArgValue::CString нигде не создаётся
//  ```
//  s1: Usize = libc.strlen("hello")
//  println(f"strlen {s1}") # ожидается 5
//  ```
//
//  todo F32-аргументы. Ожидается 4, у нас 0: float-токен всегда уходит как F64:
//  ```
//  f1: F32 = libm.sqrtf(16.0)
//  println(f"sqrtf {f1}") # ожидается 4
//  ```
//
//  todo True/False в результате (issue #65). Ожидается True, у нас None:
//  ```
//  b1: True | False = libc.abs(1)
//  println(f"bool {b1}") # ожидается True
//  ```
//
//  todo Вложенные скобки обрывают токенизацию — остаток файла молча отбрасывается.
//  Токенизатор (println((1+2)) ломается так же)
//  ```
//  println(libc.abs(5)) # ожидается 5
//  ```
//
//  todo Не проверено: x86-32 (build.sh 32). Usize/Isize там 32 бита, ожидается тот же вывод,
//  кроме U64/I64 в Infer (там читается слово Usize)
//  НО: Правильнее всего вырезать 32-бит сейчас по примеру chillffi.
//
// =================================================================================================
