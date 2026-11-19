use super::condition::executeConditionStatement;
use super::linear::linearLine;
use super::nested::searchNestedStructure;
use crate::parser::structure::structure::Structure;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

/// Проверяет, что переданный `dataType` является математическим оператором.
//
// todo Есть проблема в том, что `=` и `:=` не совсем математические операторы.
//  Возможно, что метод не должен проверять математические, а бинарные?
pub(super) const fn isMathOperator(dataType: TokenType) -> bool
{
  matches!(dataType,
    // `=` — Присваивание по области видимости (ищет вверх);
    // `:=` — Создание в текущей области (вверх не ищет).
    TokenType::Equals | TokenType::Creation |
    TokenType::PlusEquals     | // +=
    TokenType::MinusEquals    | // -=
    TokenType::MultiplyEquals | // *=
    TokenType::DivideEquals     // /=
    // TokenType::ModuloEquals   | // %=
    // TokenType::ExponentEquals | // ^=
    // TokenType::UnaryPlus      | // ++
    // TokenType::UnaryMinus     | // --
  )
}

// =================================================================================================

/// Ищет структуры; Это может быть:
/// 
/// - Вложенная структура ⟶ [`searchNestedStructure`];
/// 
/// - Линейное выражение ⟶ [searchLinearStructure`];
/// 
/// - Условная структура ⟶ [`executeConditionStatement`].
//
// todo Я всё ещё думаю, что его стоит переименовать. Потому что он ищет структуры да,
//  но не совсем так. По сути он ищет последовательности, которые приводят к их созданию.
//  или `линейное выражение` - это фактически создание/op над структурой.
//  Но назвать `expression` это нельзя, хотя может быть в какой-то мере и можно, ведь
//  мы ищем именно последовательность токенов, обрабатываем и получаем результат.
pub fn searchStructure(
  line: &RwLockReadGuard<Line>,
  structureLink: Arc<RwLock<Structure>>
) -> bool
{
  // -----------------------------------------------------------------------------------------------
    
  // Ссылка на токены линии.
  let lineTokens: &Vec<Token> = match &line.tokens {
    Some(tokens) => tokens,
    None => return false // Если в линии нет токенов, то мы её не читаем.
  };
  
  // Размер токенов линии.
  let lineTokensLength: usize = lineTokens.len();
  
  // Вложенные линии.
  let lineLines: Option<Vec< Arc<RwLock<Line>> >> = line.lines.clone();
  
  // Если в линии нет токенов, то это может быть только анонимная
  // вложенная структура `{ ... }`: токенов нет, но вложение есть.
  if lineTokensLength == 0
  {
    // Безымянную структуру вызвать нечем, поэтому тело читается сразу:
    // иначе код внутри останется мёртвым. Путь один - вложенные структуры
    // (`searchNestedStructure`); `[ffi] { ... }` (`tags`) идёт в него же
    // с маркером `isFfiBlock`.
    return if let Some(lineLines) = lineLines {
      searchNestedStructure(lineTokens, lineTokensLength, lineLines, structureLink, false)
    } else
    { // Токенов и вложений нет — читать нечего.
      false
    };
  }
  // Тип первого токена в строке.
  let firstTokenType: &TokenType = lineTokens[0].getDataType();
  
  // -----------------------------------------------------------------------------------------------

  if *firstTokenType == TokenType::Word
  { // Если мы видим `TokenType::Word` в начале строки, 
    // это значит, что это либо структура, либо линейное выражение.

    { // Если в токенах есть оператор `:=` или `=` — это линейное выражение,
      // даже если есть вложенные линии. Например: `a := ? True { = 20 }`.
      let hasOperator: bool = lineTokens.iter()
        .any(|t| isMathOperator(*t.getDataType()));
      if hasOperator
      { // Линейное выражение с оператором.
        return linearLine(lineTokens, &lineLines, structureLink);
      }
    }

    return if let Some(lineLines) = lineLines
    { // Если в линии есть вложение, то это структура с вложением.
      searchNestedStructure(lineTokens, lineTokensLength, lineLines, structureLink, false)
    } else
    { // Это линейное выражение.
      linearLine(lineTokens, &lineLines, structureLink)
    }
  } else
  
  if *firstTokenType == TokenType::Question && !lineLines.is_none()
  { // В том случае, если это не структура и не линейное выражение, 
    // мы видим `TokenType::Question` в начале строки и
    // есть вложения у этой линии, то это условное вложение.
    return executeConditionStatement(lineTokens, &lineLines, &structureLink);
  }

  // -----------------------------------------------------------------------------------------------
  
  false

  // -----------------------------------------------------------------------------------------------
}

// =================================================================================================
