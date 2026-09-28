/// run file path
pub static mut _filePath: String = String::new();

/// Файл, код которого исполняется прямо сейчас (`None` — запущенный файл, см. `_filePath`).
///
/// От него import() считает относительные пути.
/// Меняется на время import() и на время вызова функции из импортированного файла,
/// структуры запоминают его при создании (см. `Structure::sourcePath`).
pub static mut _sourcePath: Option<std::sync::Arc<String>> = None;

/// arguments count
pub static mut _argc: usize = 0;
/// arguments vector
pub static mut _argv: Vec<String> = Vec::new();

/// Значение, которое вернёт программа при завершении
pub static mut _exitCode: i32 = 0;
/// Завершилась ли программа?
pub static mut _exit: bool = false; // todo Зачем ты нужен если есть exit code?
/// version
pub static _version: &str = "241201";
