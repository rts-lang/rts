use super::condition::replaceConditionExpression;
use crate::parser::structure::structure::Structure;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
// =================================================================================================

/// Эта функция обрабатывает линию с `return` и определяет куда пишется результат:
/// 
/// - `:=`: В ближайшую именованную [`Structure`] вверх по области видимости;
/// 
/// - `=`: В текущую [`Structure`].
/// 
pub fn searchReturnLine(
  line: &RwLockReadGuard<Line>,
  structureLink: Arc<RwLock<Structure>>
) -> bool
{
  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

  let mut lineTokens: Vec<Token> =
  { // Читаемая линия, на которой мы сейчас находимся.
    match line.tokens {
      Some(ref tokens) => tokens.clone(), // Токены линии, на которой мы сейчас находимся.
      None => return false // Если в линии нет токенов, то мы её не читаем.
    }
  };
  if lineTokens.is_empty() {
    return false // Если в линии нет токенов, то мы её не читаем.
  }

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
  
  // Это `return` (`:=`) или результат блока (`=`)?
  let firstType: TokenType = *lineTokens[0].getDataType();
  if !matches!(firstType, TokenType::Equals | TokenType::Creation) {
    return false
  }

  // Удаляем первый токен (`=` или `:=`), чтобы он не мешался.
  lineTokens.remove(0);

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

  // Редактируемый родитель, поскольку мы собираемся присвоить значение его `result`.
  let newResultData: Token =
  {
    // Блок-выражение `= ? True { = 1 }`: `?` + условие заменяются результатом;
    //
    // Блоки выполняются без удержания блокировки на структуре.
    if let Some(replaced) = 
      replaceConditionExpression(&lineTokens, &line.lines, &structureLink) {
        lineTokens = replaced;
      }
    
    // Используем `expression`, чтобы получить результат выражения.
    let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
    structure.expression(&mut lineTokens)
  };

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

  drop(lineTokens); // Больше не нужно.
  
  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

  // Ищем куда писать `result`:
  //
  // - `:=` => Возврат результата только именной `Structure`.
  //           Поднимаемся через `.parent` до первого `name.is_some()`.
  //
  // - `=` => Пишем в текущую `structureLink` - т.е. текущую `Structure`.
  //
  let targetLink: Arc<RwLock<Structure>> = if firstType == TokenType::Creation
  {
    let mut currentLink: Arc<RwLock<Structure>> = structureLink.clone();
    loop
    {
      let current: RwLockReadGuard<Structure> = currentLink.read().unwrap();
      
      // Проверка на наличие имени у `Structure`.
      let hasName: bool = current.name.is_some();
      if hasName {
        break
      }
      
      // Берём нового родителя.
      let parent: Option< Arc<RwLock<Structure>> > = current.parent.clone();
      drop(current);
      
      match parent
      { // Поднимаемся выше, текущая `Structure` тоже была без имени.
        Some(parent) => currentLink = parent,
        // Дошли до самого верха, ничего выше `main` нет — `:=` игнорируется.
        None => return false
      };
    }
    currentLink
  } else {
    structureLink.clone()
  };

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

  let mut structure: RwLockWriteGuard<Structure> = targetLink.write().unwrap();

  // Структура ожидает какой-то тип в результате,
  // либо это может быть `TokenType::None`. Но мы просто будем менять `data`.

  if structure.result.is_some()
  { // Вариант, в котором результат ожидает возвращение определённого типа данных.
    if let Some(structureResult) = &mut structure.result
    { // Присваиваем новую data результату.
      structureResult.setData( newResultData.getData() );
    }
  } else
  { // Вариант, в котором тип результата был не указан;
    //
    // Используем `expression`, чтобы получить результат выражения;
    //
    // Присваиваем новый результат.
    structure.result = Some(newResultData);
  }

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

  // Всё успешно, это был результат.
  true

  // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
}

// =================================================================================================
