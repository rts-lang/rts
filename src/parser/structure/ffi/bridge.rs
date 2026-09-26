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
      indent: None,
      lines: None,
      parent: None
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
      indent: None,
      lines: None,
      parent: None
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
    {
      let v: u128 = tokenData.parse()
        .map_err(|_| format!("Failed to parse UInt: {}", tokenData))?;
      if v <= u8::MAX as u128       { Ok(FfiArgValue::U8 (v as u8)) }
      else if v <= u16::MAX as u128 { Ok(FfiArgValue::U16(v as u16)) }
      else if v <= u32::MAX as u128 { Ok(FfiArgValue::U32(v as u32)) }
      else                          { Ok(FfiArgValue::U64(v as u64)) }
    }
    TokenType::Int =>
    {
      let v: i128 = tokenData.parse()
        .map_err(|_| format!("Failed to parse Int: {}", tokenData))?;
      if v >= i8::MIN as i128 && v <= i8::MAX as i128       { Ok(FfiArgValue::I8 (v as i8)) }
      else if v >= i16::MIN as i128 && v <= i16::MAX as i128 { Ok(FfiArgValue::I16(v as i16)) }
      else if v >= i32::MIN as i128 && v <= i32::MAX as i128 { Ok(FfiArgValue::I32(v as i32)) }
      else                                                   { Ok(FfiArgValue::I64(v as i64)) }
    }
    TokenType::UFloat | TokenType::Float =>
    {
      let v: f64 = tokenData.parse()
        .map_err(|_| format!("Failed to parse Float: {}", tokenData))?;
      Ok(FfiArgValue::F64(v))
    }
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

/// Загружает библиотеку и зовёт её метод через переданный `Scope<'g>`.
///
/// Используется внутри `@ffi { ... }` блоков, чтобы удерживать один и тот же
/// chillffi scope на всём протяжении блока (Scope Retention из PR #31).
/// На каждый вызов FFI внутри блока используется один и тот же scope —
/// соответственно, загруженные через `scope.load(...)` библиотеки и
/// `AllocatedMemory` живут, пока живёт блок, и освобождаются при его выходе.
pub fn callExternalWithScope<'g>(
  scope: &Scope<'g>,
  libraryPath: &str,
  methodName: &str,
  parametersTokens: &mut [Token],
  _resultType: StructureType
) -> Result<(), String>
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

  // На текущей стадии интеграции результат вызова не используется
  // (call-сайт в Structure::expression() всегда игнорирует возвращаемое
  // значение — см. `Ok(_result) => { /* todo Обработка result */ }`).
  // Поэтому идём по fire-and-forget ветке `.void()`.
  // todo Когда появится нормальная обработка результата — выбирать
  //  `.result::<T>()` по `_resultType` через диспетчер.
  builder.void().map_err(|e| e.to_string())?;

  Ok(())
}

/// Старый путь вызова FFI — без удержания scope.
///
/// Создаёт временный scope через `ffi!{...}` макрос на каждый вызов, как и раньше.
/// Используется для обратной совместимости, когда вызов идёт вне `@ffi {}` блока:
/// внутри блока — `callExternalWithScope`, снаружи — `callExternal`.
pub fn callExternal(
  libraryPath: &str,
  methodName: &str,
  parametersTokens: &mut [Token],
  resultType: StructureType
) -> Result<(), String>
{
  // Используем ffi!{} макрос с замыканием, принимающим scope.
  // Внутри замы scope живёт ровно столько, сколько и сам ffi!{} блок.
  // ffi!{} возвращает Result<_, FFIError> — конвертируем в String на выходе.
  // Замы обязано вернуть Ok(_), поэтому оборачиваем callExternalWithScope в Ok.
  let result: Result<(), FFIError> = ffi!(|scope| {
    Ok::<_, FFIError>(
      callExternalWithScope(&scope, libraryPath, methodName, parametersTokens, resultType)
        .map_err(|e: String| FFIError::Other(e))?
    )
  });
  result.map_err(|e| e.to_string())
}

// =================================================================================================