use std::sync::{Arc, LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::{_argc, _argv, _exit};
use crate::parser::bytes::Bytes;
use crate::parser::structure::ffi::bridge;
use crate::parser::structure::ffi::scopeStack;
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::{Token};
use crate::tokenizer::types::tokenType::{TokenType};
use crate::tokenizer::tools::splitByType::splitByType;
#[cfg(not(target_family = "wasm"))]
use crate::_debugMode;
#[cfg(not(target_family = "wasm"))]
use crate::logger::logger::{log, logSeparator};
#[cfg(not(target_family = "wasm"))]
use std::time::{Duration, Instant};
use crate::parser::structure::structureType::StructureType;
// =================================================================================================

// Предоставляет механизмы для парсинга токенов,
// что позволяет запускать получившиеся структуры.

// =================================================================================================

/// Проверяет, что переданный dataType является математическим оператором
const fn isMathOperator(dataType: TokenType) -> bool
{
  matches!(dataType, 
    // todo А еще почему тут только 1 single оператор а не все math?
    TokenType::Equals      /* | // =
    todo Может плохо работать с #85, нужен контроль
    TokenType::UnaryPlus      | // ++
    TokenType::PlusEquals     | // +=
    TokenType::UnaryMinus     | // --
    TokenType::MinusEquals    | // -=
    TokenType::UnaryMultiply  | // **
    TokenType::MultiplyEquals | // *=
    TokenType::UnaryDivide    | // //
    TokenType::DivideEquals   | // /=
    TokenType::UnaryModulo    | // %%
    TokenType::ModuloEquals   | // %=
    TokenType::UnaryExponent  | // ^^
    TokenType::ExponentEquals   // ^=
    */
  )
}

// =================================================================================================

/// Проверяет, является ли токен тегом `[ffi]` (issue #79, #84).
///
/// `SquareBracketBegin` кодируется в tokenizer'е как токен с
/// `dataType = SquareBracketBegin` и вложенными `lines` —
/// собственно содержимым скобок. Внутри содержимого лежит одна линия
/// с токенами `[Word("ffi")]` (для `[ffi]`). Возвращаем `true`, если это так.
///
/// todo Когда появятся другие теги (`@async`, `@thread` и т.д.) — расширить.
fn isFfiTagToken(token: &Token) -> bool
{
  if *token.getDataType() != TokenType::SquareBracketBegin
  {
    return false;
  }
  let lines: &Vec< Arc<RwLock<Line>> > = match token.lines.as_ref()
  {
    None => return false,
    Some(lines) => lines,
  };
  for lineLink in lines
  {
    let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
    let tokens: &Vec<Token> = match line.tokens.as_ref()
    {
      None => continue,
      Some(t) => t,
    };
    for t in tokens
    {
      if *t.getDataType() == TokenType::Word
        && t.getData().toString().unwrap_or_default() == "ffi"
      {
        return true;
      }
    }
  }
  false
}

/// Проверяет, является ли предыдущая линия (по индексу `lineIndex - 1`)
/// тегом `[ffi]`. Используется для многострочной формы
///   [ffi]
///   { ... }
/// где вторая линия сама по себе не знает про тег.
fn isPrevLineFfiTag(parentLink: &Arc<RwLock<Structure>>, lineIndex: usize) -> bool
{
  if lineIndex == 0
  {
    return false;
  }
  let prevLineOpt: Option<Arc<RwLock<Line>>> = parentLink.read().unwrap()
    .lines.as_ref()
    .and_then(|lines| lines.get(lineIndex - 1).cloned());
  let prevLine: Arc<RwLock<Line>> = match prevLineOpt
  {
    None => return false,
    Some(l) => l,
  };
  let prevGuard: RwLockReadGuard<Line> = prevLine.read().unwrap();
  let prevTokens: &Vec<Token> = match prevGuard.tokens.as_ref()
  {
    None => return false,
    Some(t) => t,
  };
  // Многострочная форма: ровно один токен `[ffi]` на предыдущей строке.
  if prevTokens.len() != 1
  {
    return false;
  }
  isFfiTagToken(&prevTokens[0])
}

// =================================================================================================

/// Эта функция ищет return для структур `= value`;
/// Видно, что это не просто валяющееся значение
fn searchReturn(line: &RwLockReadGuard<Line>, structureLink: Arc<RwLock<Structure>>) -> bool
{
  let mut lineTokens: Vec<Token> =
  { // Читаемая линия, на которой мы сейчас находимся
    match line.tokens.clone()
    {
      None => return false, // Если в линии нет токенов, то мы её не читаем
      Some(tokens) =>
      { // Токены линии на которой мы сейчас находимся
        tokens
      }
    }
  };
  match lineTokens.is_empty()
  {
    false => {}
    true => return false, // Если в линии нет токенов, то мы её не читаем
  }

  // Возвращаем успешно или не успешно мы нашли
  match *lineTokens[0].getDataType() == TokenType::Equals
  {
    false => { false } // Это был не результат, идём дальше
    true =>
    { // Если нашли TokenType::Equals, значит это return, сразу удаляем его,
      // Чтобы он нам не мешался потом
      lineTokens.remove(0);

      // Редактируемый родитель, поскольку мы собираемся присвоить значение его result
      let newResultData: Token =
      { // Используем expression, чтобы получить результат выражения;
        let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
        structure.expression(&mut lineTokens)
      };

      let mut structure: RwLockWriteGuard<Structure> = structureLink.write().unwrap();

      // Структура ожидает какой-то тип в результате,
      // либо это может быть TokenType:None. Но мы просто будем менять data

      match structure.result
      {
        Some(_) =>
        { // Вариант, в котором результат ожидает возвращение определённого типа данных;
          match &mut structure.result
          { None => {} Some(structureResult) =>
          { // Присваиваем новую data результату;
            structureResult.setData( newResultData.getData() );
          }}
        }
        _ =>
        { // Вариант, в котором тип результата был не указан;
          // Используем expression, чтобы получить результат выражения;
          // Присваиваем новый результат;
          structure.result = Some( newResultData );
        }
      }

      // Всё успешно, это был результат
      true
    }
  }
}

// =================================================================================================

/// Читает линейную запись
fn linearStructure(lineTokens: &[Token], parentLink: Arc<RwLock<Structure>>) -> bool
{
  // Получаем тип операции
  let opType: TokenType = lineTokens.iter().find_map(|token| {
    if isMathOperator( *token.getDataType() ) {
      Some(*token.getDataType())
    } else {
      None
    }
  }).unwrap_or(TokenType::None);

  // Получаем левую и правую часть
  let leftValue: Vec<Token>;
  let mut rightValue: Option< Vec<Token> > = None;
  {
    if opType == TokenType::None
    { // Операции не было
      leftValue = std::mem::take(&mut lineTokens.to_owned()); // todo: Тут точно клонирование ?
    }
    else
    {// Операция есть
      let mut parts: Vec<Line> = splitByType(lineTokens.to_owned(), &[opType]); // todo: Тут точно клонирование ?

      leftValue = std::mem::take(&mut parts[0].tokens).unwrap();
      rightValue = std::mem::take(&mut parts[1].tokens);
    }
  }

  let structureName: String;
  let structureMutability: StructureMut;
  let mut structureType: StructureType;
  { // Определяем тип данных у левой части выражения
    let (structureNameTokens, structureTypeTokens): (Vec<Token>, Option< Vec<Token> >) =
    {
      let mut parts: Vec<Line> = splitByType(leftValue.clone(), &[TokenType::Colon]);
      if parts.len() == 2 {
        (std::mem::take(&mut parts[0].tokens).unwrap(), std::mem::take(&mut parts[1].tokens))
      } else {
        (std::mem::take(&mut parts[0].tokens).unwrap(), None)
      }
    };

    // Определяем тип изменяемости у левой части выражения
    let structureMutabilityType: StructureMut = match structureNameTokens.get(1)
    {
      None => match rightValue
      { // Если нет флага изменяемости и правой части
        None => StructureMut::Final,
        Some(_) => StructureMut::Constant
      }
      Some(mutabilityType) =>
      { // Если есть флаг изменяемости
        match mutabilityType.getDataType()
        {
          TokenType::DoubleTilde => StructureMut::Dynamic,
          TokenType::Tilde => StructureMut::Variable,
          _ => return false // Это что-то другое, а не линейная запись
        }
      }
    };

    //
    structureName = structureNameTokens[0].getData().toString().unwrap(); // Имя точно есть
    structureMutability = structureMutabilityType;
    structureType = match structureTypeTokens
    {
      None => StructureType::None,
      Some(structureTypeTokens) =>
        structureTypeTokens[0].getStructureTypeSimple()
    };
  };

  drop(leftValue);

  // Получаем родительскую структуру;
  // Ищем в родительской структуре, есть ли там похожая на structureName
  let structureLink: Option< Arc<RwLock<Structure>> > =
  {
    parentLink.read().unwrap()
      .getStructureByName(&structureName)
  };

  match structureLink
  {
    Some(structureLink) =>
    { // Если мы нашли структуру, то значит работаем уже с существующей структурой
      let parent: RwLockReadGuard<Structure> = parentLink.read().unwrap();

      let structureMut: StructureMut =
      {
        let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
        structure.mutable.clone()
      };

      match structureMut
      {
        StructureMut::Constant => {} // Константные структуры изменить нельзя
        StructureMut::Final | StructureMut::Variable | StructureMut::Dynamic =>
        { // Всё остальное изменить можно
          parent.structureOp(
            structureLink,
            opType,
            structureMut,
            rightValue.unwrap_or_default()
          );
          //
        }
      }

      //
      return true;
    }
    None =>
    { // Если мы не нашли структуру, то создаём новую
      // и работаем с правой частью выражения

      // Закидываем новую структуру в родительскую структуру
      let parentStructure: RwLockWriteGuard<Structure> = parentLink.write().unwrap();

      // Вычисляем правое выражение?
      if structureMutability != StructureMut::Final
      { 
        let hasTokens: bool = rightValue.is_none();
        let mut value: Token = parentStructure.expression(&mut rightValue.unwrap());
        if structureType == StructureType::None
        { // Тип вычисляется если он не был изначально определён;
          // Вычисляется он по типу из результата правой части выражения
          structureType = value.getStructureType();
        } else
        if structureMutability == StructureMut::Dynamic
        { // Тип вычисляется, если флаг изменяемости Dynamic
          // Вычисляется он по типу из результата правой части выражения
          structureType = value.getStructureType();
        } else 
        { // Требуется выполнить преобразование в указанный тип данных
          Structure::normalizeToken(&mut value, structureType.clone());
        }

        //
        rightValue = if hasTokens {
          None
        } else {
          Some(vec![value])
        };
      }

      // Создаём структуру
      let newStructureLink: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
        Some(structureName),
        structureMutability,
        structureType.clone(),
        Some(vec![
          Arc::new(RwLock::new(
            Line {
              tokens: rightValue.clone(),
              indent: None,
              lines:  None,
              parent: None // todo Назначить родителя?
            }
          ))
        ]),
        None
      )));

      // ABI-композит String: .pointer/.length поверх исходного токена
      if structureType == StructureType::String
      {
        if let Some(valueToken) = rightValue.as_ref().and_then(|tokens| tokens.first())
        {
          if let Some(fields) = bridge::stringFields(valueToken)
          {
            let newStructure: RwLockReadGuard<Structure> = newStructureLink.read().unwrap();
            for field in fields { newStructure.pushStructure(field); }
          }
        }
      }

      //
      parentStructure.pushStructure(newStructureLink);
      return true;
    }
  }
  
  false
}

// =================================================================================================

/// Эта функция ищет структуры;
///
/// Это может быть:
/// - Вложенная структура (array/vector/list ...)   todo необходимо вынести в отдельный метод
/// - Линейное выражение (a = 10)
/// - Условный блок (if/elif/else)   todo необходимо вынести в отдельный метод
pub(super) fn searchStructure(line: &RwLockReadGuard<Line>, parentLink: Arc<RwLock<Structure>>, lineIndex: *mut usize) -> bool
{
  let lineTokens: &Vec<Token> = // Ссылка на токены линии
    match &line.tokens
    {
      None => return false, // Если в линии нет токенов, то мы её не читаем
      Some(tokens) => { tokens }
    };
  let lineTokensLength: usize = lineTokens.len(); // размер токенов линии

  let lineLines: Option< Vec< Arc<RwLock<Line>> > > = line.lines.clone(); // Вложенные линии

  // -----------------------------------------------------------------------------
  // Многострочный анонимный FFI-блок:
  //
  //   [ffi]
  //   { ... }              ← эта линия
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
    && isPrevLineFfiTag(&parentLink, unsafe{ *lineIndex })
  {
    let newStructureLink: Arc<RwLock<Structure>> = Arc::new(RwLock::new(
      Structure::new(
        None,                          // анонимный
        StructureMut::Constant,
        StructureType::Method,
        lineLines.clone(),
        Some(parentLink.clone())
      )
    ));
    newStructureLink.write().unwrap().isFfiBlock = true;
    parentLink.write().unwrap()
      .pushStructure(newStructureLink.clone());
    // readLines сам пушнет scope, потому что isFfiBlock=true.
    readLines(newStructureLink);
    return true;
  }

  // Дальше идёт старая логика: пустые линии не интересуют.
  if lineTokensLength == 0
  {
    return false; // Если в линии нет токенов, то мы её не читаем
  }

  let firstTokenType: &TokenType = lineTokens[0].getDataType(); // Тип первого токена в строке

  // -----------------------------------------------------------------------------
  // Многострочный анонимный FFI-блок:
  //
  //   [ffi]
  //   { ... }              ← эта линия
  //
  // В tokenizer'е линия `{ ... }` (без ничего перед `{`) теперь выходит как
  // `tokens = None`, `lines = Some([внутренние линии])` (после фикса
  // токенайзера). Либо `tokens = Some(vec![])` — пустой список, если
  // пред-фиксовый фикс не сработал. В обоих случаях Word/SquareBracketBegin
  // ветки не сработают — ловим этот случай отдельно: если предыдущая
  // линия — `[ffi]`, а текущая — пустая/None-токены с вложением,
  // выполняем её как анонимный FFI-блок.
  // -----------------------------------------------------------------------------
  if line.tokens.is_none() || lineTokens.is_empty()
    && lineLines.is_some()
    && isPrevLineFfiTag(&parentLink, unsafe{ *lineIndex })
  {
    let newStructureLink: Arc<RwLock<Structure>> = Arc::new(RwLock::new(
      Structure::new(
        None,                          // анонимный
        StructureMut::Constant,
        StructureType::Method,
        lineLines.clone(),
        Some(parentLink.clone())
      )
    ));
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
        let newStructureLink: Arc<RwLock<Structure>> = Arc::new(RwLock::new(
          Structure::new(
            None,                          // анонимный
            StructureMut::Constant,
            StructureType::Method,
            Some(innerLines),
            Some(parentLink.clone())
          )
        ));
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

        // Если идёт метод: `[ffi] name ( params ) [-> result]`
        if isMethod
        {
          parameters =
            if let Some(lines) = &restTokens[1].lines
            {
              let mut result: Vec<(Bytes, StructureType)> = Vec::new();
              for lineLink in lines
              {
                let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
                let paramTokens: Vec<Token> = line.tokens.clone().unwrap_or_default();
                result.extend(
                  parentLink.read().unwrap().getStructureParameters(&paramTokens)
                );
              }
              Some(result)
            } else {
              None
            };
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
            StructureType::Method,
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

        newStructure.result = match newStructureResultType
        {
          Some(t) => Some(Token::newEmpty(*t)),
          None => None
        };

        let newStructureLink: Arc<RwLock<Structure>> =
          Arc::new(RwLock::new(newStructure));
        parentLink.write().unwrap()
          .pushStructure(newStructureLink.clone());

        // Для метода (с круглыми скобками) body ждёт вызова — не выполняем.
        // Для именованного блока (без скобок) body выполняется прямо сейчас,
        // под удерживаемым FFI scope. readLines сам пушнет scope, потому
        // что isFfiBlock=true, так что здесь enterFfiBlock НЕ нужен.
        if !isMethod
        {
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
    // это значит, что это либо структура, либо линейная запись
    match lineLines
    {
      Some(lineLine) =>
      { // Если в линии есть вложение, то это структура с вложением
        match lineTokens[0].getData().toString()
        { // Первый токен - имя структуры
          None => {}
          Some(newStructureName) =>
          { // получаем имя структуры
            let mut newStructureResultType: Option<&TokenType> = None; // результат структуры
            let mut parameters: Option< Vec<(Bytes, StructureType)> > = None; // параметры структуры
            match lineTokensLength > 1 && *lineTokens[1].getDataType() == TokenType::CircleBracketBegin
            {
              true =>
              { // Если токенов > 1 и 1 токен это TokenType::CircleBracketBegin 
                // значит это вариант параметры + возможно результат

                // Получаем параметры структуры
                parameters =
                  if let Some(lines) = &lineTokens[1].lines
                  { // Берём первую линию внутри скобок (там обычно перечислены параметры)
                    let mut result: Vec<(Bytes, StructureType)> = Vec::new();
                    for lineLink in lines
                    { // Берём вложенные токены в TokenType::CircleBracketBegin 
                      // получаем параметры из этих токенов, давая доступ к родительским структурам
                      let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
                      let paramTokens: Vec<Token> = line.tokens.clone().unwrap_or_default();
                      result.extend(
                        parentLink.read().unwrap().getStructureParameters(&paramTokens)
                      );
                    }
                    Some(result)
                  } else {
                    None
                  };

                // Если > 3 (т.е name () -> result)
                // то значит это результат структуры 
                // todo: Может быть объединено с блоком ниже
                match lineTokensLength > 3 &&
                  *lineTokens[2].getDataType() == TokenType::Pointer &&
                  *lineTokens[3].getDataType() != TokenType::None
                {
                  false => {} // если результата не было, то просто пропускаем
                  true =>
                    { // в таком случае просто читаем тип результата структуры
                      newStructureResultType = Some(lineTokens[3].getDataType());
                    }
                }

                //
              }
              false =>
              { // В этом случае это вариант только с результатом структуры
                match lineTokensLength > 2 &&
                  *lineTokens[1].getDataType() == TokenType::Pointer &&
                  *lineTokens[2].getDataType() != TokenType::None
                {
                  false => {} // если результата не было, то просто пропускаем
                  true =>
                  { // в таком случае просто читаем тип результата структуры
                    newStructureResultType = Some(lineTokens[2].getDataType());
                  }
                }
                //
              }
            } // Если параметров и результата не было, то просто пропускаем

            // Проверка блока тегов [ffi]
            // todo Нет [ffi] {} т.е. это проверка если строка сверху но нет в 1 строку.
            let isFfi: bool =
              match unsafe{*lineIndex} == 0
              { true => false, false =>
              {
                match parentLink.read().unwrap().lines.clone()
                { None => false, Some(siblingLines) =>
                {
                  let prevLine: RwLockReadGuard<Line> = siblingLines[unsafe{*lineIndex} - 1].read().unwrap();
                  match &prevLine.tokens
                  { None => false, Some(prevTokens) =>
                  {
                    match prevTokens.is_empty()
                      || *prevTokens[0].getDataType() != TokenType::SquareBracketBegin
                      || prevLine.lines.is_some()
                    { true => false, false =>
                    {
                      match &prevTokens[0].lines
                      { None => false, Some(bracketLines) =>
                      {
                        let mut found: bool = false;
                        for bracketLine in bracketLines
                        {
                          let bracketLineGuard: RwLockReadGuard<Line> = bracketLine.read().unwrap();
                          match &bracketLineGuard.tokens
                          { None => {} Some(bracketTokens) =>
                          {
                            for token in bracketTokens
                            {
                              if *token.getDataType() == TokenType::Word
                                && token.getData().toString().unwrap_or_default() == "ffi"
                              { found = true; }
                            }
                          }}
                        }
                        found
                      }}
                    }}
                  }}
                }}
              }};
            
            // Cоздаём новую структуру
            let mut newStructure: Structure =
              Structure::new(
                Some(newStructureName),
                StructureMut::Constant, // todo По идее надо вычислять из синтаксиса
                StructureType::Method, // todo По идее надо вычислять из синтаксиса
                Some(lineLine),
                Some(parentLink.clone())
              );
            newStructure.isFfiBlock = isFfi;
            println!("isFfi {}",isFfi);

            // Ставим параметры структуры, если они были
            match &parameters
            { None => {} Some(parameters) =>
            {
              for parameter in parameters
              {
                newStructure.pushStructure(
                  Arc::new(RwLock::new(Structure::new(
                    parameter.0.toString(),
                    StructureMut::Constant, // todo По идее надо еще читать правила mut (в getStructureParameters)
                    parameter.1.clone(),
                    None,
                    None,
                  )))
                );
              }
            }}

            // Ставим результат структуры, если он есть
            newStructure.result = match newStructureResultType
            {
              Some(newStructureResultType) =>
                Some( Token::newEmpty(*newStructureResultType) ),
              None => None,
            };

            // Запоминаем, была ли это форма МЕТОДА (с круглыми скобками) или
            // просто именованного блока. Для метода body ждёт вызова `name()`,
            // для именованного блока body выполняется СРАЗУ — иначе его никто
            // никогда не выполнит (никто не пишет `name` без скобок для вызова).
            let isMethodForm: bool = parameters.is_some();

            let newStructureLink: Arc<RwLock<Structure>> =
              Arc::new(RwLock::new(newStructure));
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
            if isFfi && !isMethodForm
            {
              readLines(newStructureLink);
            }
            //            readLines(
            //              parentLink.read().unwrap()
            //                .getStructureByName(&newStructureName).unwrap(), // todo: плохой вариант, можно лучше
            //            );
            return true;
          }
        }
      }
      None =>
      { // Это линейная запись
        return linearStructure(lineTokens, parentLink);
      }
    }
  } else
  // В том случае, если это не структура и не линейная запись, 
  // мы видим TokenType::Question в начале строки и есть вложения у этой линии, 
  // то это условное вложение
  if *firstTokenType == TokenType::Question && !lineLines.is_none()
  { // Условное вложение запускает код внутри себя, в том случае если её условное выражение = true;
    // если условное выражение = false, то условное вложение не запускается, 
    // но может продолжить запускать блоки ниже, если такие там есть.
    // в этом моменте мы точно уверены что нашли первое условное вложение
    let mut conditions: Vec< Arc<RwLock<Line>> > = Vec::new();
    let mut saveNewLineIndex: usize = 0;  // сдвиг вниз на сколько условных блоков мы увидели
    { // теперь мы ищем все условные вложения ниже
      let lines: Option< Vec< Arc<RwLock<Line>> > > =
      {
        parentLink.read().unwrap() // Родительская структура
          .lines.clone()           // Родительские линии
      };
      match lines
      { None => {} Some(lines) =>
      {
        let linesLength: usize = lines.len(); // Количество линий родительской структуры
        { // Смотрим линии внизу
          let mut i: usize = unsafe{*lineIndex};
          while i < linesLength
          { // Если line index < lines length, то читаем вниз линии,
            // и если там первый токен не имеет TokenType::Question,
            // или количество токенов == 0, то только в этом случае break;
            // это будет означать, что мы нашли все возможные условные блоки.
            let lineBottomLink: Arc<RwLock<Line>> = lines[i].clone(); // ссылка на нижнюю линию
            { // берём нижнюю линию на чтение
              let bottomLine: RwLockReadGuard<Line> = lineBottomLink.read().unwrap();
              match &bottomLine.tokens
              { // Выходим если линия пустая
                None => { break; }
                Some(tokens) =>
                {
                  match *tokens[0].getDataType() != TokenType::Question
                  { // Выходим если в начале линии нет TokenType::Question
                    true  => { break; }
                    false => {}
                  }
                }
              }
            }
            // Если мы не вышли, значит это условный блок;
            // значит мы его добавляем
            conditions.push(lineBottomLink);
            i += 1;
          }
        }
        // В данном месте мы точно уверенны
        // что conditions.len() > 1 из-за первого блока
        saveNewLineIndex = conditions.len()-1;
        //
      }}
      //
    }
    // После нахождения всех возможных условных блоков,
    // начинаем читать их условия и выполнять
    let mut conditionTruth: bool = false; // заранее создаём true/false ячейку
    for conditionLink in &mut conditions
    { // Итак, мы читаем ссылки на условия в цикле;
      // после чего мы берём само условие на чтение
      let condition: RwLockReadGuard<Line> = conditionLink.read().unwrap();
      match &condition.tokens
      { None => {} Some(tokens) =>
      {
        match tokens.len() > 1
        {
          true =>
          { // Если условие больше чем просто один токен TokenType::Question,
            // то значит там обычное if/elif условие
            { // проверяем верность условия;
              let mut conditionTokens: Vec<Token> = tokens.clone(); // todo: no clone ? fix its please
              // Удаляем TokenType::Question токен
              conditionTokens.remove(0);
              // И проверяем
              conditionTruth =
              { // Получаем string ответ от expression, true/false
                let expressionResult: Option<String> =
                  parentLink.read().unwrap() // для этого берём родительскую линию;
                    .expression(&mut conditionTokens)
                    .getData().toString(); // и её токены.
                // Итоговый boolean результат
                match expressionResult
                {
                  Some(expressionResult) => { expressionResult == "1" }
                  None => { false }
                }
              };
            }
            // Если условие верно
            match conditionTruth
            { false => {} true =>
            { // Создаём новую временную структуру условного блока
              let structure: Arc<RwLock<Structure>> =
                Arc::new(RwLock::new(
                  Structure::new(
                    Some(String::from("if-elif")),
                    StructureMut::Constant,
                    StructureType::Method, // todo может быть что-то другое ?
                    condition.lines.clone(),
                    Some(parentLink)
                  )
                ));
              // После создания, читаем эту структуру
              drop(condition);
              readLines(structure);
              break; // end
            }}
          }
          // В случае если в токенах условия просто TokenType::Question,
          // значит это else блок
          false => if !conditionTruth
          { // создаём новую временную структуру условного блока
            let structure: Arc<RwLock<Structure>> =
              Arc::new(RwLock::new(
                Structure::new(
                  Some(String::from("else")),
                  StructureMut::Constant,
                  StructureType::Method, // todo может быть что-то другое ?
                  condition.lines.clone(),
                  Some(parentLink)
                )
              ));
            // После создания, читаем эту структуру
            drop(condition);
            readLines(structure);
            break; // end
          }
        }
        //
      }}
      //
    }

    // и только после прочтения всех блоков, 
    // мы можем сдвигать указатель ниже
    unsafe{*lineIndex += saveNewLineIndex}
    return true;
  }
  false
}

// =================================================================================================

/// Основная структура; В неё вкладываются остальные;
/// В эту структуру будут переданы стартовые параметры;
/// Неизменяемая; Действует во время всей жизни программы;
pub static MainStructure: LazyLock< Arc<RwLock<Structure>> > =
  LazyLock::new(|| {
    Arc::new(
      RwLock::new(
        Structure::new(
          Some(String::from("main")),
          StructureMut::Constant,
          StructureType::Method,
          None,
          None,
        )
      )
    )
  });

// =================================================================================================

/// Это основная функция для парсинга строк;
/// Она разделена на подготовительную часть, и часть запуска readLine()
pub fn parseLines(tokenizerLinesLinks: Vec< Arc<RwLock<Line>> >) -> ()
{ // Начинается подготовка к запуску
  #[cfg(not(target_family = "wasm"))]
  match unsafe{_debugMode}
  { false => {} true  =>
    {
      logSeparator("Preparation");
    }}

  { // Присваиваем в главную структуру
    let mut main: RwLockWriteGuard<Structure> = MainStructure.write().unwrap();

    // Присваиваем линии от Tokenizer
    main.lines = Some(tokenizerLinesLinks);

    // argc
    main.pushStructure(
      Arc::new(RwLock::new(Structure::new(
        Some(String::from("argc")),
        StructureMut::Constant, // Неизменяемая;
        StructureType::Usize,   // Не может быть меньше 0
        // В линии структуры
        Some(vec![
          Arc::new(RwLock::new( // добавляем линию с 1 токеном
            Line
            {
              tokens: Some(vec![
                Token::new(
                  TokenType::UInt,
                  Bytes::new( unsafe{_argc.to_string()} )
                )
              ]),
              indent: None,
              lines:  None,
              parent: None
            }
          ))
        ]),
        Some( MainStructure.clone() ), // Ссылаемся на родителя
      )))
    );

    // argv
    let mut argv: Vec< Arc<RwLock<Line>> > = Vec::new();
    for a in unsafe{&_argv}
    {
      argv.push(
        Arc::new(RwLock::new( // Добавляем линию с 1 токеном
          Line
          {
            tokens: Some(vec![
              Token::new(
                TokenType::String,
                Bytes::new( String::from(a) )
              )
            ]),
            indent: None,
            lines: None,
            parent: None
          }
        ))
      );
    }
    main.pushStructure(
      Arc::new(RwLock::new(Structure::new(
        Some(String::from("argv")),
        StructureMut::Constant, // Неизменяемая;
        StructureType::List,    // Список;
        Some(argv),             // В линии структуры добавляем все argv линии;
        Some( MainStructure.clone() ),  // ссылаемся на родителя
      )))
    );
  }

  // Выводим arch & argv
  #[cfg(not(target_family = "wasm"))]
  unsafe
  {
    match _debugMode
    { false => {} true =>
    {
      log("ok", &format!("argc [{}]", _argc));
      match _argc > 0
      { false => {} true =>
      {
        log("ok", &format!("argv {:?}", _argv));
      }}
    }}
  }

  // Подготовка закончена, читаем линии
  #[cfg(not(target_family = "wasm"))]
  let startTime: Instant = Instant::now(); // Получаем текущее время для debug замера
  #[cfg(not(target_family = "wasm"))]
  match unsafe{ _debugMode }
  { false => {} true  =>
  {
    logSeparator("Interpretation");
  }}
  
  // Передаём ссылку на структуру и запускаем
  readLines(MainStructure.clone());
  // Далее идут debug замеры
  #[cfg(not(target_family = "wasm"))]
  match unsafe{_debugMode}
  { false => {} true =>
  {
    let endTime:  Instant  = Instant::now();    // Получаем текущее время
    let duration: Duration = endTime-startTime; // Получаем сколько всего прошло
    logSeparator("End");
    log("ok",&format!("Parser duration [{:?}]",duration));
  }}
}
/// Эта функция занимается чтением блоков по ссылке на них
/// todo: исправить переполнение стека
pub fn readLines(structureLink: Arc<RwLock<Structure>>) -> ()
{ // Получаем сколько линий вложено в структуру,
  // а также индекс чтения строк (у каждой структуры он свой, чтобы не путать чтение)

  // FFI-обёртка: если эта структура создана из `[ffi] { ... }` блока,
  // пушим слот в FFI scope-стек на всё время чтения её тела. Это
  // покрывает ВСЕ пути, по которым блок может быть выполнен:
  //   - анонимный `[ffi] { ... }` (выполняется прямо в searchStructure,
  //     но и оттуда можно звать readLines);
  //   - метод `[ffi] test() { ... }` (вызывается как функция в procedure.rs
  //     → readLines(calledStructureLink));
  //   - именованный блок `[ffi] name { ... }` (тоже вызывается).
  // Scope сам создаётся лениво в scopeStack::ensureFfiScope() при первом
  // FFI-вызове внутри блока — точно как просит issue #79/#84.
  let isFfi: bool = structureLink.read().unwrap().isFfiBlock;
  if isFfi
  {
    scopeStack::enterFfiBlock();
  }

  let (lineIndex, linesLength): (*mut usize, usize) =
  {
    let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap(); // Читаем структуру
    (
      { &structure.lineIndex as *const usize as *mut usize }, // Возвращаем ссылку на этот индекс
      match &structure.lines
      {
        None => return, // Если нет линий, то нет смысла читать
        Some(lines) =>
        { // Если линии есть, значит получаем сколько их необходимо прочитать
          lines.len()
        }
        //
      }
    )
    //
  };

  // Выполнение программы происходит до тех пор,
  // пока не будет всё прочитано, либо 
  // пока не будет вызван _exitCode на true
  let mut lineLink: Arc< RwLock<Line> >;

  while unsafe{_exit == false} && unsafe{*lineIndex < linesLength}
  { // Если мы читаем строки, то создаём сразу ссылку на текущую линию
    lineLink =
    { // Получаем её через чтение текущей структуры;
      // Берём линию по индексу линии (она точно будет, поскольку выше мы это проверили)
      let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
      let lines: &Vec< Arc<RwLock<Line>> > = structure.lines.as_ref().unwrap();
      lines[unsafe{ *lineIndex }].clone() // Клонируем нужную линию по индексу
    };
    let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
    // После чего проверяем, если линия пустая на токены, то не читаем и идём дальше
    match line.tokens.is_none()
    { false => {} true =>
    {
      unsafe{*lineIndex += 1}
      continue;
    }}
    // Если всё хорошо, то начинаем читать через специальные функции;
    // Ищем структуры
    match !searchStructure(&line, structureLink.clone(), lineIndex)
    { false => {} true =>
    { // Читаем return

      match !searchReturn(&line, structureLink.clone())
      { false => {} true =>
      { // Ищем линейные выражения

        let tokens: &mut Vec<Token> =
          &mut line
            .tokens.clone() // Клонируем токены, для сохранения возможности повторного запуска
            .unwrap_or_default(); // todo плохо
        structureLink.read().unwrap()
          .expression(tokens);
      }}
    }}
    // Идём дальше
    unsafe{*lineIndex += 1}
  }
  // Сбрасываем указатель линий для текущей структуры на 0
  // Для того чтобы можно было запускать повторно
  unsafe{*lineIndex = 0}

  // FFI-обёртка: снимаем свой слот со стека.
  // Внутри scope дропается (если был создан) вместе с библиотеками
  // и AllocatedMemory, привязанными к нему. Делаем это ПОСЛЕ сброса
  // lineIndex, чтобы можно было перезапустить блок — повторный запуск
  // снова пушнет свой слот.
  if isFfi
  {
    scopeStack::exitFfiBlock();
  }
}

// =================================================================================================