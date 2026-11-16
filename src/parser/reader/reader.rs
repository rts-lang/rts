use super::condition::{executeConditionStatement, replaceConditionExpression};
use super::linear::linearLine;
use crate::parser::bytes::Bytes;
use crate::parser::structure::ffi::bridge;
use crate::parser::structure::ffi::scopeStack;
use crate::parser::structure::primitives::{isPrimitiveName, ArgsPrimitives, MethodsPrimitives};
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::tokenizer::tokenizer::readTokensSimple;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use crate::{_argc, _argv, _exit};
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
// =================================================================================================

// Обрабатывает последовательности из `Token`:
// 
// - Читает `Token` ⟶ вложения или одиночные;
//
// - Читает последовательности `Token` ⟶ `Line`;
//
// - Создаёт `Structure`.
//
// Всё это приводит к работе/вычислению этих записей как кода.

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

/// Проверяет, является ли [`Token`] тегом `[ffi]`.
/// 
/// Токен должен иметь тип [`TokenType::SquareBracketBegin`] и
/// хранить одну вложенную линию с токеном `ffi` типа [`TokenType::Word`];
/// 
/// Возвращаем `true`, если это так.
//
// todo Когда появятся другие теги (`async`, `thread` и т.д.) — расширить.
fn isFfiTagToken(token: &Token) -> bool
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
fn isPrevLineFfiTag(parentLink: &Arc<RwLock<Structure>>, lineIndex: usize) -> bool
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

/// Эта функция ищет линию с `return` и определяет куда пишется результат:
/// 
/// - `:=`: В ближайшую именованную [`Structure`] вверх по области видимости;
/// 
/// - `=`: В текущую [`Structure`].
/// 
fn searchReturnLine(
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

/// Ищет структуры.
///
/// Это может быть:
/// - Вложенная структура (array/vector/list ...).   todo Необходимо вынести в отдельный метод.
/// - Линейное выражение ⟶ [linearStructure`];
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
    // readLines сам пушнет scope, потому что isFfiBlock=true.
    readLines(newStructureLink);
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
    // readLines сам пушнет scope, потому что isFfiBlock=true.
    readLines(newStructureLink);
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
  //      блок), помечаем `isFfiBlock = true`. `readLines` сам за себя разберётся
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

        // readLines сам пушнет scope, потому что isFfiBlock=true.
        // Здесь НЕ зовём enterFfiBlock/exitFfiBlock явно — будет двойной push.
        readLines(newStructureLink);
        return true;
      }

      // --- Случай B: `[ffi] <Word> ...` — структура с тегом. ---
      // Дальше идёт стандартная обработка структуры: берём имя, парсим
      // параметры/результат, создаём Structure, читаем body. isFfiBlock
      // проставится внутри благодаря новой логике в readLines, но чтобы
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
        // под удерживаемым FFI scope. readLines сам пушнет scope, потому
        // что isFfiBlock=true, так что здесь enterFfiBlock НЕ нужен.
        if !isMethod {
          readLines(newStructureLink);
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
        // ждёт явного вызова. `readLines` сам пушнет FFI scope, потому
        // что `isFfiBlock=true`.
        if isFfi && !isMethodForm {
          readLines(newStructureLink);
        }
        //            readLines(
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

// =================================================================================================

/// Создает основную структуру программы.
pub fn createMainStructure(tokenizerLinesLinks: Vec< Arc<RwLock<Line>> >) -> Arc<RwLock<Structure>>
{
  // -----------------------------------------------------------------------------------------------

  // Основная структура; В неё вкладываются остальные;
  //
  // В эту структуру будут переданы стартовые параметры;
  //
  // Неизменяемая; Действует во время всей жизни программы.
  let mainStructureLink: Arc<RwLock<Structure>> =
    Arc::new(RwLock::new(Structure::new(
      None,
      StructureMut::Constant,
      StructureType::Primitive(PrimitiveType::Method),
      None,
      None
    )));
  let mut mainStructure: RwLockWriteGuard<Structure> = mainStructureLink.write().unwrap();

  // Присваиваем линии от Tokenizer; Это то что будет запущено как основная часть.
  mainStructure.lines = Some(tokenizerLinesLinks);

  // -----------------------------------------------------------------------------------------------
  // Параметры запуска (`argc`, `argv`).
  //
  // Значения приходят от запуска, поэтому у них, в отличие от остальных примитивов, есть линии.

  for (name, dataType) in ArgsPrimitives
  {
    let lines: Vec< Arc<RwLock<Line>> > = match *name
    {
      // Одна линия с одним токеном - сколько аргументов.
      "argc" => vec![
        Arc::new(RwLock::new(Line
        {
          tokens: Some(vec![
            Token::new(
              TokenType::UInt,
              Bytes::new(_argc.get().copied().unwrap_or(0).to_string())
            )
          ]),
          lines: None
        }))
      ],

      // По линии на каждый аргумент.
      "argv" => _argv.get().map(|values| values.as_slice()).unwrap_or(&[])
        .iter()
        .map(|arg: &String| Arc::new(RwLock::new(Line
        {
          tokens: Some(vec![
            Token::new(
              TokenType::String,
              Bytes::new( String::from(arg) )
            )
          ]),
          lines: None
        })))
        .collect(),

      _ => Vec::new()
    };

    mainStructure.pushStructure(
      Arc::new(RwLock::new(Structure::new(
        Some(String::from(*name)),
        StructureMut::Constant, // Неизменяемая.
        StructureType::Primitive(dataType.clone()),
        Some(lines),
        Some( mainStructureLink.clone() ) // Ссылаемся на родителя.
      )))
    );
  }

  // -----------------------------------------------------------------------------------------------
  // Примитивы (базовые структуры языка).

  for (name, dataType) in PrimitiveType::iter()
  {
    mainStructure.pushStructure(
      Arc::new(RwLock::new(Structure::new(
        Some(String::from(name)),
        StructureMut::Constant, // Неизменяемая.
        StructureType::Primitive(dataType.clone()),
        None, // Без значения — просто маркер.
        Some( mainStructureLink.clone() ) // Ссылаемся на родителя.
      )))
    );
  }

  for name in MethodsPrimitives
  {
    mainStructure.pushStructure(
      Arc::new(RwLock::new(Structure::new(
        Some(String::from(*name)),
        StructureMut::Constant, // Неизменяемая.
        StructureType::Primitive(PrimitiveType::Method),
        None, // Без значения — просто маркер.
        Some( mainStructureLink.clone() ) // Ссылаемся на родителя.
      )))
    );
  }
  
  // -----------------------------------------------------------------------------------------------
  drop(mainStructure);
  mainStructureLink
}

// =================================================================================================

/// Это основная функция для парсинга строк;
/// 
/// Она разделена на подготовительную часть, и часть запуска readLine().
fn parseLines(tokenizerLinesLinks: Vec< Arc<RwLock<Line>> >) -> Arc<RwLock<Structure>>
{ 
  // Подготовка к запуску.
  let mainStructureLink: Arc<RwLock<Structure>> = createMainStructure(tokenizerLinesLinks);

  // Подготовка закончена, читаем линии;
  //
  // Передаём ссылку на структуру и запускаем.
  readLines(mainStructureLink.clone());
  
  //
  mainStructureLink
}

// =================================================================================================

/// Эта функция занимается чтением блоков по ссылке на них.
//
// todo: исправить переполнение стека.
pub fn readLines(structureLink: Arc<RwLock<Structure>>) -> ()
{ // Получаем сколько линий вложено в структуру,
  // а также индекс чтения строк (у каждой структуры он свой, чтобы не путать чтение)

  // FFI-обёртка: если эта структура создана из `[ffi] { ... }` блока,
  // пушим слот в FFI scope-стек на всё время чтения её тела. Это
  // покрывает ВСЕ пути, по которым блок может быть выполнен:
  //   - анонимный `[ffi] { ... }` (выполняется прямо в searchStructure,
  //     но и оттуда можно звать readLines);
  //   - метод `[ffi] test() { ... }` (вызывается как функция в procedure.rs
  //     ⟶ readLines(calledStructureLink));
  //   - именованный блок `[ffi] name { ... }` (тоже вызывается).
  // Scope сам создаётся лениво в scopeStack::ensureFfiScope() при первом
  // FFI-вызове внутри блока — точно как просит issue #79/#84.
  let isFfi: bool = structureLink.read().unwrap().isFfiBlock;
  if isFfi {
    scopeStack::enterFfiBlock();
  }

  let linesLength: usize =
  {
    let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
    match &structure.lines
    {
      Some(lines) => lines.len(), // Если линии есть, значит получаем сколько их необходимо прочитать.
      None => return // Если нет линий, то не читаем.
    }
  };

  // Выполнение программы происходит до тех пор,
  // пока не будет всё прочитано, либо
  // пока не будет вызван _exitCode на true.
  let mut lineLink: Arc< RwLock<Line> >;

  while !_exit.load(Ordering::Relaxed)
  { // Если мы читаем строки, то создаём сразу ссылку на текущую линию.
    
    let lineIndex: usize = structureLink.read().unwrap().lineIndex.get();
    if lineIndex >= linesLength {
      break;
    }

    lineLink =
    { // Получаем её через чтение текущей структуры;
      // Берём линию по индексу линии (она точно будет, поскольку выше мы это проверили).
      let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
      let lines: &Vec< Arc<RwLock<Line>> > = structure.lines.as_ref().unwrap();
      lines[lineIndex].clone() // Клонируем нужную линию по индексу.
    };
    let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
    
    // После чего проверяем, если линия пустая на токены, то не читаем и идём дальше.
    if line.tokens.is_none()
    {
      structureLink.read().unwrap().lineIndex.set(lineIndex + 1);
      continue;
    }
    
    // Если всё хорошо, то начинаем читать через специальные функции.
    //
    // Ищем структуры.
    if !searchStructure(&line, structureLink.clone())
    { // Читаем return.

      if !searchReturnLine(&line, structureLink.clone())
      { // Ищем линейные выражения.

        let tokens: &mut Vec<Token> =
          &mut line
            .tokens.clone() // Клонируем токены, для сохранения возможности повторного запуска.
            .unwrap_or_default(); // todo плохо.
        drop(line);
        // Линия-оператор: результат никому не нужен (`lib.print(x)`).
        structureLink.read().unwrap()
          .expressionWith(tokens, &bridge::FfiExpect::Discard);
      }
    }
    else
    { // `searchStructure` обработал структуру (например `?`-блок).
      //
      // Если после этого у метода появился `result` — внутри блока был `:=`,
      // который поднялся до этого метода. Прерываем чтение.
      if structureLink.read().unwrap().result.is_some() {
        break
      }
    }

    { // Идём дальше
      let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
      structure.lineIndex.set(structure.lineIndex.get() + 1);
    }
  }

  // Сбрасываем указатель линий для текущей структуры на `0`
  // для того чтобы можно было запускать повторно.
  structureLink.read().unwrap().lineIndex.set(0);

  // FFI-обёртка: снимаем свой слот со стека.
  //
  // Внутри scope дропается (если был создан) вместе с библиотеками
  // и `AllocatedMemory`, привязанными к нему. Делаем это после сброса
  // lineIndex, чтобы можно было перезапустить блок — повторный запуск
  // снова пушнет свой слот.
  if isFfi {
    scopeStack::exitFfiBlock();
  }
}

// =================================================================================================

// todo Я вынес `runCode!` за место метода, который был тут;
//  И вот не понятно должны ли эти методы тоже быть в `testing`, хотя тот же `runCode!`
//  должен ли быт там? Логично что это часть основной инфраструктуры `parser`, поэтому оно
//  пока что так разделено. Это надо решить, возможно оно так и должно быть.

/// Запускает код и возвращает основную структуру.
pub fn runCodeBytes(buffer: &mut Vec<u8>) -> Arc<RwLock<Structure>>
{
  parseLines( readTokensSimple(buffer) )
}

/// Запускает `.rt` файл и возвращает основную структуру.
///
/// В отличие от обёртки `runCode!(r#"m := import("…")"#)` не опирается на
/// `import` — файл читается и выполняется напрямую, как entry в CLI.
pub fn runCodeFile(path: &str) -> Arc<RwLock<Structure>>
{
  assert!(
    path.ends_with(".rt"),
    "runCodeFile: expected a `.rt` file, got `{path}`"
  );
  let mut buffer: Vec<u8> = std::fs::read(path)
    .unwrap_or_else(|error| panic!("runCodeFile: read `{path}`: {error}"));
  let absolutePath: String = std::fs::canonicalize(path)
    .unwrap_or_else(|error| panic!("runCodeFile: canonicalize `{path}`: {error}"))
    .to_string_lossy().replace('\\', "/");
  let previousPath: Option<Arc<String>> =
    crate::_sourcePath.with(|sourcePath| sourcePath.borrow().clone());
  crate::_sourcePath.with(|sourcePath|
    *sourcePath.borrow_mut() = Some(Arc::new(absolutePath))
  );
  let result: Arc<RwLock<Structure>> = runCodeBytes(&mut buffer);
  crate::_sourcePath.with(|sourcePath|
    *sourcePath.borrow_mut() = previousPath
  );
  result
}

// =================================================================================================
