
/// run file path
pub static _filePath: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Файл, код которого исполняется прямо сейчас (`None` — запущенный файл, см. `_filePath`).
///
/// От него import() считает относительные пути.
/// Меняется на время import() и на время вызова функции из импортированного файла,
/// структуры запоминают его при создании (см. `Structure::sourcePath`).
pub static _sourcePath: std::sync::RwLock<Option<std::sync::Arc<String>>> =
  std::sync::RwLock::new(None);

/// arguments count
pub static _argc: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
/// arguments vector
pub static _argv: std::sync::OnceLock< Vec<String> > = std::sync::OnceLock::new();

/// Значение, которое вернёт программа при завершении
pub static _exitCode: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
/// Завершилась ли программа?
/// 
/// // todo Зачем ты нужен если есть exit code?
pub static _exit: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// version
pub static _version: &str = "241201";
