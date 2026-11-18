use super::nested::searchNestedStructure;
use crate::parser::readStructureLines;
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

// todo Тут описание о том, что тут будут теги в будущем,
//  но пока что только [ffi], но логика подготовлена на будущее.

// =================================================================================================

/// Проверяет, является ли [`Token`] тегом `[ffi]`.
/// 
/// Токен должен иметь тип [`TokenType::SquareBracketBegin`] и
/// хранить одну вложенную линию с токеном `ffi` типа [`TokenType::Word`];
/// 
/// Возвращаем `true`, если это так.
//
// todo Когда появятся другие теги (`async`, `thread` и т.д.) — расширить.
pub fn isFfiTagToken(token: &Token) -> bool
{
  // todo desc
  if *token.getDataType() != TokenType::SquareBracketBegin {
    return false;
  }

  // todo desc
  let lines: &Vec< Arc<RwLock<Line>> > = if let Some(lines) =
    token.lines.as_ref() { lines }
    else { return false };

  // todo desc
  for lineLink in lines
  {
    // todo desc
    let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
    let tokens: &[Token] = if let Some(tokens) =
      line.tokens.as_ref() { tokens }
      else { continue };

    // todo desc
    for token in tokens
    {
      if *token.getDataType() == TokenType::Word &&
        token.getData().toString().unwrap_or_default() == "ffi"
      {
        return true;
      }
    }
  }
  
  //
  false
}

// =================================================================================================

/// Ищет тег `[ffi]` в начале линии и создаёт помеченную структуру:
///
/// - Анонимная `[ffi] { ... }` — создаёт структуру, ставит `isFfiBlock`,
///   сразу читает тело.
/// 
/// - Именная `[ffi] name { ... }` / `[ffi] name() { ... }` —
///   отдаёт хвост в `searchNestedStructure` с `isFfi = true`.
///
/// Тег и тело могут быть на одной линии или на соседних.
///
/// `true` — линия обработана; `false` — это не `[ffi]`.
pub fn searchFfiStructure(
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
  let mut lineLines: Option<Vec< Arc<RwLock<Line>> >> = line.lines.clone();
  
  // Если в линии нет токенов, то мы её не читаем.
  if lineTokensLength == 0 {
    return false;
  }
  // Тип первого токена в строке.
  let firstTokenType: &TokenType = lineTokens[0].getDataType();
  
  // -----------------------------------------------------------------------------------------------
  
  // Обработка тега `[ffi]`:
  //
  // - `[ffi] test() { ... }`: Метод с FFI-тегом.
  //
  // - `[ffi] name { ... }`: Именованный FFI-блок.
  //
  // - `[ffi] { ... }`: Анонимный FFI-блок (на одной строке).
  //
  // - `[ffi]`
  //   `{ ... }`: Анонимный FFI-блок (тело ниже, как в condition).
  //
  // - `[ffi]`
  //   `name { ... }`: Именованный FFI-блок (тело ниже, как в condition).
  //
  // - `[ffi]`
  //   `name() { ... }`: Метод с FFI-тегом (тело ниже, как в condition).
  //
  //
  // Механизм как у `condition`:
  //
  // 1) Первый токен — SquareBracketBegin, внутри которого `ffi`;
  //
  // 2) Если после тега нет токенов и нет `body` на этой линии —
  //    смотрим следующую линию (как `collectConditionBranches`):
  //    если у неё пустые токены — берём `body` оттуда (анонимный блок),
  //    если у неё имя и есть вложение — создаём её сразу как FFI-структуру;
  //    в обоих случаях сдвигаем `lineIndex`;
  //
  // 3) Дальше единая обработка: `rest` + `body`.
  //
  if *firstTokenType == TokenType::SquareBracketBegin
  {
    // Проверяем, что внутри скобок именно "ffi" (а не любой тег).
    let isFfiTag: bool = isFfiTagToken(&lineTokens[0]);
    if isFfiTag
    {
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      // Достаём токены после тега `[ffi]`.
      let restTokens: &[Token] = &lineTokens[1..];
      let restTokensLength: usize = restTokens.len();

      // Если `body` нет на этой линии и rest пуст — ищем тело ниже (как в `condition`).
      if restTokensLength == 0 && lineLines.is_none()
      {
        // Следующая линия — это либо анонимный блок `{ ... }`,
        // либо именованная структура `name { ... }` / `name() { ... }`.
        let nextLine: Option<( Vec<Token>, Option< Vec< Arc<RwLock<Line>> > > )> =
        {
          let parent: RwLockReadGuard<Structure> = structureLink.read().unwrap();
          let start: usize = parent.lineIndex.get();
          match parent.lines.as_ref().and_then(|lines| lines.get(start + 1))
          {
            Some(nextLink) =>
            {
              let nextLine: RwLockReadGuard<Line> = nextLink.read().unwrap();
              Some((nextLine.tokens.clone().unwrap_or_default(), nextLine.lines.clone()))
            }
            None => None
          }
        };

        if let Some((nextTokens, nextLines)) = nextLine
        {
          // Анонимный блок: токенов нет, но есть вложение.
          if nextTokens.is_empty() && nextLines.is_some()
          {
            lineLines = nextLines;
            // Сдвигаем `lineIndex`: эту линию мы прочитали сами.
            let parent: RwLockReadGuard<Structure> = structureLink.read().unwrap();
            parent.lineIndex.set(parent.lineIndex.get() + 1);
          }
          // Именованная структура: имя в начале и есть вложение.
          else if !nextTokens.is_empty() && nextLines.is_some() &&
            *nextTokens[0].getDataType() == TokenType::Word
          {
            // Сдвигаем `lineIndex`: эту линию мы прочитали сами.
            {
              let parent: RwLockReadGuard<Structure> = structureLink.read().unwrap();
              parent.lineIndex.set(parent.lineIndex.get() + 1);
            }
            // Создаём её сразу как FFI-структуру.
            return searchNestedStructure(
              &nextTokens,
              nextTokens.len(),
              nextLines.unwrap(),
              structureLink,
              true
            );
          }
        }
      }

      // Без body дальше не идём.
      let Some(innerLines) = lineLines else {
        return false;
      };

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      // Случай A: анонимный блок `[ffi] { ... }` (или `[ffi]` + тело ниже).
      //
      // Тег — единственный токен, и блок сразу же выполняется.
      if restTokensLength == 0
      {
        let newStructureLink: Arc<RwLock<Structure>> = 
          Arc::new(RwLock::new(Structure::new(
            None, // Анонимный.
            StructureMut::Constant,
            StructureType::Primitive(PrimitiveType::Method),
            Some(innerLines),
            Some(structureLink.clone())
          )));
        // `isFfiBlock=true` уже выставлен дефолтом (false), но пометим явно для ясности.
        newStructureLink.write().unwrap().isFfiBlock = true;

        structureLink.write().unwrap()
          .pushStructure(newStructureLink.clone());

        // `readStructureLines` сам пушнет scope, потому что `isFfiBlock=true`.
        // Здесь не зовём `enterFfiBlock`/`exitFfiBlock` явно — будет двойной push.
        readStructureLines(newStructureLink);
        return true;
      }

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      // Случай B: `[ffi] <Word> ...` — именная структура на той же строке.
      //
      // Собирает её `searchNestedStructure`. Флаг передаём явно:
      // тег стоит на этой же линии, поэтому предыдущую строку смотреть не нужно.
      if restTokensLength >= 1 &&
         *restTokens[0].getDataType() == TokenType::Word
      {
        return searchNestedStructure(
          restTokens,
          restTokensLength,
          innerLines,
          structureLink,
          true
        );
      }

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      // Неизвестная форма после `[ffi]` — падаем в основной парсер, чтобы
      // он попробовал обработать как структуру (и корректно ругнулся, если
      // это не структура).

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    }
  }

  // -----------------------------------------------------------------------------------------------
  
  false
  
  // -----------------------------------------------------------------------------------------------
}

// =================================================================================================
