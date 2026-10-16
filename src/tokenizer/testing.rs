use crate::tokenizer::tokenizer::readTokensSimple;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::mem;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
// =================================================================================================

// Общие помощники для `tests` токенайзера.
//
// todo desc надо или нет?

// =================================================================================================

/// Получает `bytes` -> выдает token types для проверки в тестах.
pub(crate) fn getTokensFromBuffer(src: &str) -> Vec<Token>
{
  let mut buffer: Vec<u8> = src.as_bytes().to_vec();
  let lines: Vec< Arc<RwLock<Line>> > = readTokensSimple(&mut buffer);

  let mut types: Vec<Token> = Vec::new();
  for lineLink in lines
  {
    let mut line: RwLockWriteGuard<Line> = lineLink.write().unwrap();
    if let Some(tokens) = &mut line.tokens
    {
      let taken: Vec<Token> = mem::take(tokens); // Изымаем, т.к. нужно только нам.
      for token in taken
      {
        types.push(token);
      }
      //
    }
  }

  types
}

// =================================================================================================

/// Проверяет типы токенов: сколько их получилось и какой тип у каждого, по порядку.
///
/// Пробелы стираются, поэтому сверяются только типы (без значений).
/// 
/// Токены читаются по всем линиям первого уровня подряд.
pub(crate) fn checkTokensTypes(code: &str, expected: &[TokenType]) -> ()
{
  let actual: Vec<String> = getTokensFromBuffer(code).iter()
    .map(|token: &Token| token.getDataType().to_string())
    .collect();
  let expected: Vec<String> = expected.iter()
    .map(|tokenType: &TokenType| tokenType.to_string())
    .collect();
  assert_eq!(actual, expected,
             "Bytes '{}' should have created tokens with different types", code.replace('\n', "\\n"));
}

/// Проверяет разделение пробелами на несколько токенов: `checkTokensTypes` для набора кейсов.
pub(crate) fn checkSplit(cases: &[(&str, &[TokenType])])
{
  for (code, expected) in cases
  {
    checkTokensTypes(code, expected);
  }
}

// =================================================================================================

/// Проверяем тип и значение.
pub(crate) fn checkValues<const N: usize>(cases: [(&str, TokenType); N], checkData: bool)
{
  for (src, expectedType) in cases
  {
    let tokens: Vec<Token> = getTokensFromBuffer(src);

    // Должен быть 1 токен
    let tokensLen: usize = tokens.len();
    assert_eq!(tokensLen, 1,
               "Bytes '{}' should have created 1 token, but created {}:{:?}", src, tokensLen, tokens);

    // Тип должен совпадать
    let tokenType: String = tokens[0].getDataType().to_string();
    let expectedTokenType: String = expectedType.to_string();
    assert_eq!(tokenType, expectedTokenType,
               "Bytes '{}' should have returned type '{}', but returned '{}'", src, expectedTokenType, tokenType);

    if checkData 
    { // Значение должно совпадать с изначальным
      let tokenData: String = tokens[0].to_string();
      assert_eq!(tokenData, src,
                 "Expected original bytes '{}', but got '{}':'{}'", src, tokenData, tokenType);
    }
  }
  //
}

// =================================================================================================

/// Проверяет через несколько токенов.
pub(crate) fn checkThroughOthers<const N: usize>(cases: [(&str, &str, &str, TokenType); N])
{
  for (input, name, op, typ) in cases
  {
    let tokens: Vec<Token> = getTokensFromBuffer(input);

    //
    let tokensLen: usize = tokens.len();
    assert_eq!(tokensLen, 3,
               "Bytes '{}' should have created 3 tokens, but created {}:{:?}", input, tokensLen, tokens);

    //
    let t0: String = tokens[0].to_string();
    assert_eq!(t0, name,
               "Bytes '{}' should have created '{}' in the first token, but created '{}'", input, name, t0);

    //
    let t1: String = tokens[1].to_string();
    assert_eq!(t1, op,
               "Bytes '{}' should have created '{}' in the second token, but created '{}'", input, op, t1);

    //
    let t2: String = tokens[2].getDataType().to_string();
    let expectedT2Type: String = typ.to_string();
    assert_eq!(t2, expectedT2Type,
               "Bytes '{}' should have created '{}' from the third token, but created '{}'", input, expectedT2Type, t2);
  }
}

// =================================================================================================
