use super::condition::executeConditionStatement;
use super::linear::linearLine;
use super::tags::{isFfiTagToken, isPrevLineFfiTag};
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

/// Проверяет, что переданный `dataType` является математическим оператором.
//
// todo Есть проблема в том, что `=` и `:=` не совсем математические операторы.
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
/// - Вложенная структура (array/vector/list ...).   todo Необходимо вынести в отдельный метод.
/// 
/// - Линейное выражение ⟶ [linearStructure`];
/// 
/// - Условный блок ⟶ [`executeConditionStatement`].
pub fn searchStructure(
  line: &RwLockReadGuard<Line>,
  parentLink: Arc<RwLock<Structure>>
) -> bool
{
  let lineTokens: &Vec<Token> = // Ссылка на токены линии.
    if let Some(tokens) = &line.tokens { tokens } 
      else { return false }; // Если в линии нет токенов, то мы её не читаем.
  let lineTokensLength: usize = lineTokens.len(); // размер токенов линии.

  let lineLines: Option< Vec< Arc<RwLock<Line>> > > = line.lines.clone(); // Вложенные линии.

  // -----------------------------------------------------------------------------
  // Многострочный анонимный FFI-блок:
  //
  //   [ffi]
  //   { ... }              ⟵ эта линия
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
  // -----------------------------------------------------------------------------
  if lineTokensLength == 0
    && lineLines.is_some()
    && isPrevLineFfiTag(&parentLink, parentLink.read().unwrap().lineIndex.get())
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
    // `readStructureLines` сам пушнет scope, потому что isFfiBlock=true.
    readStructureLines(newStructureLink);
    return true;
  }

  // Дальше идёт старая логика: пустые линии не интересуют.
  if lineTokensLength == 0 {
    return false; // Если в линии нет токенов, то мы её не читаем.
  }

  let firstTokenType: &TokenType = lineTokens[0].getDataType(); // Тип первого токена в строке.

  // -----------------------------------------------------------------------------
  // Многострочный анонимный FFI-блок:
  //
  //   [ffi]
  //   { ... }              ⟵ эта линия
  //
  // В tokenizer'е линия `{ ... }` (без ничего перед `{`) теперь выходит как
  // `tokens = None`, `lines = Some([внутренние линии])` (после фикса
  // токенайзера). Либо `tokens = Some(vec![])` — пустой список, если
  // пред-фиксовый фикс не сработал. В обоих случаях Word/SquareBracketBegin
  // ветки не сработают — ловим этот случай отдельно: если предыдущая
  // линия — `[ffi]`, а текущая — пустая/None-токены с вложением,
  // выполняем её как анонимный FFI-блок.
  // -----------------------------------------------------------------------------
  if line.tokens.is_none() || lineTokens.is_empty() && // todo Кстати интересно тут норм что || без скобок?
     lineLines.is_some() && 
     isPrevLineFfiTag(&parentLink, parentLink.read().unwrap().lineIndex.get())
  {
    let newStructureLink: Arc<RwLock<Structure>> = 
      Arc::new(RwLock::new(
      Structure::new(
        None, // Анонимный
        StructureMut::Constant,
        StructureType::Primitive(PrimitiveType::Method),
        lineLines.clone(),
        Some(parentLink.clone())
      )));
    newStructureLink.write().unwrap().isFfiBlock = true;
    parentLink.write().unwrap()
      .pushStructure(newStructureLink.clone());
    // `readStructureLines` сам пушнет scope, потому что isFfiBlock=true.
    readStructureLines(newStructureLink);
    return true;
  }

  // -----------------------------------------------------------------------------
  // Обработка тега `[ffi]` в НАЧАЛЕ той же строки, что и блок:
  //
  //   [ffi] test() { ... }   — метод с FFI-тегом (вариант #3 в release/native/scopeRetantion/main.rt)
  //   [ffi] name { ... }     — именованный FFI-блок (вариант #7)
  //   [ffi] { ... }          — анонимный FFI-блок (варианты #4, #5)
  //
  // Логика:
  //   1) первый токен — SquareBracketBegin, внутри которого `ffi`;
  //   2) скипаем тег и работаем с оставшимися токенами как с обычной структурой;
  //   3) если после тега нет больше токенов — анонимный блок: создаём структуру,
  //      пушим scope, читаем тело, пупим scope.
  //   4) если после тега есть Word — обычная структура (метод или именованный
  //      блок), помечаем `isFfiBlock = true`. `readStructureLines` сам за себя разберётся
  //      с FFI scope, когда эта структура будет вызвана.
  //
  // Раньше поддерживался только тег `[ffi]` ОТДЕЛЬНОЙ строкой над структурой
  // (варианты #2 и #6). Этот код добавляет #3, #4, #5, #7.
  // -----------------------------------------------------------------------------
  if *firstTokenType == TokenType::SquareBracketBegin
    && lineLines.is_some()
  {
    // Проверяем, что внутри скобок именно "ffi" (а не любой тег).
    let isFfiTag: bool = isFfiTagToken(&lineTokens[0]);
    if isFfiTag
    {
      // Достаём токены ПОСЛЕ тега `[ffi]`.
      let restTokens: &[Token] = &lineTokens[1..];
      let restTokensLength: usize = restTokens.len();
      let innerLines: Vec< Arc<RwLock<Line>> > = lineLines.clone().unwrap();

      // --- Случай A: анонимный блок `[ffi] { ... }` ---
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

      // --- Случай B: `[ffi] <Word> ...` — структура с тегом. ---
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
      // Неизвестная форма после `[ffi]` — падаем в основной парсер, чтобы
      // он попробовал обработать как структуру (и корректно ругнулся, если
      // это не структура).
    }
  }

  if *firstTokenType == TokenType::Word
  { // Если мы видим TokenType::Word в начале строки, 
    // это значит, что это либо структура, либо линейное выражение.
    //
    // Если в токенах есть оператор (`:=` или `=`) — это линейное выражение,
    // даже если есть вложенные линии (например `a := ? True { = 20 }`).
    let hasOperator: bool = lineTokens.iter()
      .any(|t| isMathOperator(*t.getDataType()));
    if hasOperator
    { // Линейное выражение с оператором.
      return linearLine(lineTokens, &lineLines, parentLink);
    }
    if let Some(lineLines) = lineLines
    {
      // Если в линии есть вложение, то это структура с вложением.
      if let Some(newStructureName) = lineTokens[0].getData().toString()
      { // Здесь первый токен - имя структуры.
        
        // получаем имя структуры.
        let mut newStructureResultType: Option<&TokenType> = None; // Результат структуры.
        let mut parameters: Option< Vec<(Bytes, StructureType)> > = None; // Параметры структуры.
        if lineTokensLength > 1 && *lineTokens[1].getDataType() == TokenType::CircleBracketBegin
        { // Если токенов > 1 и 1 токен это TokenType::CircleBracketBegin 
          // значит это вариант параметры + возможно результат.

          // Получаем параметры структуры.
          parameters =
            if let Some(lines) = &lineTokens[1].lines
            { // Берём первую линию внутри скобок (там обычно перечислены параметры).
              let mut result: Vec<(Bytes, StructureType)> = Vec::new();
              for lineLink in lines
              { // Берём вложенные токены в TokenType::CircleBracketBegin 
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

        // Проверка блока тегов [ffi]
        // todo Нет [ffi] {} т.е. это проверка если строка сверху но нет в 1 строку. (актуально?)
        // todo Точно требуются комментарии по этому блоку.
        let isFfi: bool = if parentLink.read().unwrap().lineIndex.get() == 0
        { false } else
        {
          if let Some(siblingLines) = &parentLink.read().unwrap().lines
          {
            let prevLine: RwLockReadGuard<Line> = siblingLines[parentLink.read().unwrap().lineIndex.get() - 1].read().unwrap();
            if let Some(prevTokens) = &prevLine.tokens
            {
              if prevTokens.is_empty() ||
                *prevTokens[0].getDataType() != TokenType::SquareBracketBegin ||
                prevLine.lines.is_some()
              { false } else
              {
                if let Some(bracketLines) = &prevTokens[0].lines
                {
                  let mut found: bool = false;
                  for bracketLine in bracketLines
                  {
                    let bracketLineGuard: RwLockReadGuard<Line> = bracketLine.read().unwrap();
                    if let Some(bracketTokens) = &bracketLineGuard.tokens
                    {
                      for token in bracketTokens
                      {
                        if *token.getDataType() == TokenType::Word && 
                           token.getData().toString().unwrap_or_default() == "ffi"
                        {
                          found = true;
                        }
                      }
                      //
                    }
                  }
                  found
                  //
                } else {
                  false
                }
              }
              //
            } else {
              false
            }
          } else {
            false
          }
          //
        };
        
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
              StructureMut::Constant, // todo По идее надо еще читать правила mut (в getStructureParameters).
              parameter.1.clone(),
              None,
              None
            );
            // Объединение типов `n: U8 | String`: набор `вариантов` хранится отдельно,
            // как и у обычного объявления; `вариант` подбирается при каждом вызове.
            if let StructureType::Union(variants) = &parameter.1
            {
              parameterStructure.unionTypes = Some(variants.clone());
            }
            newStructure.pushStructure(Arc::new(RwLock::new(parameterStructure)));
          }
        }

        // Ставим результат структуры, если он есть.
        newStructure.result = newStructureResultType
          .map(|newStructureResultType| Token::newEmpty(*newStructureResultType));

        // Запоминаем, была ли это форма МЕТОДА (с круглыми скобками) или
        // просто именованного блока. Для метода body ждёт вызова `name()`,
        // для именованного блока body выполняется сразу — иначе его никто
        // никогда не выполнит (никто не пишет `name` без скобок для вызова).
        let isMethodForm: bool = parameters.is_some();

        let newStructureLink: Arc<RwLock<Structure>> = Arc::new(RwLock::new(newStructure));
        { // Добавляем новую структуру в родителя
          parentLink.write().unwrap()
            .pushStructure(newStructureLink.clone());
        }
        // Просматриваем строки этой новой структуры;
        // todo: в целом, это можно заменить на чтение при первом обращении к структуре;
        //       сейчас же все структуры читаются (подготавливаются),
        //       если попали на lineIndex указатель.
        // Для FFI-блоков (как анонимных, так и именованных) тело должно
        // выполниться прямо здесь, иначе оно никогда не выполнится —
        // никто не зовёт `name` без скобок. Метод (с круглыми скобками)
        // ждёт явного вызова. `readStructureLines` сам пушнет FFI scope, потому
        // что `isFfiBlock=true`.
        if isFfi && !isMethodForm {
          readStructureLines(newStructureLink);
        }
        //            readStructureLines(
        //              parentLink.read().unwrap()
        //                .getStructureByLink(&newStructureName).unwrap(), // todo: плохой вариант, можно лучше.
        //            );
        return true;
      }
    } else
    { // Это линейное выражение.
      return linearLine(lineTokens, &lineLines, parentLink);
    }
  } else
  // В том случае, если это не структура и не линейное выражение, 
  // мы видим `TokenType::Question` в начале строки и есть вложения у этой линии, 
  // то это условное вложение.
  if *firstTokenType == TokenType::Question && !lineLines.is_none()
  {
    return executeConditionStatement(lineTokens, &lineLines, &parentLink);
  }
  //
  false
}