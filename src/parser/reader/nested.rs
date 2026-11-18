use crate::parser::bytes::Bytes;
use crate::parser::readStructureLines;
use crate::parser::structure::primitives::isPrimitiveName;
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

/// Ищет вложенную [`Structure`]; Это линия, у которой есть вложенные линии.
///
/// Например:
/// ```ignore
/// name {
///   ...
/// }
/// ```
pub fn searchNestedStructure(
  lineTokens: &[Token],
  lineTokensLength: usize,
  lineLines: Vec< Arc<RwLock<Line>> >,
  parentLink: Arc<RwLock<Structure>>,
  isFfi: bool
) -> bool
{
  // Если в линии есть вложение, то это структура с вложением.
  if let Some(newStructureName) = lineTokens[0].getData().toString()
  { // Здесь первый токен - имя структуры.
    
    // Получаем имя структуры.
    let mut newStructureResultType: Option<&TokenType> = None; // Результат структуры.
    let mut parameters: Option< Vec<(Bytes, StructureType)> > = None; // Параметры структуры.
    if lineTokensLength > 1 && *lineTokens[1].getDataType() == TokenType::CircleBracketBegin
    { // Если токенов > 1 и 1 токен это `TokenType::CircleBracketBegin`
      // значит это вариант параметры + возможно результат.

      // Получаем параметры структуры.
      parameters =
        if let Some(lines) = &lineTokens[1].lines
        { // Берём первую линию внутри скобок (там обычно перечислены параметры).
          let mut result: Vec<(Bytes, StructureType)> = Vec::new();
          for lineLink in lines
          { // Берём вложенные токены в `TokenType::CircleBracketBegin`
            // получаем параметры из этих токенов, давая доступ к родительским структурам.
            let paramTokens: Vec<Token> = {
              let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
              line.tokens.clone().unwrap_or_default()
            };
            result.extend(
              parentLink.read().unwrap().getStructureParameters(&paramTokens)
            );
          }
          Some(result)
        } else {
          None
        };

      // Если > 3 (т.е name () -> result)
      // то значит это результат структуры .
      // todo: Может быть объединено с блоком ниже
      if lineTokensLength > 3 &&
        *lineTokens[2].getDataType() == TokenType::Pointer &&
        *lineTokens[3].getDataType() != TokenType::None
      {
        // В таком случае просто читаем тип результата структуры.
        newStructureResultType = Some(lineTokens[3].getDataType());
      }
      // Если результата не было, то просто пропускаем.
      
    } else
    { // В этом случае это вариант только с результатом структуры.
      
      if lineTokensLength > 2 &&
        *lineTokens[1].getDataType() == TokenType::Pointer &&
        *lineTokens[2].getDataType() != TokenType::None
      {
        // В таком случае просто читаем тип результата структуры.
        newStructureResultType = Some(lineTokens[2].getDataType());
      }
      // Если результата не было, то просто пропускаем.
      
    }
    // Если параметров и результата не было, то просто пропускаем.
    
    // Имя структуры-примитива занято — параллельную структуру не создаём.
    if isPrimitiveName(&newStructureName) {
      return true;
    }

    // Создаём новую структуру.
    let mut newStructure: Structure =
      Structure::new(
        Some(newStructureName),
        StructureMut::Constant, // todo По идее надо вычислять из синтаксиса.
        StructureType::Primitive(PrimitiveType::Method), // todo По идее надо вычислять из синтаксиса.
        Some(lineLines),
        Some(parentLink.clone())
      );
    newStructure.isFfiBlock = isFfi;

    // Ставим параметры структуры, если они были
    if let Some(parameters) = &parameters
    {
      for parameter in parameters
      {
        let mut parameterStructure: Structure = Structure::new(
          parameter.0.toString(),
          StructureMut::Constant, // todo По идее надо еще читать правила mut (в `getStructureParameters`).
          parameter.1.clone(),
          None,
          None
        );
        // Объединение типов `n: U8 | String`: набор `вариантов` хранится отдельно,
        // как и у обычного объявления; `вариант` подбирается при каждом вызове.
        if let StructureType::Union(variants) = &parameter.1 {
          parameterStructure.unionTypes = Some(variants.clone());
        }
        newStructure.pushStructure(Arc::new(RwLock::new(parameterStructure)));
      }
    }

    // Ставим результат структуры, если он есть.
    newStructure.result = newStructureResultType
      .map(|newStructureResultType| Token::newEmpty(*newStructureResultType));

    // Запоминаем, была ли это форма метода (с круглыми скобками) или
    // просто именованного блока.
    // 
    // Для метода `body` ждёт вызова `name()`, для именованного блока `body`
    // выполняется сразу — иначе его никто никогда не выполнит
    // (никто не пишет `name` без скобок для вызова).
    let isMethodForm: bool = parameters.is_some();

    let newStructureLink: Arc<RwLock<Structure>> = Arc::new(RwLock::new(newStructure));
    { // Добавляем новую структуру в родителя.
      parentLink.write().unwrap()
        .pushStructure(newStructureLink.clone());
    }
    // Просматриваем строки этой новой структуры;
    //
    // todo В целом, это можно заменить на чтение при первом обращении к структуре;
    //      сейчас же все структуры читаются (подготавливаются),
    //      если попали на `lineIndex` указатель.
    //
    // Для FFI-блоков (как анонимных, так и именованных) тело должно
    // выполниться прямо здесь, иначе оно никогда не выполнится —
    // никто не зовёт `name` без скобок.
    // 
    // Метод (`Structure` с круглыми скобками) ждёт явного вызова.
    // `readStructureLines` сам сделает `push` FFI `scope`, потому
    // что `isFfiBlock=true`.
    if isFfi && !isMethodForm {
      readStructureLines(newStructureLink);
    }
    return true;
  }

  false
}

// =================================================================================================
