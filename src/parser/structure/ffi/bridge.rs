use chillffi::{call, ffi};
use chillffi::ffi::errors::FFIError;
use chillffi::ffi::value::{Pointer, Primitive, Type, Value};
use crate::parser::bytes::Bytes;
use crate::parser::structure::structureType::StructureType;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

// todo Для расширения FFI остается сделать еще:
//  1. Блоки с ffi {}, включая результат и работа внутри.
//     Все сводится как в chillffi - только FFI работают с FFI scope, остальное просто RTS код.
//  2. Использование RawString, CString, String - должны быть примитивами по идее.
//     И для этого есть специальные заранее сделанные поля например pointer+len для String примитива.
//     (вроде как уже что-то было подготовлено для этого и уже сейчас возможно).

// =================================================================================================

/// Token -> chillffi::Value
pub fn tokenToValue(token: &mut Token) -> Result<Value, String>
{
  let tokenDataType: &TokenType = token.getDataType();
  let tokenData: String = token.getData().toString()
    .ok_or_else(|| "Token data is empty".to_owned())?;

  match tokenDataType
  {
    TokenType::UInt =>
    {
      let v: u128 = tokenData.parse().map_err(|_| format!("Failed to parse UInt: {}", tokenData))?;
      if v <= u8::MAX as u128       { Ok(Value::U8(v as u8)) }
      else if v <= u16::MAX as u128 { Ok(Value::U16(v as u16)) }
      else if v <= u32::MAX as u128 { Ok(Value::U32(v as u32)) }
      else if v <= u64::MAX as u128 { Ok(Value::U64(v as u64)) }
      else { Err(format!("UInt out of range: {}", v)) }
    }
    TokenType::Int =>
    {
      let v: i128 = tokenData.parse().map_err(|_| format!("Failed to parse Int: {}", tokenData))?;
      if v >= i8::MIN as i128 && v <= i8::MAX as i128   { Ok(Value::I8(v as i8)) }
      else if v >= i16::MIN as i128 && v <= i16::MAX as i128 { Ok(Value::I16(v as i16)) }
      else if v >= i32::MIN as i128 && v <= i32::MAX as i128 { Ok(Value::I32(v as i32)) }
      else if v >= i64::MIN as i128 && v <= i64::MAX as i128 { Ok(Value::I64(v as i64)) }
      else { Err(format!("Int out of range: {}", v)) }
    }
    TokenType::UFloat | TokenType::Float =>
    {
      let v: f64 = tokenData.parse().map_err(|_| format!("Failed to parse Float: {}", tokenData))?;
      Ok(Value::F64(v))
    }
    TokenType::String => Ok(Value::RawString(tokenData.into_bytes())),
    _ => Err("Unsupported TokenType".to_owned()),
  }
}

/// chillffi::Value -> Token.
pub fn valueToToken(value: Value) -> Token
{
  let (tokenDataType, bytes): (TokenType, Bytes) = match value
  {
    Value::U8(v) => (TokenType::UInt, Bytes::from(v.to_string())),
    Value::U16(v) => (TokenType::UInt, Bytes::from(v.to_string())),
    Value::U32(v) => (TokenType::UInt, Bytes::from(v.to_string())),
    Value::U64(v) => (TokenType::UInt, Bytes::from(v.to_string())),
    Value::I8(v) => (TokenType::Int, Bytes::from(v.to_string())),
    Value::I16(v) => (TokenType::Int, Bytes::from(v.to_string())),
    Value::I32(v) => (TokenType::Int, Bytes::from(v.to_string())),
    Value::I64(v) => (TokenType::Int, Bytes::from(v.to_string())),
    Value::F32(v) => (TokenType::Float, Bytes::from(v.to_string())),
    Value::F64(v) => (TokenType::Float, Bytes::from(v.to_string())),
    Value::RawString(v) => (TokenType::String, Bytes::from(v)),
    _ => (TokenType::None, Bytes::empty()),
  };
  Token::new(tokenDataType, bytes)
}

/// Единственное место, где используется ffi!{}.
pub fn callExternal(
  libraryPath: &str,
  methodName: &str,
  parametersTokens: &mut [Token],
  resultType: StructureType,
) -> Result<Token, String>
{
  let argsVec: Vec<Value> = parametersTokens.iter_mut()
    .map(tokenToValue)
    .collect::<Result<Vec<_>, _>>()?;

  let libPathOwned: String = libraryPath.to_string();
  let methodOwned: String = methodName.to_string();

  let result: Value = ffi!{
    let library: Library = Library::load(&libPathOwned)?;

    // T зависит от рантайм-значения resultType — generic-вывод тут невозможен,
    // поэтому match вместо одного call!/.call::<T>() вызова.
    //
    // todo Возможно стоит сделать что-то вроде цепочки вызовов - тогда можно отказать от макроса
    //   в пользу .arg() и также передать если нам надо преобразование или нет. Т.е. 2 выхода 
    //   можно построить как .a() и .b() но они будут возвращать разное.
    //   потому что по сути тут 1 лишнее преобразование, хотя я могу ошибаться.
    //   ну и + это вручную все - а могло быть не вручную?
    let value: Value = match resultType 
    {
      StructureType::None => { library.call::<()>(&methodOwned, argsVec)?; Value::None }
      StructureType::U8 => library.call::<u8>(&methodOwned, argsVec)?.toValue(),
      StructureType::U16 => library.call::<u16>(&methodOwned, argsVec)?.toValue(),
      StructureType::U32 => library.call::<u32>(&methodOwned, argsVec)?.toValue(),
      StructureType::U64 => library.call::<u64>(&methodOwned, argsVec)?.toValue(),
      StructureType::Usize => library.call::<usize>(&methodOwned, argsVec)?.toValue(),
      StructureType::I8 => library.call::<i8>(&methodOwned, argsVec)?.toValue(),
      StructureType::I16 => library.call::<i16>(&methodOwned, argsVec)?.toValue(),
      StructureType::I32 => library.call::<i32>(&methodOwned, argsVec)?.toValue(),
      StructureType::I64 => library.call::<i64>(&methodOwned, argsVec)?.toValue(),
      StructureType::Isize => library.call::<isize>(&methodOwned, argsVec)?.toValue(),
      StructureType::F32 => library.call::<f32>(&methodOwned, argsVec)?.toValue(),
      StructureType::F64 => library.call::<f64>(&methodOwned, argsVec)?.toValue(),
      StructureType::Bool => library.call::<u8>(&methodOwned, argsVec)?.toValue(), // как в structureTypeToChillType
      StructureType::Pointer => library.call::<Pointer>(&methodOwned, argsVec)?.toValue(),
      other => return Err(FFIError::Other(format!("Unsupported FFI type: {}", other.to_string()))),
    };

    Ok(value)
  }.map_err(|e| e.to_string())?;

  Ok(valueToToken(result))
}

// =================================================================================================