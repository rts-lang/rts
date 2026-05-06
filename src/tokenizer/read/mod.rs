pub(super) mod numbers;
pub(super) mod words;
pub(super) mod operators;
pub(super) mod quotes;
pub(super) mod comments;

// =================================================================================================

#[cfg(test)]
mod tests
{
  use std::mem;
  use std::sync::{Arc, RwLock, RwLockWriteGuard};
  use crate::tokenizer::types::line::Line;
  use crate::tokenizer::types::token::{Token, TokenType};

  // ===============================================================================================
  
  /// Получает bytes -> выдает token types для проверки в тестах
  pub(super) fn getTokensFromBuffer(src: &str) -> Vec<Token>
  {
    let buffer: Vec<u8> = src.as_bytes().to_vec();
    let lines: Vec< Arc<RwLock<Line>> > = crate::tokenizer::tokenizer::readTokens(buffer, false);

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

  // ===============================================================================================

  /// Проверяем тип и значение
  pub(super) fn checkValues<const N: usize>(cases: [(&str, TokenType); N], checkData: bool)
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

  // ===============================================================================================

  /// Проверяет разделение пробелами на несколько токенов
  pub(super) fn checkSplit(cases: &[(&str, &[TokenType])]) 
  {
    for (input, expected_types) in cases 
    {
      let tokens: Vec<Token> = getTokensFromBuffer(input);
      assert_eq!(
        tokens.len(),
        expected_types.len(),
        "Байты '{}' должны были создать {} токенов, а создали {}:{:?}",
        input,
        expected_types.len(),
        tokens.len(),
        tokens
      );

      for (i, (token, expectedType)) in tokens.iter().zip(expected_types.iter()).enumerate() 
      {
        let tokenType: String = token.getDataType().to_string();
        let expectedType: String = expectedType.to_string();
        assert_eq!(
          tokenType,
          expectedType,
          "Байты '{}' создали токен '{}' с типом '{}', а ожидался '{}'",
          input, i, tokenType, expectedType
        );
        //
      }
    }
    //
  }

  // ===============================================================================================

  /// Проверяет через несколько токенов
  pub(super) fn checkThroughOthers<const N: usize>(cases: [(&str, &str, &str, TokenType); N]) 
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

  // ===============================================================================================
}

// =================================================================================================