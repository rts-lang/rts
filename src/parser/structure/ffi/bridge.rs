use std::num::IntErrorKind;
use chillffi::ffi;
use chillffi::ffi::errors::FFIError;
use chillffi::ffi::library::{CallBuilder, Library};
use chillffi::ffi::scope::Scope;
use std::sync::{Arc, RwLock};
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::StructureType;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Мост Token -> ABI StructureType::String.
///
/// StructureType не-custom типы — хардкод примитивов,
/// т.к. ABI chillffi сильно ограничен. String — единственный ABI-примитив,
/// у которого есть заранее подготовленные поля (.pointer .length), поэтому
/// только для него нужен явный мост из токена в 2 поля.
/// CString = Pointer (алиас) и RawString — обычные скаляры, моста не требуют:
/// данные и длина/NUL берутся прямо из токена в момент FFI-вызова (tokenToFfiArg).
///
/// Работает аддитивно: не трогает `.lines` структуры (там остаётся исходный
/// токен, поэтому голое имя `str` в `lib.print(str)` по-прежнему резолвится
/// в TokenType::String через обычный linkExpression() и идет в
/// FfiArgValue::String ниже — chillffi сам разложит его на pointer+len).
/// Вызывать после того, как `.lines` структуры уже выставлены.
pub fn stringFields(token: &Token) -> Option<[Arc<RwLock<Structure>>; 2]>
{
  match token.getDataType()
  {
    TokenType::String => {}
    _ => return None
  }
  let bytes: String = token.getData().toString()?;
  let length: usize = bytes.len();

  let pointerField: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
    Some("pointer".to_string()),
    StructureMut::Constant,
    StructureType::Pointer,
    Some(vec![Arc::new(RwLock::new(Line
    {
      tokens: Some(vec![token.clone()]),
      lines: None
    }))]),
    None
  )));

  let lengthField: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
    Some("length".to_string()),
    StructureMut::Constant,
    StructureType::Usize,
    Some(vec![Arc::new(RwLock::new(Line
    {
      tokens: Some(vec![Token::new(TokenType::UInt, length.to_string())]),
      lines: None
    }))]),
    None
  )));

  Some([pointerField, lengthField])
}

// =================================================================================================

/// Конкретный типизированный аргумент FFI, который можно положить в `CallBuilder::arg::<T>`.
///
/// chillffi жёстко разделяет `Value` (приватный enum) и публичный
/// `CallBuilder::arg::<T: FfiArg>(...)` — `Value` снаружи не сконструируешь,
/// поэтому единственный путь — превратить динамический `Token` в один
/// из известных типов и завернуть через typed builder.
enum FfiArgValue
{
  U8(u8),
  U16(u16),
  U32(u32),
  U64(u64),
  I8(i8),
  I16(i16),
  I32(i32),
  I64(i64),
  F64(f64),
  String(String),
  CString(std::ffi::CString),
  RawString(Vec<u8>)
}

/// Token -> FfiArgValue, с приведением размера к наименьшему подходящему типу.
fn tokenToFfiArg(token: &Token) -> Result<FfiArgValue, String>
{
  let tokenDataType: &TokenType = token.getDataType();
  let tokenData: String = token.getData().toString()
    .ok_or_else(|| "Token data is empty".to_owned())?;

  match tokenDataType
  {
    TokenType::UInt =>
    { // Токен бесконечен, а u64 нет: всё что больше - граница u64 (#71)
      let v: u64 = match tokenData.parse::<u64>() 
      {
        Ok(v) => v,
        Err(error) if *error.kind() == IntErrorKind::PosOverflow => u64::MAX,
        Err(_) => return Err(format!("Failed to parse UInt: {}", tokenData))
      };
      if v <= u8::MAX as u64       { Ok(FfiArgValue::U8 (v as u8)) }
      else if v <= u16::MAX as u64 { Ok(FfiArgValue::U16(v as u16)) }
      else if v <= u32::MAX as u64 { Ok(FfiArgValue::U32(v as u32)) }
      else                         { Ok(FfiArgValue::U64(v)) }
    }
    TokenType::Int =>
    { // Токен бесконечен, а i64 нет: всё что больше или меньше - граница i64 (#71)
      let v: i64 = match tokenData.parse::<i64>() 
      {
        Ok(v) => v,
        Err(error) if *error.kind() == IntErrorKind::PosOverflow => i64::MAX,
        Err(error) if *error.kind() == IntErrorKind::NegOverflow => i64::MIN,
        Err(_) => return Err(format!("Failed to parse Int: {}", tokenData))
      };
      if v >= i8::MIN as i64 && v <= i8::MAX as i64        { Ok(FfiArgValue::I8 (v as i8)) }
      else if v >= i16::MIN as i64 && v <= i16::MAX as i64 { Ok(FfiArgValue::I16(v as i16)) }
      else if v >= i32::MIN as i64 && v <= i32::MAX as i64 { Ok(FfiArgValue::I32(v as i32)) }
      else                                                 { Ok(FfiArgValue::I64(v)) }
    }
    TokenType::UFloat | TokenType::Float =>
    { // todo F32-аргументы: float-токен всегда уходит как F64, поэтому `libm.sqrtf(16.0)` даёт 0 (ожидается 4)
      let v: f64 = tokenData.parse()
        .map_err(|_| format!("Failed to parse Float: {}", tokenData))?;
      Ok(FfiArgValue::F64(v))
    }
    // todo CString-аргументы: FfiArgValue::CString нигде не создаётся, String уходит как (ptr, len) без NUL,
    // поэтому `libc.strlen("hello")` ненадёжен (ожидается 5, бывает 6)
    TokenType::String => Ok(FfiArgValue::String(tokenData)),
    TokenType::RawString => Ok(FfiArgValue::RawString(tokenData.into_bytes())),
    _ => Err(format!("Unsupported TokenType for FFI arg: {}", tokenDataType.to_string()))
  }
}

/// Приклеивает один `FfiArgValue` к `CallBuilder` через typed `arg::<T>`.
fn pushArg<'a, 'g>(builder: CallBuilder<'a, 'g>, arg: FfiArgValue) -> CallBuilder<'a, 'g>
{
  match arg
  {
    FfiArgValue::U8 (v) => builder.arg::<u8 >(v),
    FfiArgValue::U16(v) => builder.arg::<u16>(v),
    FfiArgValue::U32(v) => builder.arg::<u32>(v),
    FfiArgValue::U64(v) => builder.arg::<u64>(v),
    FfiArgValue::I8 (v) => builder.arg::<i8 >(v),
    FfiArgValue::I16(v) => builder.arg::<i16>(v),
    FfiArgValue::I32(v) => builder.arg::<i32>(v),
    FfiArgValue::I64(v) => builder.arg::<i64>(v),
    FfiArgValue::F64(v) => builder.arg::<f64>(v),
    FfiArgValue::String(v) => builder.arg::<String>(v),
    FfiArgValue::CString(v) => builder.arg::<std::ffi::CString>(v),
    FfiArgValue::RawString(v) => builder.arg::<Vec<u8>>(v)
  }
}

/// Что вызывающий код ждёт от результата FFI-вызова.
///
/// chillffi требует знать тип результата ДО вызова (`.result::<T>()`), потому что от него
/// зависит, из какого регистра ABI читать значение. RTS же узнаёт этот тип из контекста:
///
/// ```text
/// lib.print(x)                  # Discard       — результат не нужен
/// a = libc.strnlen("abc")       # Infer         — тип берётся от правой части
/// a: Usize = libc.strnlen("abc")  # Typed(Usize)  — правая часть приводится к левой
/// ```
#[derive(Clone, PartialEq)]
pub enum FfiExpect
{
  /// Вызов-оператор, результат не используется (`.void()`).
  Discard,
  /// Тип слева не указан: ABI-вызов идёт как `usize`.
  /// 
  /// Итоговый тип структуры выводится из значения — так же, как для `a = 10`
  /// (native/types, issue #63). Usize в результате не бывает: ветка U64
  /// перехватывает значение раньше (structureType.rs).
  Infer,
  /// Тип слева указан: он же является типом возврата C-функции.
  Typed(StructureType)
}

// =================================================================================================

/// Беззнаковое целое -> абстрактный токен UInt;
fn unsignedToken(value: u64) -> Token
{
  Token::new(TokenType::UInt, value.to_string())
}

/// Знаковое целое -> абстрактный токен (как у примитивов: `>= 0` это UInt, `< 0` это Int).
fn signedToken(value: i64) -> Token
{
  if value < 0 {
    Token::new(TokenType::Int,  value.to_string())
  } else {
    Token::new(TokenType::UInt, value.to_string())
  }
}

/// Число с плавающей точкой -> абстрактный токен (`>= 0` это UFloat, `< 0` это Float).
/// `text` — уже отформатированное значение (для F32 оно отличается от F64).
fn floatToken(text: String, negative: bool) -> Token
{
  let mut text: String = text;
  // Rust печатает 4.0 как "4" — возвращаем точку, 
  // чтобы это оставалось числом с плавающей точкой.
  if !text.contains(['.', 'e', 'E', 'N', 'i']) {
    text.push_str(".0");
  }
  if negative {
    Token::new(TokenType::Float,  text)
  } else {
    Token::new(TokenType::UFloat, text)
  }
}

/// Выполняет вызов и превращает результат C-функции в токен по ожиданию `expect`.
fn callWithResult<'a, 'g>(builder: CallBuilder<'a, 'g>, expect: &FfiExpect) -> Result<Token, String>
{
  let error = |e: FFIError| -> String { e.to_string() };

  // Тип слева не указан или "любой" — ABI-вызов идёт как usize (внутренний
  // регистр-read, не итоговый stype() — тот выводится из значения ниже).
  let expectType: &StructureType = match expect
  {
    FfiExpect::Discard =>
    { // Результат не нужен — идём по fire-and-forget ветке.
      builder.void().map_err(error)?;
      return Ok(Token::newEmpty(TokenType::None));
    }
    // todo Infer читает Usize: у узких (8/16/32 бит), знаковых и float результатов старшие биты/регистр не те,
    // для них тип слева обязателен. Не проверено на x86-32 (build.sh 32): там Usize это 32 бита
    FfiExpect::Infer => &StructureType::Usize,
    FfiExpect::Typed(structureType) => match structureType
    {
      StructureType::None | StructureType::Any => &StructureType::Usize,
      // Адрес читаем как число; он живёт только пока жив FFI scope (см. chillffi Value::Pointer).
      // todo Вне [ffi] блока scope временный — полученный адрес после вызова использовать нельзя
      StructureType::Pointer => &StructureType::Usize,
      other => other
    }
  };

  match expectType
  {
    StructureType::U8    => Ok(unsignedToken(builder.result::<u8   >().map_err(error)? as u64)),
    StructureType::U16   => Ok(unsignedToken(builder.result::<u16  >().map_err(error)? as u64)),
    StructureType::U32   => Ok(unsignedToken(builder.result::<u32  >().map_err(error)? as u64)),
    StructureType::U64   => Ok(unsignedToken(builder.result::<u64 >().map_err(error)?)),
    StructureType::Usize => Ok(unsignedToken(builder.result::<usize>().map_err(error)? as u64)),
    StructureType::I8    => Ok(signedToken(builder.result::<i8   >().map_err(error)? as i64)),
    StructureType::I16   => Ok(signedToken(builder.result::<i16  >().map_err(error)? as i64)),
    StructureType::I32   => Ok(signedToken(builder.result::<i32  >().map_err(error)? as i64)),
    StructureType::I64   => Ok(signedToken(builder.result::<i64  >().map_err(error)?)),
    StructureType::Isize => Ok(signedToken(builder.result::<isize>().map_err(error)? as i64)),
    StructureType::F32 =>
    {
      let value: f32 = builder.result::<f32>().map_err(error)?;
      Ok(floatToken(value.to_string(), value.is_sign_negative()))
    }
    StructureType::F64 =>
    {
      let value: f64 = builder.result::<f64>().map_err(error)?;
      Ok(floatToken(value.to_string(), value.is_sign_negative()))
    }
    // todo True/False (issue #65), String/CString/RawString (нужна длина), Custom и т.д.
    other => Err(format!("Unsupported FFI result type: {}", other.to_string()))
  }
}

// =================================================================================================

/// Загружает библиотеку и зовёт её метод через переданный `Scope<'g>`.
///
/// Используется внутри `[ffi] { ... }` блоков, чтобы удерживать один и тот же
/// chillffi scope на всём протяжении блока (Scope Retention).
/// На каждый вызов FFI внутри блока используется один и тот же scope —
/// соответственно, загруженные через `scope.load(...)` библиотеки и
/// `AllocatedMemory` живут, пока живёт блок, и освобождаются при его выходе.
///
/// Возвращает результат C-функции в виде токена; тип результата задаёт `expect`.
pub fn callExternalWithScope<'g>(
  scope: &Scope<'g>,
  libraryPath: &str,
  methodName: &str,
  parametersTokens: &mut [Token],
  expect: &FfiExpect
) -> Result<Token, String>
{
  // Загружаем библиотеку в удерживаемом scope.
  let library: Library<'g> = scope.load(libraryPath)
    .map_err(|e| e.to_string())?;

  // Конвертируем аргументы и приклеиваем к CallBuilder через typed .arg::<T>().
  let mut builder: CallBuilder<'_, 'g> = library.call(methodName);
  for token in parametersTokens.iter_mut()
  {
    let arg: FfiArgValue = tokenToFfiArg(token)?;
    builder = pushArg(builder, arg);
  }

  // Тип результата выбирается диспетчером по ожиданию вызывающего кода.
  callWithResult(builder, expect)
}

/// Старый путь вызова FFI — без удержания scope.
///
/// Создаёт временный scope через `ffi!{...}` макрос на каждый вызов, как и раньше.
/// Используется, когда вызов идёт вне `[ffi]` блока:
/// внутри блока — `callExternalWithScope`, снаружи — `callExternal`.
pub fn callExternal(
  libraryPath: &str,
  methodName: &str,
  parametersTokens: &mut [Token],
  expect: &FfiExpect
) -> Result<Token, String>
{
  // ffi!{} возвращает Result<_, FFIError> — конвертируем в String на выходе.
  // Токен результата — обычные данные (не адрес и не ссылка на память scope),
  // поэтому спокойно живёт после выхода из ffi!{}.
  let result: Result<Token, FFIError> = ffi!(|scope| {
    callExternalWithScope(
      &scope,
      libraryPath,
      methodName,
      parametersTokens,
      expect
    )
    .map_err(|e: String| FFIError::Other(e))
  });
  
  result.map_err(|e| e.to_string())
}

// =================================================================================================

#[cfg(test)]
mod tests
{
  use crate::parser::structure::ffi::bridge::{callExternal, FfiExpect, FfiArgValue, tokenToFfiArg, unsignedToken, signedToken};
  use crate::parser::structure::structureType::StructureType;
  use crate::tokenizer::types::token::Token;
  use crate::tokenizer::types::tokenType::TokenType;
  // ===============================================================================================

  const LibcPath: &str = "libc.so.6";
  const LibmPath: &str = "libm.so.6";

  /// todo desc
  fn call(
    library: &str, 
    method: &str, 
    mut parameters: Vec<Token>, 
    expect: FfiExpect
  ) -> Result<Token, String>
  {
    callExternal(library, method, &mut parameters, &expect)
  }

  /// todo desc
  fn dataOf(token: &Token) -> (TokenType, String)
  {
    (*token.getDataType(), token.getData().toString().unwrap_or_default())
  }

  // ===============================================================================================
  
  /// Результат FFI: целые границы типа не теряются при превращении в токен;
  #[test]
  fn resultTokens() -> ()
  {
    assert!(dataOf(&unsignedToken(u64::MAX)) == (TokenType::UInt, u64::MAX.to_string()));
    assert!(dataOf(&signedToken(i64::MAX)) == (TokenType::UInt, i64::MAX.to_string()));
    assert!(dataOf(&signedToken(i64::MIN)) == (TokenType::Int,  i64::MIN.to_string()));
    assert!(dataOf(&signedToken(0)) == (TokenType::UInt, String::from("0")));
  }

  /// Аргумент FFI: число за пределами u64 и i64 становится границей (#71), а не обрезается;
  #[test]
  fn argBigNumbers() -> ()
  {
    let big: &str = "44444444444444444444444444444444444444444444"; // 44 цифры
    let negBig: String = format!("-{}", big);
    let arg = |tokenType: TokenType, data: &str| tokenToFfiArg(&Token::new(tokenType, data));
    
    // Внутри диапазона размер подбирается под число
    assert!(matches!(arg(TokenType::UInt, "255"), Ok(FfiArgValue::U8(255))));
    assert!(matches!(arg(TokenType::UInt, "65536"), Ok(FfiArgValue::U32(65536))));
    assert!(matches!(arg(TokenType::Int,  "-129"), Ok(FfiArgValue::I16(-129))));
    
    // Больше u64: раньше `v as u64` обрезало 18446744073709551616 до 0
    assert!(matches!(arg(TokenType::UInt, "18446744073709551616"), Ok(FfiArgValue::U64(v)) if v == u64::MAX));
    assert!(matches!(arg(TokenType::UInt, big), Ok(FfiArgValue::U64(v)) if v == u64::MAX));
    
    // Меньше i64
    assert!(matches!(arg(TokenType::Int, "-9223372036854775809"), Ok(FfiArgValue::I64(v)) if v == i64::MIN));
    assert!(matches!(arg(TokenType::Int, &negBig), Ok(FfiArgValue::I64(v)) if v == i64::MIN));
    
    // Не число - ошибка, как и раньше
    assert!(arg(TokenType::UInt, "abc").is_err());
  }

  /// `a: Usize = libc.strnlen("hello world")` — тип слева является типом возврата.
  #[test]
  fn typedUsize() -> ()
  {
    let result: Token = call(
      LibcPath, 
      "strnlen", 
      vec![Token::new(TokenType::String, "hello world")], 
      FfiExpect::Typed(StructureType::Usize)
    ).unwrap();
    assert!(dataOf(&result) == (TokenType::UInt, String::from("11")));
  }

  /// `a = libc.strnlen("hello world")` — тип слева не указан, тип выводится
  /// из величины значения (не Usize) — см. assert ниже: U8, не Usize.
  #[test]
  fn inferUnsigned() -> ()
  {
    let result: Token = call(
      LibcPath, 
      "strnlen", 
      vec![Token::new(TokenType::String, "hello world")], 
      FfiExpect::Infer
    ).unwrap();
    assert!(dataOf(&result) == (TokenType::UInt, String::from("11")));
    assert!(result.clone().getStructureType() == StructureType::U8); // Как у примитива `a = 11`.
  }

  /// Знаковые результаты: `>= 0` это UInt, `< 0` это Int (как у примитивов).
  #[test]
  fn typedSigned() -> ()
  {
    let positive: Token = call(
      LibcPath, 
      "abs", 
      vec![Token::new(TokenType::Int, "-5")], 
      FfiExpect::Typed(StructureType::I32)
    ).unwrap();
    assert!(dataOf(&positive) == (TokenType::UInt, String::from("5")));

    let negative: Token = call(
      LibcPath, 
      "close", 
      vec![Token::new(TokenType::UInt, "999")], 
      FfiExpect::Typed(StructureType::I32)
    ).unwrap();
    assert!(dataOf(&negative) == (TokenType::Int, String::from("-1")));
  }

  /// Узкие и широкие целые: результат читается ровно тем типом, который указан слева.
  #[test]
  fn typedIntegerWidths() -> ()
  {
    let small: Token = call(
      LibcPath, 
      "toupper", 
      vec![Token::new(TokenType::UInt, "97")], 
      FfiExpect::Typed(StructureType::U8)
    ).unwrap();
    assert!(dataOf(&small) == (TokenType::UInt, String::from("65")));

    let wide: Token = call(
      LibcPath, 
      "labs", 
      vec![Token::new(TokenType::Int, "-7")], 
      FfiExpect::Typed(StructureType::I64)
    ).unwrap();
    assert!(dataOf(&wide) == (TokenType::UInt, String::from("7")));
  }

  /// Числа с плавающей точкой: `>= 0` это UFloat, `< 0` это Float, точка сохраняется.
  #[test]
  fn typedFloat() -> ()
  {
    let positive: Token = call(
      LibmPath, 
      "sqrt", 
      vec![Token::new(TokenType::UFloat, "16.0")], 
      FfiExpect::Typed(StructureType::F64)
    ).unwrap();
    assert!(dataOf(&positive) == (TokenType::UFloat, String::from("4.0")));

    let negative: Token = call(
      LibmPath, 
      "floor", 
      vec![Token::new(TokenType::Float, "-2.5")], 
      FfiExpect::Typed(StructureType::F64)
    ).unwrap();
    assert!(dataOf(&negative) == (TokenType::Float, String::from("-3.0")));
  }

  /// Вызов-оператор: результат не нужен, токен пустой.
  #[test]
  fn discard() -> ()
  {
    let result: Token = call(
      LibcPath, 
      "getpid", 
      vec![], 
      FfiExpect::Discard
    ).unwrap();
    assert!(*result.getDataType() == TokenType::None);
  }

  /// Вызов без аргументов.
  #[test]
  fn noParameters() -> ()
  {
    let result: Token = call(
      LibcPath, 
      "getpid", 
      vec![], 
      FfiExpect::Typed(StructureType::I32)
    ).unwrap();
    // chillffi исполняет вызовы в worker-процессе, поэтому pid не равен pid текущего процесса
    assert!(dataOf(&result).1.parse::<i32>().unwrap() > 0);
  }

  /// Типы без поддержки результата и несуществующие методы — ошибка (в выражении станет None).
  #[test]
  fn errors() -> ()
  {
    assert!(call(
      LibcPath, 
      "abs", 
      vec![Token::new(TokenType::Int, "-1")], 
      FfiExpect::Typed(StructureType::String)
    ).is_err());
    
    assert!(call(
      LibcPath, 
      "noSuchFunction", 
      vec![], 
      FfiExpect::Infer
    ).is_err());
  }

  // ===============================================================================================
}
