use super::condition::executeConditionStatement;
use super::linear::linearLine;
use super::nested::searchNestedStructure;
use super::tags::{isFfiTagToken, isPrevLineFfiTag};
use crate::parser::bytes::Bytes;
use crate::parser::readStructureLines;
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
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
/// - Вложенная структура (array/vector/list ...). ⟶ [`searchNestedStructure`];
/// 
/// - Линейное выражение ⟶ [searchLinearStructure`];
/// 
/// - Условный блок ⟶ [`executeConditionStatement`].
//
// todo Я всё ещё думаю, что его стоит переименовать. Потому что он ищет структуры да,
//  но не совсем так. По сути он ищет последовательности, которые приводят к их созданию.
//  или `линейное выражение` - это фактически создание/op над структурой.
//  Но назвать `expression` это нельзя, хотя может быть в какой-то мере и можно, ведь
//  мы ищем именно последовательность токенов, обрабатываем и получаем результат.
pub fn searchStructure(
  line: &RwLockReadGuard<Line>,
  parentLink: Arc<RwLock<Structure>>
) -> bool
{
  // -----------------------------------------------------------------------------------------------
  
  // Ссылка на токены линии.
  let lineTokens: &Vec<Token> = match &line.tokens {
    Some(tokens) => tokens,
    None => return false // Если в линии нет токенов, то мы её не читаем.
  };
  
  let lineTokensLength: usize = lineTokens.len(); // размер токенов линии.

  let lineLines: Option<Vec< Arc<RwLock<Line>> >> = line.lines.clone(); // Вложенные линии.

  // -----------------------------------------------------------------------------------------------
  
  // Многострочный анонимный FFI-блок:
  //
  //   [ffi]
  //   { ... } ⟵ эта линия
  //
  // В tokenizer'е линия `{ ... }` (без ничего перед `{`) теперь выходит как
  // `tokens = Some(vec![])`, `lines = Some([внутренние линии])` (после фикса
  // токенайзера). Либо `tokens = Some(vec![])` — пустой список, если
  // пред-фиксовый фикс не сработал. В обоих случаях Word/SquareBracketBegin
  // ветки не сработают — ловим этот случай отдельно: если предыдущая
  // линия — `[ffi]`, а текущая — пустая/None-токены с вложением,
  // выполняем её как анонимный FFI-блок.
  //
  // Эта проверка идёт ДО раннего `return false` на пустых токенах ниже —
  // иначе анонимный FFI-блок после `[ffi]` на предыдущей строке никогда
  // бы не дошёл досюда.
  //
  if lineTokensLength == 0 &&
    lineLines.is_some() &&
    isPrevLineFfiTag(&parentLink, parentLink.read().unwrap().lineIndex.get())
  {
    let newStructureLink: Arc<RwLock<Structure>> = 
      Arc::new(RwLock::new(Structure::new(
        None, // Анонимный
        StructureMut::Constant,
        StructureType::Primitive(PrimitiveType::Method),
        lineLines.clone(),
        Some(parentLink.clone())
      )));
    newStructureLink.write().unwrap().isFfiBlock = true;
    parentLink.write().unwrap()
      .pushStructure(newStructureLink.clone());
    // `readStructureLines` сам сделает push `scope`, потому что `isFfiBlock=true`.
    readStructureLines(newStructureLink);
    return true;
  }

  // Дальше идёт старая логика: пустые линии не интересуют.
  if lineTokensLength == 0 {
    return false; // Если в линии нет токенов, то мы её не читаем.
  }

  let firstTokenType: &TokenType = lineTokens[0].getDataType(); // Тип первого токена в строке.

  // -----------------------------------------------------------------------------------------------
  
  // Обработка тега `[ffi]` в начале той же строки, что и блок:
  //
  // - `[ffi] test() { ... }`: Метод с FFI-тегом.
  //
  // - `[ffi] name { ... }`: Именованный FFI-блок.
  //
  // - `[ffi] { ... }`: Анонимный FFI-блок.
  //
  //
  // Логика:
  //
  // 1) Первый токен — SquareBracketBegin, внутри которого `ffi`;
  //
  // 2) Скипаем тег и работаем с оставшимися токенами как с обычной структурой;
  //
  // 3) Если после тега нет больше токенов — анонимный блок: создаём структуру,
  //    пушим `scope`, читаем тело, пупим `scope`.
  //
  // 4) Если после тега есть Word — обычная структура (метод или именованный
  //    блок), помечаем `isFfiBlock = true`. `readStructureLines` сам за себя разберётся
  //    с FFI `scope`, когда эта структура будет вызвана.
  //
  if *firstTokenType == TokenType::SquareBracketBegin
    && lineLines.is_some()
  {
    // Проверяем, что внутри скобок именно "ffi" (а не любой тег).
    let isFfiTag: bool = isFfiTagToken(&lineTokens[0]);
    if isFfiTag
    {
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      // Достаём токены после тега `[ffi]`.
      let restTokens: &[Token] = &lineTokens[1..];
      let restTokensLength: usize = restTokens.len();
      let innerLines: Vec< Arc<RwLock<Line>> > = lineLines.clone().unwrap();

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      // Случай A: анонимный блок `[ffi] { ... }`.
      //
      // Тег — единственный токен, и блок сразу же выполняется.
      if restTokensLength == 0
      {
        let newStructureLink: Arc<RwLock<Structure>> = 
          Arc::new(RwLock::new(Structure::new(
            None, // Анонимный
            StructureMut::Constant,
            StructureType::Primitive(PrimitiveType::Method),
            Some(innerLines),
            Some(parentLink.clone())
          )));
        // isFfiBlock=true уже выставлен дефолтом (false), но пометим явно для ясности.
        newStructureLink.write().unwrap().isFfiBlock = true;

        parentLink.write().unwrap()
          .pushStructure(newStructureLink.clone());

        // `readStructureLines` сам пушнет scope, потому что isFfiBlock=true.
        // Здесь НЕ зовём enterFfiBlock/exitFfiBlock явно — будет двойной push.
        readStructureLines(newStructureLink);
        return true;
      }

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      // Случай B: `[ffi] <Word> ...` — структура с тегом.
      //
      // Дальше идёт стандартная обработка структуры: берём имя, парсим
      // параметры/результат, создаём Structure, читаем body. isFfiBlock
      // проставится внутри благодаря новой логике в `readStructureLines`, но чтобы
      // не зависеть от того, что блок когда-то будет вызван, ставим флаг
      // сразу на этапе создания.
      if restTokensLength >= 1
        && *restTokens[0].getDataType() == TokenType::Word
      {
        // Перепаковываем оставшиеся токены в синтетический Line, чтобы
        // дальше прогнать их через существующую логику обработки Word/()/etc.
        // Самый простой способ — собрать новый Vec<Token> и подменить line.tokens
        // через уже существующий "магический" путь ниже. Для этого
        // делаем минимальную отдельную ветку.
        let newStructureName: String = restTokens[0].getData()
          .toString().unwrap_or_default();
        let mut newStructureResultType: Option<&TokenType> = None;
        let mut parameters: Option< Vec<(Bytes, StructureType)> > = None;

        // Определяем форму: метод `name(...)` или именованный блок `name` (без скобок).
        // Для метода body НЕ выполняется на этапе определения — он ждёт вызова.
        // Для именованного блока body ВЫПОЛНЯЕТСЯ сразу (как анонимный блок).
        let isMethod: bool = restTokensLength > 1
          && *restTokens[1].getDataType() == TokenType::CircleBracketBegin;

        // Если идёт метод: `[ffi] name ( params ) [-> result]`.
        if isMethod
        {
          parameters =
            if let Some(lines) = &restTokens[1].lines
            {
              let mut result: Vec<(Bytes, StructureType)> = Vec::new();
              for lineLink in lines
              {
                let paramTokens: Vec<Token> = {
                  let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
                  line.tokens.clone().unwrap_or_default()
                };
                result.extend(
                  parentLink.read().unwrap().getStructureParameters(&paramTokens)
                );
              }
              Some(result)
            } else { None };
          if restTokensLength > 3
            && *restTokens[2].getDataType() == TokenType::Pointer
            && *restTokens[3].getDataType() != TokenType::None
          {
            newStructureResultType = Some(restTokens[3].getDataType());
          }
        } else
        if restTokensLength > 2
          && *restTokens[1].getDataType() == TokenType::Pointer
          && *restTokens[2].getDataType() != TokenType::None
        {
          newStructureResultType = Some(restTokens[2].getDataType());
        }

        let mut newStructure: Structure =
          Structure::new(
            Some(newStructureName),
            StructureMut::Constant,
            StructureType::Primitive(PrimitiveType::Method),
            Some(innerLines),
            Some(parentLink.clone())
          );
        newStructure.isFfiBlock = true;

        for parameter in parameters.unwrap_or_default()
        {
          newStructure.pushStructure(
            Arc::new(RwLock::new(Structure::new(
              parameter.0.toString(),
              StructureMut::Constant,
              parameter.1,
              None,
              None
            )))
          );
        }

        newStructure.result = newStructureResultType
          .map(|tokenType| Token::newEmpty(*tokenType));

        let newStructureLink: Arc<RwLock<Structure>> =
          Arc::new(RwLock::new(newStructure));
        parentLink.write().unwrap()
          .pushStructure(newStructureLink.clone());

        // Для метода (с круглыми скобками) body ждёт вызова — не выполняем.
        //
        // Для именованного блока (без скобок) body выполняется прямо сейчас,
        // под удерживаемым FFI scope. `readStructureLines` сам пушнет scope, потому
        // что isFfiBlock=true, так что здесь enterFfiBlock НЕ нужен.
        if !isMethod {
          readStructureLines(newStructureLink);
        }
        return true;
      }

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      // Неизвестная форма после `[ffi]` — падаем в основной парсер, чтобы
      // он попробовал обработать как структуру (и корректно ругнулся, если
      // это не структура).

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    }
  }

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
        return linearLine(lineTokens, &lineLines, parentLink);
      }
    }

    return if let Some(lineLines) = lineLines
    { // Если в линии есть вложение, то это структура с вложением.
      searchNestedStructure(lineTokens, lineTokensLength, lineLines, parentLink)
    } else
    { // Это линейное выражение.
      linearLine(lineTokens, &lineLines, parentLink)
    }
  } else
  
  if *firstTokenType == TokenType::Question && !lineLines.is_none()
  { // В том случае, если это не структура и не линейное выражение, 
    // мы видим `TokenType::Question` в начале строки и
    // есть вложения у этой линии, то это условное вложение.
    return executeConditionStatement(lineTokens, &lineLines, &parentLink);
  }

  // -----------------------------------------------------------------------------------------------
  
  false

  // -----------------------------------------------------------------------------------------------
}

// =================================================================================================
