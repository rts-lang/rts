/// run file path
pub static mut _filePath: String = String::new();
/// Вывод дебага>
pub static mut _debugMode: bool = false;

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