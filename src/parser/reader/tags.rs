use crate::parser::bytes::Bytes;
use crate::parser::readStructureLines;
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
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

/// Проверяет, является ли предыдущая [`Line`] тегом `[ffi]`.
/// 
/// Используется для многострочной формы:
/// ```ignore
/// [ffi]
/// {
///   ...
/// }
/// ```
/// 
/// Здесь вторая [`Line`] сама по себе не знает о теге сверху.
pub fn isPrevLineFfiTag(parentLink: &Arc<RwLock<Structure>>, lineIndex: usize) -> bool
{
  // Если до этой `Line` нет других - то ничего не будет.
  if lineIndex == 0 {
    return false;
  }

  // Получаем предыдущую `Line`.
  let prevLineLinkOption: Option< Arc<RwLock<Line>> > = parentLink.read().unwrap()
    .lines.as_ref()
    .and_then(|lines| lines.get(lineIndex-1).cloned());
  
  let prevLineLink: Arc<RwLock<Line>> = match prevLineLinkOption {
    Some(line) => line,
    None => return false,
  };

  // Получаем токены предыдущей `Line`.
  let prevLine: RwLockReadGuard<Line> = prevLineLink.read().unwrap();
  let prevLineTokens: &[Token] = match prevLine.tokens.as_ref() {
    Some(tokens) => tokens,
    None => return false,
  };
  
  // Предыдущая `Line` должна иметь ровно один токен: `[ffi]`.
  if prevLineTokens.len() != 1 {
    return false;
  }
  
  // Проверяет, является ли токен тегом `[ffi]`.
  isFfiTagToken(&prevLineTokens[0])
}

// =================================================================================================

// todo desc
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
  let lineLines: Option<Vec< Arc<RwLock<Line>> >> = line.lines.clone();
  
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
    isPrevLineFfiTag(&structureLink, structureLink.read().unwrap().lineIndex.get())
  {
    let newStructureLink: Arc<RwLock<Structure>> = 
      Arc::new(RwLock::new(Structure::new(
        None, // Анонимный.
        StructureMut::Constant,
        StructureType::Primitive(PrimitiveType::Method),
        lineLines.clone(),
        Some(structureLink.clone())
      )));
    newStructureLink.write().unwrap().isFfiBlock = true;
    structureLink.write().unwrap()
      .pushStructure(newStructureLink.clone());
    // `readStructureLines` сам сделает push `scope`, потому что `isFfiBlock=true`.
    readStructureLines(newStructureLink);
    return true;
  }
  
  // -----------------------------------------------------------------------------------------------

  // Если в линии нет токенов, то мы её не читаем.
  if lineTokensLength == 0 {
    return false;
  }
  // Тип первого токена в строке.
  let firstTokenType: &TokenType = lineTokens[0].getDataType();
  
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
  if *firstTokenType == TokenType::SquareBracketBegin &&
     lineLines.is_some()
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
                  structureLink.read().unwrap().getStructureParameters(&paramTokens)
                );
              }
              Some(result)
            } else {
              None
            };
          if restTokensLength > 3 &&
             *restTokens[2].getDataType() == TokenType::Pointer &&
             *restTokens[3].getDataType() != TokenType::None
          {
            newStructureResultType = Some(restTokens[3].getDataType());
          }
        } else
        if restTokensLength > 2 &&
          *restTokens[1].getDataType() == TokenType::Pointer &&
          *restTokens[2].getDataType() != TokenType::None
        {
          newStructureResultType = Some(restTokens[2].getDataType());
        }

        let mut newStructure: Structure =
          Structure::new(
            Some(newStructureName),
            StructureMut::Constant,
            StructureType::Primitive(PrimitiveType::Method),
            Some(innerLines),
            Some(structureLink.clone())
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
        structureLink.write().unwrap()
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
  
  false
  
  // -----------------------------------------------------------------------------------------------
}

// =================================================================================================
