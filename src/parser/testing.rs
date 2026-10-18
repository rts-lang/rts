use std::sync::atomic::{AtomicUsize, Ordering};
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::StructureType;
use crate::tokenizer::tokenizer::readTokensSimple;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

// Общие помощники для `tests` парсера.
//
// todo desc надо или нет?

// =================================================================================================

/// Считает выражение из токенов и возвращает токен-результат.
///
/// Для тестов Token/TokenType: структуры и StructureType тут не участвуют, поэтому нет `a = ...`.
pub(super) fn expressionOf(code: &str) -> Token
{
  let mut buffer: Vec<u8> = code.as_bytes().to_vec();
  let lines: Vec< Arc<RwLock<Line>> > = readTokensSimple(&mut buffer);
  
  let mut tokens: Vec<Token> = lines.first()
    .unwrap_or_else(|| panic!("Expression `{}` has no lines", code))
    .read().unwrap().tokens.clone()
    .unwrap_or_else(|| panic!("Expression `{}` has no tokens", code));
  
  let main: Structure = Structure::new(
    Some(String::from("main")),
    StructureMut::Constant,
    StructureType::Method,
    None,
    None
  );
  
  main.expression(&mut tokens)
}

/// Возвращает структуру по `name` из выполненного кода; если её нет - паника.
///
/// `"a"` — по области видимости; `"a.b"` — строго вниз по пути.
pub(super) fn getStructure(main: &Arc<RwLock<Structure>>, name: &str) -> Arc<RwLock<Structure>>
{
  let mainGuard: RwLockReadGuard<Structure> = main.read().unwrap();
  let found: Option< Arc<RwLock<Structure>> > =
    if name.contains('.') {
      mainGuard.getStructureByLink(name)
    } else {
      mainGuard.getStructureByScope(name)
    };
  found.unwrap_or_else(|| panic!("Structure '{}' was not created", name))
}

/// Токен структуры: тип токена, строковое значение.
fn tokenOf(main: &Arc<RwLock<Structure>>, name: &str) -> (TokenType, String)
{
  let structure: Arc<RwLock<Structure>> = getStructure(main, name);
  let structureLock: RwLockReadGuard<Structure> = structure.read().unwrap();
  
  let line: RwLockReadGuard<Line> = structureLock.lines.as_ref()
    .unwrap_or_else(|| panic!("Structure '{}' has no lines", name))[0].read().unwrap();
  let token: &Token = line.tokens.as_ref()
    .unwrap_or_else(|| panic!("Structure '{}' has no tokens", name))
    .first().unwrap();
  
  (*token.getDataType(), token.getData().toString().unwrap_or_default())
}

// =================================================================================================

// Грани возвращают `Err(текст)`, а не паникуют: `checkStructure!` паникует сам, а `tryCheckStructure!` отдаёт
// `Result`, поэтому ошибку сверки можно проверить без паники и без вывода в `--nocapture`.

/// `type` - тип токена.
pub(super) fn checkType(main: &Arc<RwLock<Structure>>, name: &str, code: &str, expected: &TokenType) -> Result<(), String>
{
  let (tokenType, _): (TokenType, String) = tokenOf(main, name);
  if tokenType == *expected { return Ok(()); }
  Err(format!(
    "check type in `{}` for '{}': expected '{}', got '{}'",
    code, name, expected.to_string(), tokenType.to_string()
  ))
}

/// `stype` - тип структуры.
pub(super) fn checkStype(main: &Arc<RwLock<Structure>>, name: &str, code: &str, expected: &StructureType) -> Result<(), String>
{
  let structure: Arc<RwLock<Structure>> = getStructure(main, name);
  let structureLock: RwLockReadGuard<Structure> = structure.read().unwrap();
  if structureLock.dataType == *expected { return Ok(()); }
  Err(format!(
    "check stype in `{}` for '{}': expected '{}', got '{}'",
    code, name, expected.to_string(), structureLock.dataType.to_string()
  ))
}

/// `utype` - полный объявленный тип: набор вариантов объединения или `stype`, если объединения нет.
pub(super) fn checkUtype(main: &Arc<RwLock<Structure>>, name: &str, code: &str, expected: &StructureType) -> Result<(), String>
{
  let structure: Arc<RwLock<Structure>> = getStructure(main, name);
  let structureLock: RwLockReadGuard<Structure> = structure.read().unwrap();
  let utype: StructureType = structureLock.utype();
  if utype == *expected { return Ok(()); }
  Err(format!(
    "check utype in `{}` for '{}': expected '{}', got '{}'",
    code, name, expected.to_string(), utype.to_string()
  ))
}

/// `value` - значение структуры.
pub(super) fn checkValue(main: &Arc<RwLock<Structure>>, name: &str, code: &str, expected: &str) -> Result<(), String>
{
  let (_, data): (TokenType, String) = tokenOf(main, name);
  if data == expected { return Ok(()); }
  Err(format!(
    "check value in `{}` for '{}': expected '{}', got '{}'",
    code, name, expected, data
  ))
}

/// `mut` - уровень изменения структуры.
pub(super) fn checkMut(main: &Arc<RwLock<Structure>>, name: &str, code: &str, expected: &StructureMut) -> Result<(), String>
{
  let structure: Arc<RwLock<Structure>> = getStructure(main, name);
  let structureLock: RwLockReadGuard<Structure> = structure.read().unwrap();
  let mutable: StructureMut = structureLock.mutable.clone();
  if mutable == *expected { return Ok(()); }
  Err(format!(
    "check mut in `{}` for '{}': expected '{}', got '{}'",
    code, name, expected.to_string(), mutable.to_string()
  ))
}

/// `lines` - сколько строк внутри структуры.
pub(super) fn checkLines(main: &Arc<RwLock<Structure>>, name: &str, code: &str, expected: usize) -> Result<(), String>
{
  let structure: Arc<RwLock<Structure>> = getStructure(main, name);
  let structureLock: RwLockReadGuard<Structure> = structure.read().unwrap();
  let lines: usize = structureLock.lines.as_ref().map(|l| l.len()).unwrap_or(0);
  if lines == expected { return Ok(()); }
  Err(format!(
    "check lines in `{}` for '{}': expected '{}', got '{}'",
    code, name, expected, lines
  ))
}

/// Печатает все грани структуры: `type`, `stype`, `utype`, `value`.
///
/// Для отладки кейсов без assert.
/// 
/// В `cargo test` вывод виден только с `-- --nocapture`
/// (или `RUST_TEST_NOCAPTURE=1`).
pub(super) fn printStructure(main: &Arc<RwLock<Structure>>, name: &str, code: &str) -> ()
{
  let (tokenType, data): (TokenType, String) = tokenOf(main, name);
  
  let structure: Arc<RwLock<Structure>> = getStructure(main, name);
  let structureLock: RwLockReadGuard<Structure> = structure.read().unwrap();
  
  let stype: String = structureLock.dataType.to_string();
  let utype: String = structureLock.utype().to_string();
  println!(
    "show `{}` for '{}' | type={}, stype={}, utype={}, value={}",
    code, name, tokenType.to_string(), stype, utype, data
  );
}

// =================================================================================================

/// Проверка структуры одной строкой; выполняет код и сверяет ТОЛЬКО указанные грани.
///
/// Грани юзаются в любом наборе - ровно те, что проверяет `.rt` файл: где-то сверяются
/// `type` и `stype`, где-то только один из них, где-то ещё и значение. Порядок граней
/// фиксированный - `type` → `stype` → `utype` → `value`, как в `.rt` файлах (`type(a) | stype(a) = a`):
///
///     checkStructure!("a: U8 = 300", "a", type TokenType::UInt, stype StructureType::U8, value "255");
///     checkStructure!("a := 0.0", "a", type TokenType::UFloat, stype StructureType::F32); // Без значения
///     checkStructure!("a := 10", "a", stype StructureType::U8); // Только stype
///     checkStructure!("a: U8 | 10 = 10", "a", stype number("10"), utype Union(vec![U8, number("10")]));
///
/// Грани: `type` - тип токена (`type(a)` в `.rt`), `stype` - тип структуры (`stype(a)`),
/// `utype` - полный объявленный тип: набор вариантов объединения (`utype(a)`), а без
/// объединения он равен `stype`; `value` - значение (`{a}`).
/// Две формы, имя структуры всегда строкой (`&str`/литерал; путь через точку: `"m.a"`):
///
///     checkStructure!("a := 10", "a", value "10");   // код
///     checkStructure!(main, "a", value "10");       // выполненный `main`
///
/// Паникует на первой неверной грани; `tryCheckStructure!` вместо паники возвращает `Err`.
macro_rules! checkStructure
{
  ($($arguments:tt)*) =>
  {
    if let Err(message) = $crate::parser::testing::tryCheckStructure!($($arguments)*)
    {
      panic!("{}", message);
    }
  };
}

pub(super) use checkStructure;

/// Версия `checkStructure!` без паники: возвращает `Result<(), String>`,
/// где первая неверная грань - `Err` с сообщением. Формы и грани те же.
macro_rules! tryCheckStructure
{
  // Код: `tryCheckStructure!("a := 10", "a", value "10")`.
  ($code:literal, $name:expr $(, type $t:expr)? $(, stype $s:expr)? $(, utype $u:expr)? $(, value $v:expr)? $(, mut $m:expr)? $(, lines $l:expr)? $(,)?) =>
  {{
    let main: ::std::sync::Arc<::std::sync::RwLock<$crate::parser::structure::structure::Structure>> =
      $crate::parser::parser::runCode($code);
    (|| -> Result<(), String>
    {
      $(
        $crate::parser::testing::checkType(&main, $name, $code, &$t)?;
      )?
      $(
        $crate::parser::testing::checkStype(&main, $name, $code, &$s)?;
      )?
      $(
        $crate::parser::testing::checkUtype(&main, $name, $code, &$u)?;
      )?
      $(
        $crate::parser::testing::checkValue(&main, $name, $code, &$v)?;
      )?
      $(
        $crate::parser::testing::checkMut(&main, $name, $code, &$m)?;
      )?
      $(
        $crate::parser::testing::checkLines(&main, $name, $code, $l)?;
      )?
      Ok(())
    })()
  }};

  // Структура: `tryCheckStructure!(main, "a", value "10")`.
  ($main:expr, $name:expr $(, type $t:expr)? $(, stype $s:expr)? $(, utype $u:expr)? $(, value $v:expr)? $(, mut $m:expr)? $(, lines $l:expr)? $(,)?) =>
  {{
    let main = &$main;
    (|| -> Result<(), String>
    {
      $(
        $crate::parser::testing::checkType(main, $name, "", &$t)?;
      )?
      $(
        $crate::parser::testing::checkStype(main, $name, "", &$s)?;
      )?
      $(
        $crate::parser::testing::checkUtype(main, $name, "", &$u)?;
      )?
      $(
        $crate::parser::testing::checkValue(main, $name, "", &$v)?;
      )?
      $(
        $crate::parser::testing::checkMut(main, $name, "", &$m)?;
      )?
      $(
        $crate::parser::testing::checkLines(main, $name, "", $l)?;
      )?
      Ok(())
    })()
  }};
}

#[allow(unused_imports)]
pub(super) use tryCheckStructure;

// =================================================================================================

/// Отладка без assert: выполняет код и печатает type, stype, value структуры.
///
/// Принимает те же грани, что и `checkStructure!`: чтобы переключиться между показом и
/// проверкой, достаточно поменять имя макроса. Печатает ВСЕ грани, а для указанных
/// ещё и расхождение с ожиданием (`≠`). Не паникует.
///
/// Вывод в консоль:
///
///     cargo test -- --nocapture --test-threads=1
///     cargo test testName -- --nocapture
///
///     showStructure!("a := 1e308 + 1e308", "a", type TokenType::UFloat);
///     // show `a = 1e308 + 1e308` for 'a' | type=UFloat, stype=F64, value=1.7976931348623157e308
macro_rules! showStructure
{
  // `showStructure!("a := 10", "a", value "11")` — имя структуры строкой (путь: `"a.b"`).
  ($code:expr, $name:expr $(, type $t:expr)? $(, stype $s:expr)? $(, utype $u:expr)? $(, value $v:expr)? $(, mut $m:expr)? $(, lines $l:expr)? $(,)?) =>
  {{
    let main = $crate::parser::parser::runCode($code);
    $crate::parser::testing::printStructure(&main, $name, $code);
    $(
      if let Err(message) = $crate::parser::testing::checkType(&main, $name, $code, &$t)
      { println!("  ≠ {}", message); }
    )?
    $(
      if let Err(message) = $crate::parser::testing::checkStype(&main, $name, $code, &$s)
      { println!("  ≠ {}", message); }
    )?
    $(
      if let Err(message) = $crate::parser::testing::checkUtype(&main, $name, $code, &$u)
      { println!("  ≠ {}", message); }
    )?
    $(
      if let Err(message) = $crate::parser::testing::checkValue(&main, $name, $code, &$v)
      { println!("  ≠ {}", message); }
    )?
    $(
      if let Err(message) = $crate::parser::testing::checkMut(&main, $name, $code, &$m)
      { println!("  ≠ {}", message); }
    )?
    $(
      if let Err(message) = $crate::parser::testing::checkLines(&main, $name, $code, $l)
      { println!("  ≠ {}", message); }
    )?
  }};
}

#[allow(unused_imports)]
pub(super) use showStructure;

// =================================================================================================

/// Смоук-тесты самого `tryCheckStructure!`: разные наборы граней компилируются и реально сверяются.
/// Проверка выражения из токенов (без `a = ...`): сверяются ТОЛЬКО указанные грани.
///
/// Грани: `type` - тип токена результата, `value` - его значение; порядок `type` → `value`.
///
///     checkExpression!("(1e308 + 1e308) - (1e308 + 1e308)", type TokenType::UFloat, value "0");
///     checkExpression!("1e309", value &f64::MAX.to_string()); // Только значение
///
/// Паникует на первой неверной грани.
macro_rules! checkExpression
{
  ($code:expr $(, type $t:expr)? $(, value $v:expr)? $(,)?) =>
  {{
    let code: &str = $code;
    let token: $crate::tokenizer::types::token::Token = $crate::parser::testing::expressionOf(code);
    $(
      let expectedType: $crate::tokenizer::types::tokenType::TokenType = $t;
      assert!(
        *token.getDataType() == expectedType,
        "check expression type in `{}`: expected '{}', got '{}'",
        code, expectedType.to_string(), token.getDataType().to_string()
      );
    )?
    $(
      let expectedValue: &str = &$v;
      let data: String = token.getData().toString().unwrap_or_default();
      assert!(
        data == expectedValue,
        "check expression value in `{}`: expected '{}', got '{}'",
        code, expectedValue, data
      );
    )?
  }};
}

pub(super) use checkExpression;

// =================================================================================================

pub(super) static FileCounter: AtomicUsize = AtomicUsize::new(0);
pub(super) static LibraryCounter: AtomicUsize = AtomicUsize::new(0);

// =================================================================================================

/// Создаёт папку в `/tmp` с уникальным номером и возвращает путь к ней.
///
/// Для тестов с `import()`, где несколько `.rt` файлов лежат в одной папке.
///
///     let dir: String = createDirectory!();
///     let aPath: String = writeCodeFile!(&dir, r#"hello() {{ = 42 }}"#);
///     let aRel: String = relativePath!(&aPath);
///     let mainPath: String = writeCodeFile!(&dir, r#"a := import("{aRel}")"#);
macro_rules! createDirectory
{
  () =>
  {{
    let id: usize = $crate::parser::testing::FileCounter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let directoryPath: std::path::PathBuf = std::env::temp_dir().join(format!("rtsDir{id}"));

    std::fs::create_dir_all(&directoryPath)
      .unwrap_or_else(|error| panic!("Create dir `{}`: {}", directoryPath.display(), error));

    directoryPath.to_string_lossy().replace('\\', "/")
  }};
}

pub(super) use createDirectory;

// =================================================================================================

/// Absolute path → relative для `import()` в RTS-коде.
///
/// Берёт последний компонент пути; опциональный префикс каталога:
///
///     relativePath!(&path)              // `/tmp/x/10.rt` → `./10.rt`
///     relativePath!(&path, "lib")       // → `./lib/10.rt`
///     relativePath!(&path, "lib/deep")  // → `./lib/deep/10.rt`
macro_rules! relativePath
{
  ($absolute:expr) =>
  {{
    $crate::parser::testing::relativePathFrom(($absolute).as_ref(), None)
  }};

  ($absolute:expr, $dir:expr) =>
  {{
    $crate::parser::testing::relativePathFrom(($absolute).as_ref(), Some(($dir).as_ref()))
  }};
}

pub(super) use relativePath;

/// Absolute path → `./[dir/]name.rt` для `import()`.
pub(super) fn relativePathFrom(absolute: &str, dir: Option<&str>) -> String
{
  let name = std::path::Path::new(absolute)
    .file_name()
    .unwrap_or_else(|| panic!("relativePath: no file name in `{absolute}`"))
    .to_string_lossy();
  match dir
  {
    None => format!("./{name}"),
    Some(d) => format!("./{}/{name}", d.trim_matches('/')),
  }
}

// =================================================================================================

/// Записывает код в `.rt` с уникальным номером, возвращает **absolute** path.
///
/// - `writeCodeFile!(code)` — в `/tmp/{id}.rt`
/// - `writeCodeFile!(dir, code)` — в `{dir}/{id}.rt` (несколько файлов в одной папке для `import`)
///
/// Литерал проходит через `format!`: `{libraryPath}` / `{rel}` подставляются из scope.
/// Уже готовая `String` пишется как есть (без повторного `format!`).
///
///     let path: String = writeCodeFile!(r#"a := 10"#);
///     let path: String = writeCodeFile!(r#"lib = importNative("{libraryPath}")"#);
///     let aPath: String = writeCodeFile!(&dir, r#"hello() { = 42 }"#);
macro_rules! writeCodeFile
{
  // Литерал — format! (подстановка `{name}` из окружающего scope).
  ($code:literal) =>
  {{
    let id: usize = $crate::parser::testing::FileCounter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path: std::path::PathBuf = std::env::temp_dir().join(format!("{id}.rt"));
    let content: String = format!($code);
    std::fs::write(&path, &content)
      .unwrap_or_else(|error| panic!("Write of `{}`: {}", path.display(), error));
    path.to_string_lossy().replace('\\', "/")
  }};

  ($dir:expr, $code:literal) =>
  {{
    let id: usize = $crate::parser::testing::FileCounter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path: std::path::PathBuf = std::path::PathBuf::from($dir).join(format!("{id}.rt"));
    let content: String = format!($code);
    std::fs::write(&path, &content)
      .unwrap_or_else(|error| panic!("Write of `{}`: {}", path.display(), error));
    path.to_string_lossy().replace('\\', "/")
  }};

  // Уже собранная строка (например, результат внешнего `format!`).
  ($code:expr) =>
  {{
    let id: usize = $crate::parser::testing::FileCounter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path: std::path::PathBuf = std::env::temp_dir().join(format!("{id}.rt"));
    std::fs::write(&path, $code)
      .unwrap_or_else(|error| panic!("Write of `{}`: {}", path.display(), error));
    path.to_string_lossy().replace('\\', "/")
  }};

  ($dir:expr, $code:expr) =>
  {{
    let id: usize = $crate::parser::testing::FileCounter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path: std::path::PathBuf = std::path::PathBuf::from($dir).join(format!("{id}.rt"));
    std::fs::write(&path, $code)
      .unwrap_or_else(|error| panic!("Write of `{}`: {}", path.display(), error));
    path.to_string_lossy().replace('\\', "/")
  }};
}

pub(super) use writeCodeFile;

// =================================================================================================

/// Собирает C-код в динамическую библиотеку `.so` в `/tmp` с уникальным номером.
///
///     let libraryPath: String = buildLibrary!(r#"int add(int a, int b) { return a + b; }"#);
macro_rules! buildLibrary
{
  ($code:expr) =>
  {{
    let id: usize = $crate::parser::testing::LibraryCounter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // Временный `.c` файл.
    let sourcePath: std::path::PathBuf = std::env::temp_dir().join(format!("rtsSource-{id}.c"));

    std::fs::write(&sourcePath, $code)
      .unwrap_or_else(|error| panic!("Write of `{}`: {}", sourcePath.display(), error));

    // Компиляция.
    let libraryPath: std::path::PathBuf = std::env::temp_dir().join(format!("rtsLibrary-{id}.so"));

    let result: std::process::Output = std::process::Command::new("cc")
      .args(["-shared", "-fPIC", "-o"])
      .arg(&libraryPath)
      .arg(&sourcePath)
      .output()
      .unwrap_or_else(|error| panic!("Build: cc was not started: {}", error));

    // Удаляем исходник.
    let _ = std::fs::remove_file(&sourcePath);

    assert!(
      result.status.success(),
      "Build failed: {}",
      String::from_utf8_lossy(&result.stderr)
    );

    libraryPath.to_string_lossy().replace('\\', "/")
  }};
}

pub(super) use buildLibrary;

// =================================================================================================

#[cfg(test)]
mod tests
{
  use crate::parser::parser::runCode;
  use crate::parser::structure::structure::Structure;
  use crate::parser::structure::structureType::StructureType;
  use crate::tokenizer::types::tokenType::TokenType;
  use std::sync::{Arc, RwLock};
  // ===============================================================================================

  /// Вторая форма: выполненный `main` вместо кода; в тексте ошибки кода нет.
  #[test]
  fn mainSource() -> ()
  {
    let main: Arc<RwLock<Structure>> = runCode("a := 10");
    checkStructure!(main, "a", stype StructureType::U8, value "10");
    assert_eq!(
      tryCheckStructure!(main, "a", value "11").unwrap_err(),
      "check value in `` for 'a': expected '11', got '10'"
    );
  }

  /// Имя через точку - путь вниз: `f.n`.
  #[test]
  fn dottedName() -> ()
  {
    checkStructure!("f(n: UInt) {\n  = n\n}\nf(1)\n", "f.n", value "1");
  }

  /// Только `type` - как в `types1.rt` (`println(type(10+10))`).
  #[test]
  fn typeOnly() -> ()
  {
    checkStructure!("a := 10", "a", type TokenType::UInt);
  }

  /// Только `stype`.
  #[test]
  fn stypeOnly() -> ()
  {
    checkStructure!("a := 10", "a", stype StructureType::U8);
  }

  /// Только `value`.
  #[test]
  fn valueOnly() -> ()
  {
    checkStructure!("a := 10", "a", value "10");
  }

  /// Только `utype`: без объединения равен `stype`, с объединением - набор вариантов.
  #[test]
  fn utypeOnly() -> ()
  {
    checkStructure!("a := 10", "a", utype StructureType::U8);
    checkStructure!(
      "a: U8 | String = 10", "a",
      utype StructureType::Union(vec![StructureType::U8, StructureType::String])
    );
  }

  /// `stype` и `value` без `type`.
  #[test]
  fn stypeAndValue() -> ()
  {
    checkStructure!("a := 10", "a", stype StructureType::U8, value "10");
  }

  /// Все грани и висячая запятая.
  #[test]
  fn allFacetsTrailingComma() -> ()
  {
    checkStructure!("a := 10", "a", type TokenType::UInt, stype StructureType::U8, value "10",);
  }

  /// Сверка ловит неверную грань, а не проходит молча. Через `tryCheckStructure!`: паники нет.
  #[test]
  fn catchesMismatch() -> ()
  {
    assert_eq!(
      tryCheckStructure!("a := 10", "a", type TokenType::Int).unwrap_err(),
      "check type in `a := 10` for 'a': expected 'Int', got 'UInt'"
    );
    assert_eq!(
      tryCheckStructure!("a := 10", "a", stype StructureType::U16).unwrap_err(),
      "check stype in `a := 10` for 'a': expected 'U16', got 'U8'"
    );
    assert_eq!(
      tryCheckStructure!("a := 10", "a", value "11").unwrap_err(),
      "check value in `a := 10` for 'a': expected '11', got '10'"
    );
    assert_eq!(
      tryCheckStructure!("a: U8 | String = 10", "a", utype StructureType::U8).unwrap_err(),
      "check utype in `a: U8 | String = 10` for 'a': expected 'U8', got 'U8 | String'"
    );
    // Верные грани - `Ok`.
    assert!(tryCheckStructure!("a := 10", "a", type TokenType::UInt, stype StructureType::U8, value "10").is_ok());
  }

  /// `showStructure!` не паникует и печатает грани (смотреть с `-- --nocapture`).
  #[test]
  fn showStructure() -> ()
  {
    showStructure!("a := 10", "a");
  }

  // ===============================================================================================
}

// =================================================================================================
