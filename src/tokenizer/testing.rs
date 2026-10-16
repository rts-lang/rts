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

/// Вспомогательная функция: 
/// Получает bytes -> выдает token types для проверки в тестах
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
      let taken: Vec<Token> = mem::take(tokens); // Изымаем, т.к. нужно только нам
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

/// Вспомогательная функция:
/// Проверяет типы токенов: сколько их получилось и какой тип у каждого, по порядку.
///
/// Пробелы стираются, поэтому сверяются только типы (без значений).
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
             "Байты '{}' должны были создать токены с другими типами", code.replace('\n', "\\n"));
}

/// Вспомогательная функция:
/// Проверяет разделение пробелами на несколько токенов: `checkTokensTypes` для набора кейсов.
pub(crate) fn checkSplit(cases: &[(&str, &[TokenType])])
{
  for (code, expected) in cases
  {
    checkTokensTypes(code, expected);
  }
}

// =================================================================================================

/// Вспомогательная функция: 
/// Проверяем тип и значение
pub(crate) fn checkValues<const N: usize>(cases: [(&str, TokenType); N], checkData: bool)
{
  for (src, expectedType) in cases
  {
    let tokens: Vec<Token> = getTokensFromBuffer(src);

    // Должен быть 1 токен
    let tokensLen: usize = tokens.len();
    assert_eq!(tokensLen, 1,
               "Байты '{}' должны были создать 1 токен, а создали {}:{:?}", src, tokensLen, tokens);

    // Тип должен совпадать
    let tokenType: String = tokens[0].getDataType().to_string();
    let expectedTokenType: String = expectedType.to_string();
    assert_eq!(tokenType, expectedTokenType,
               "Байты '{}' должны были вернуть тип '{}', а вернули '{}'", src, expectedTokenType, tokenType);

    if checkData 
    { // Значение должно совпадать с изначальным
      let tokenData: String = tokens[0].to_string();
      assert_eq!(tokenData, src,
                 "Ожидались исходные байты '{}', а получили '{}':'{}'", src, tokenData, tokenType);
    }
  }
  //
}

// =================================================================================================

/// Вспомогательная функция: 
/// Проверяет через несколько токенов
pub(crate) fn checkThroughOthers<const N: usize>(cases: [(&str, &str, &str, TokenType); N]) 
{
  for (input, name, op, typ) in cases
  {
    let tokens: Vec<Token> = getTokensFromBuffer(input);

    //
    let tokensLen: usize = tokens.len();
    assert_eq!(tokensLen, 3,
               "Байты '{}' должны были создать 3 токена, а создали {}:{:?}", input, tokensLen, tokens);

    //
    let t0: String = tokens[0].to_string();
    assert_eq!(t0, name,
               "Байты '{}' должны были создать в первом токене '{}', а создали '{}'", input, name, t0);

    //
    let t1: String = tokens[1].to_string();
    assert_eq!(t1, op,
               "Байты '{}' должны были создать во втором токене '{}', а создали '{}'", input, op, t1);

    //
    let t2: String = tokens[2].getDataType().to_string();
    let expectedT2Type: String = typ.to_string();
    assert_eq!(t2, expectedT2Type,
               "Байты '{}' должны были создать из третьего токена '{}', а создали '{}'", input, expectedT2Type, t2);
  }
}

// =================================================================================================
