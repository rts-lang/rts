use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::Ordering;
use crate::{_argc, _argv, _exit};
use crate::parser::bytes::Bytes;
use crate::parser::structure::ffi::bridge;
use crate::parser::structure::ffi::scopeStack;
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::primitives::{isPrimitiveName, ArgsPrimitives, MethodsPrimitives, TypesPrimitives};
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::{Token};
use crate::tokenizer::types::tokenType::{TokenType};
use crate::tokenizer::tools::splitByType::splitByType;
use crate::parser::structure::structureType::StructureType;
use crate::tokenizer::tokenizer::readTokensSimple;
// =================================================================================================

// Предоставляет механизмы для парсинга токенов,
// что позволяет запускать получившиеся структуры.

// =================================================================================================

/// Проверяет, что переданный dataType является математическим оператором.
const fn isMathOperator(dataType: TokenType) -> bool
{
  matches!(dataType,
    // `=` — присваивание по области видимости (ищет вверх);
    // `:=` — создание в текущей области (вверх НЕ ищет).
    TokenType::Equals | TokenType::Creation
    /* | // =
    todo Может плохо работать с #85, нужен контроль
    TokenType::UnaryPlus      | // ++
    TokenType::PlusEquals     | // +=
    TokenType::UnaryMinus     | // --
    TokenType::MinusEquals    | // -=
    TokenType::MultiplyEquals | // *=
    TokenType::DivideEquals   | // /=
    TokenType::ModuloEquals   | // %=
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

/// Проверяет, является ли предыдущая линия (по индексу `lineIndex - 1`)
/// тегом `[ffi]`. Используется для многострочной формы
///   [ffi]
///   { ... }
/// где вторая линия сама по себе не знает про тег.
fn isPrevLineFfiTag(parentLink: &Arc<RwLock<Structure>>, lineIndex: usize) -> bool
{
  // todo desc
  if lineIndex == 0 {
    return false;
  }

  // todo desc
  let prevLineOpt: Option< Arc<RwLock<Line>> > = parentLink.read().unwrap()
    .lines.as_ref()
    .and_then(|lines| lines.get(lineIndex - 1).cloned());

  // todo desc
  let prevLine: Arc<RwLock<Line>> = if let Some(line) =
    prevLineOpt { line }
    else { return false };

  // todo desc
  let prevGuard: RwLockReadGuard<Line> = prevLine.read().unwrap();
  let prevTokens: &[Token] = if let Some(tokens) =
    prevGuard.tokens.as_ref() { tokens }
    else { return false };
  
  // Многострочная форма: ровно один токен `[ffi]` на предыдущей строке.
  if prevTokens.len() != 1 {
    return false;
  }
  
  // todo desc
  isFfiTagToken(&prevTokens[0])
}

// =================================================================================================

/// Эта функция ищет return для структур `= value`;
/// 
/// Видно, что это не просто валяющееся значение.
fn searchReturn(
  line: &RwLockReadGuard<Line>,
  structureLink: Arc<RwLock<Structure>>,
  lineIndex: *mut usize
) -> bool
{
  let mut lineTokens: Vec<Token> =
  { // Читаемая линия, на которой мы сейчас находимся.
    if let Some(tokens) = 
      line.tokens.clone() { tokens } // Токены линии на которой мы сейчас находимся.
      else { return false } // Если в линии нет токенов, то мы её не читаем.
  };
  if lineTokens.is_empty() {
    return false // Если в линии нет токенов, то мы её не читаем.
  }

  // Что это: return (`:=`) или результат блока (`=`)?
  let firstType: TokenType = *lineTokens[0].getDataType();
  let isReturn: bool = firstType == TokenType::Equals || firstType == TokenType::Creation;
  if !isReturn {
    return false
  }

  // Удаляем первый токен (`=` или `:=`), чтобы он не мешался.
  lineTokens.remove(0);

  // Редактируемый родитель, поскольку мы собираемся присвоить значение его `result`.
  let newResultData: Token =
  {
    // Блок-выражение `= ? True { = 1 }`: `?` + условие заменяются результатом;
    //
    // Блоки выполняются без удержания блокировки на структуре.
    if let Some(replaced) = 
      replaceConditionExpression(&lineTokens, &line.lines, &structureLink, lineIndex) {
        lineTokens = replaced;
      }
    // Используем `expression`, чтобы получить результат выражения.
    let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
    structure.expression(&mut lineTokens)
  };

  // Куда писать `result`:
  //
  // - `:=` => Возврат результата только именной структуры.
  //           Поднимаемся через `parent` до первого `name.is_some()`.
  //
  // - `=` => Пишем в текущую `structureLink` - т.е. текущую структуру.
  let targetLink: Arc<RwLock<Structure>> = if firstType == TokenType::Creation
  {
    let mut current: Arc<RwLock<Structure>> = structureLink.clone();
    loop
    {
      let hasName: bool = current.read().unwrap().name.is_some();
      if hasName {
        break
      }
      let parent: Option<Arc<RwLock<Structure>>> = current.read().unwrap().parent.clone();
      if let Some(parent) = parent {
        current = parent;
      } else
      { // Дошли до верха (main, noname) — `:=` игнорируется.
        return false 
      }
    }
    current
  } else {
    structureLink.clone()
  };

  let mut structure: RwLockWriteGuard<Structure> = targetLink.write().unwrap();

  // Структура ожидает какой-то тип в результате,
  // либо это может быть `TokenType:None`. Но мы просто будем менять `data`.

  if structure.result.is_some()
  { // Вариант, в котором результат ожидает возвращение определённого типа данных.
    if let Some(structureResult) = &mut structure.result
    { // Присваиваем новую data результату.
      structureResult.setData( newResultData.getData() );
    }
  } else
  { // Вариант, в котором тип результата был не указан.
    //
    // Используем `expression`, чтобы получить результат выражения.
    //
    // Присваиваем новый результат.
    structure.result = Some(newResultData);
  }

  // Всё успешно, это был результат.
  true
}

// =================================================================================================

/// Читает линейную запись.
fn linearStructure(lineTokens: &[Token], parentLink: Arc<RwLock<Structure>>, isBlockValue: bool) -> bool
{
  // Получаем тип операции.
  let operatorType: TokenType = lineTokens.iter().find_map(|token| {
    if isMathOperator( *token.getDataType() ) {
      Some(*token.getDataType())
    } else {
      None
    }
  }).unwrap_or(TokenType::None);

  // Получаем левую и правую часть.
  let leftValue: Vec<Token>;
  let mut rightValue: Option< Vec<Token> > = None;
  {
    if operatorType == TokenType::None
    { // Операции не было.
      leftValue = std::mem::take(&mut lineTokens.to_owned()); // todo: Тут точно клонирование ?
    }
    else
    {// Операция есть.
      let mut parts: Vec<Line> = splitByType(lineTokens.to_owned(), &[operatorType]); // todo: Тут точно клонирование ?

      leftValue = std::mem::take(&mut parts[0].tokens).unwrap();
      rightValue = std::mem::take(&mut parts[1].tokens);
    }
  }

  let structureName: String;
  let structureMutability: StructureMut;
  let mut structureType: StructureType;
  { // Определяем тип данных у левой части выражения.
    let (structureNameTokens, structureTypeTokens): (Vec<Token>, Option< Vec<Token> >) =
    {
      let mut parts: Vec<Line> = splitByType(leftValue.clone(), &[TokenType::Colon]);
      if parts.len() == 2 {
        (std::mem::take(&mut parts[0].tokens).unwrap(), std::mem::take(&mut parts[1].tokens))
      } else {
        (std::mem::take(&mut parts[0].tokens).unwrap(), None)
      }
    };

    // Определяем тип изменяемости у левой части выражения.
    let structureMutabilityType: StructureMut = 
      if let Some(mutabilityType) = structureNameTokens.get(1)
      { // Если есть флаг изменяемости.
        match mutabilityType.getDataType()
        {
          TokenType::DoubleTilde => StructureMut::Dynamic,
          TokenType::Tilde => StructureMut::Variable,
          _ => return false // Это что-то другое, а не линейная запись.
        }
      }
      else if rightValue.is_some() {
        StructureMut::Constant
      } else 
      { // Если нет флага изменяемости и правой части.
        StructureMut::Final
      };

    //
    structureName = structureNameTokens[0].getData().toString().unwrap(); // Имя точно есть.
    structureMutability = structureMutabilityType;
    structureType = if let Some(structureTypeTokens) = structureTypeTokens
    {
      // `a: U8 | String` — объединение типов (issue #59);
      // обычный `a: U8` даёт здесь такой же одиночный тип, как и раньше.
      StructureType::fromTypeTokens(&structureTypeTokens)
    } else
    {
      StructureType::None
    }
  };

  drop(leftValue);

  // Ищем структуру с таким именем. Здесь две разные области видимости:
  //
  // - `:=` => Создание в текущем области. Ищем только среди своих структур, наверх не идём.
  //
  // - `=` => Присваивание по области видимости: сначала свои структуры, потом до самого верха.
  //
  let isCreation: bool = operatorType == TokenType::Creation;

  // `:= None`— это явная очистка структуры из текущей области;
  // Это `примитив`, а не выражение.
  //
  // 
  // Не путать с `:= method()`, где `method` уже вернул `None`: 
  // в этом случае структура создаётся со значением `None` для хранения.
  //
  //
  // Различаем по токенам, а не по вычисленному значению:
  // - `a := None` => правая часть = `[Token(None)]` (один литерал) => Удалить.
  //
  // - `a := b` => правая часть = `[ident(b)]` (выражение) => Создать/перезаписать.
  //
  // - `a := method()` => правая часть = `[method, (, )]` => Создать/перезаписать.
  //
  // - `a = None` => оператор `=`, не `:=` => Записать `None` как значение.
  //
  //
  // Это разделение намерений между явным со стороны программиста и 
  // работой состояний внутри программы.
  //
  // Если правая часть пришла из блока-выражения `a := ? False { = 1 }`, то `None` — это
  // результат блока, а не литерал из кода: структуру `a` нужно создать, а не удалить.
  let isCreationNoneLiteral: bool =
    isCreation && !isBlockValue &&
    rightValue.as_ref()
      .map(|tokens| tokens.len() == 1 && *tokens[0].getDataType() == TokenType::None)
      .unwrap_or(false);
  let structureLink: Option< Arc<RwLock<Structure>> > =
  {
    let parent: RwLockReadGuard<Structure> = parentLink.read().unwrap();
    if isCreation
    { // Только свои, строго вниз.
      parent.getStructureByLink(&structureName)
    } else
    { // Свои, потом вверх до самого верха.
      parent.getStructureByScope(&structureName)
    }
  };

  // `:= None`: удаляем найденную локальную структуру.
  //
  //
  // Если структуры нет — ничего не делаем (нечего удалять).
  //
  // Если структура — примитив, то `removeStructure` внутри себя откажется её удалять, 
  // базовая часть языка останется на месте.
  //
  // Это safety-net: даже если проверка `isPrimitiveName` выше пропустит
  // имя, `removeStructure` всё равно проверит по списку примитивов.
  if isCreationNoneLiteral
  {
    if let Some(structureLink) = structureLink
    {
      parentLink.read().unwrap()
        .removeStructure(structureLink);
    }
    return true;
  }

  if let Some(structureLink) = structureLink
  { // Если мы нашли структуру, то значит работаем уже с существующей структурой.
    let parent: RwLockReadGuard<Structure> = parentLink.read().unwrap();

    let structureMut: StructureMut =
    {
      let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
      structure.mutable.clone()
    };

    match structureMut
    {
      StructureMut::Constant => {} // Константные структуры изменить нельзя.
      StructureMut::Final | StructureMut::Variable | StructureMut::Dynamic =>
      { // Всё остальное изменить можно.
        
        // `:=` отличается от `=` только тем, где искать структуру.
        //
        // Само присваивание делается то же самое, поэтому `:=` сводим к `=`
        // уже после того, как нужная структура найдена.
        let operator: TokenType = if operatorType == TokenType::Creation {
          TokenType::Equals
        } else {
          operatorType
        };

        parent.structureOp(
          structureLink,
          operator,
          structureMut,
          rightValue.unwrap_or_default()
        );
        //
      }
    }
  } else if isPrimitiveName(&structureName) {
    // Зарезервированное имя — новую структуру не создаём.
  } else if !isCreation && structureType == StructureType::None && rightValue.is_some()
  { // `=` без существующей структуры и без объявления типа (`:`) — скип.
    //
    // Это случай именно `a = X` (есть правая часть, но не `:=`): нет структуры =>
    // нечего менять => пропускаем. Создание — это `:=`.
    //
    // Но forward-declaration `a` (без правой части, без `:`, без `~`) — это
    // объявление структуры, которая ждёт первого `=`. Её мы создаём как Final,
    // поэтому условие `rightValue.is_some()` обязательно: иначе `a` без значения
    // пропустилось бы, и потом `a = X` не нашло бы её.
    //
    // `a: U8 = 300` — это объявление с типом, оно создаёт структуру через
    // `structureType != None` ниже. Сюда не попадает.
    //
    // Получается если нет структуры => нечего менять => пропускаем.
  } else
  { // `:=` без существующей структуры, либо `=` с явным типом (`:`),
    // либо forward-declaration `a` без значения — создаём новую.

    // Объединение типов: сам набор вариантов объявления;
    //
    // Хранится отдельно от structureType, потому что в структуре лежит
    // ровно один из вариантов, а объединение продолжает ограничивать
    // все следующие присваивания.
    let unionTypes: Option<Vec<StructureType>> = match structureType
    {
      StructureType::Union(ref variants) => Some(variants.clone()),
      _ => None
    };

    // Считаем правую часть только когда она реально есть;
    //
    // Правая часть бывает не у всех объявлений:
    //
    // - `a` => Final без значения, ждём первого `=` (станет Const);
    //
    // - `a: UInt` => То же с явным типом;
    //
    // - `a~` => Variable без значения, тип выведется при первом присваивании.
    //
    // todo `a~~` еще по идее? нужно проверить отдельно и вписать сюда все варианты.
    let hasValue: bool = rightValue.is_some();

    // Вычисляем правое выражение?
    if hasValue
    {
      // Что ждём от FFI-вызова справа:
      //
      // - `a: Usize = lib.f(x)` => Тип указан, он же тип возврата C-функции
      //   и в него кастуется результат (normalizeToken ниже);
      //
      // - `a = lib.f(x)` / `a~~ = lib.f(x)` => Тип слева получаем от правой части;
      //
      // - `a: U8 | String = lib.f(x)` => У объединения нет единственного ABI-типа,
      //   поэтому вызов читается как Infer, а вариант подбирается уже по значению.
      let expect: bridge::FfiExpect =
        if structureType == StructureType::None || unionTypes.is_some() ||
           structureMutability == StructureMut::Dynamic
        { bridge::FfiExpect::Infer } else
        { bridge::FfiExpect::Typed(structureType.clone()) };

      // Важно: правую часть считаем под блокировкой чтения, а не записи.
      //
      // Вычисление выражения может искать имя вверх по области видимости
      // (getStructureByScope) — а он берёт read() на каждого родителя, включая
      // текущий. Если бы read() приходился на структуру, которую мы уже держим
      // на записи, то std::sync::RwLock не перевзводим и процесс вставал бы
      // навсегда (0% CPU, sleeping). Старый код (tag 231206) считал ровно так же:
      // сначала expression, потом write на pushStructure.
      //
      // todo rewrite desc, должно быть по смыслу работы.
      let mut value: Token =
      {
        let parent: RwLockReadGuard<Structure> = parentLink.read().unwrap();
        parent.expressionWith(&mut rightValue.unwrap(), &expect)
      };

      // Динамический import(): правая часть выражения — не скаляр, а целая
      // под-структура (модуль со своими подструктурами).
      // 
      // Обычная ветка ниже (inference/normalizeToken/Line{tokens: Some(vec![value])}) 
      // рассчитана на скаляр, поэтому переносим (dataType, lines, structures) модуля
      // напрямую в новую структуру и выходим раньше.
      //
      // todo rewrite desc
      if *value.getDataType() == TokenType::Link
      {
        if let Some(markerName) = value.getData().toString()
        {
          if let Ok(moduleId) = markerName.parse::<usize>()
          {
            let moduleLink: Option< Arc<RwLock<Structure>> > =
            {
              let parent: RwLockReadGuard<Structure> = parentLink.read().unwrap();
              parent.getModuleById(moduleId)
            };
            if let Some(moduleLink) = moduleLink
            { // Если это модуль.
              
              let moduleGuard: RwLockReadGuard<Structure> = moduleLink.read().unwrap();
              let newStructureLink: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
                Some(structureName),
                structureMutability,
                moduleGuard.dataType.clone(),
                moduleGuard.lines.clone(),
                None
              )));
              *newStructureLink.read().unwrap().structures.write().unwrap() =
                moduleGuard.structures.read().unwrap().clone();
              drop(moduleGuard);

              parentLink.write().unwrap()
                .pushStructure(newStructureLink);
              return true;
              
              //
            }
          }
          //
        }
      }

      if let Some(variants) = unionTypes.clone()
      { // Объединение типов (issue #59): под значение выбирается один вариант.
        if structureMutability == StructureMut::Dynamic
        { // При полной мутабельности `~~` тип всё равно свободно меняется,
          // поэтому объединение можно указать, но оно ничего не ограничивает (#22).
          structureType = value.getStructureType();
        } else
        {
          structureType = Structure::normalizeUnion(&mut value,
            &StructureType::Union(variants)
          );
        }
      } else
      if structureType == StructureType::None
      { // Тип вычисляется если он не был изначально определён.
        // Вычисляется он по типу из результата правой части выражения.
        structureType = value.getStructureType();
      } else
      if structureMutability == StructureMut::Dynamic
      { // Тип вычисляется, если флаг изменяемости Dynamic.
        // Вычисляется он по типу из результата правой части выражения.
        structureType = value.getStructureType();
      } else 
      { // Требуется выполнить преобразование в указанный тип данных.
        Structure::normalizeToken(&mut value, structureType.clone());
      }

      //
      rightValue = Some(vec![value]);
    }

    // Создаём структуру.
    let newStructureLink: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
      Some(structureName),
      structureMutability,
      structureType.clone(),
      Some(vec![
        Arc::new(RwLock::new(Line {
          tokens: rightValue.clone(),
          lines: None
        }))
      ]),
      None
    )));

    // Сохраняем объединение, чтобы оно ограничивало и следующие присваивания (#59).
    if let Some(variants) = unionTypes
    {
      newStructureLink.write().unwrap().unionTypes = Some(variants);
    }

    // ABI-композит String: .pointer/.length поверх исходного токена.
    // Primitive(Text) тоже String на уровне ABI (stype при этом показывает примитив).
    if structureType.abiType() == StructureType::String
    {
      if let Some(valueToken) = rightValue.as_ref().and_then(|tokens| tokens.first())
      {
        if let Some(fields) = bridge::stringFields(valueToken)
        {
          let newStructure: RwLockReadGuard<Structure> = newStructureLink.read().unwrap();
          for field in fields
          {
            newStructure.pushStructure(field);
          }
        }
        //
      }
    }

    // Блокировку на запись берём только здесь — уже после того, как правая
    // часть посчитана. Так ничего не держит родителя на записи, пока
    // вычисляется выражение, и поиск вверх по области видимости остаётся
    // безопасным.
    parentLink.write().unwrap()
      .pushStructure(newStructureLink);
  }

  //
  true
}

// =================================================================================================

/// Одна ветка условной цепочки: `? условие { тело }`.
///
/// `condition == None` — это `else` (`? { тело }`), его условие всегда `True`.
struct ConditionBranch
{
  /// Токены условия.
  condition: Option< Vec<Token> >,
  
  /// Линии блока (структуры).
  body: Option< Vec< Arc<RwLock<Line>> > >
}

/// Читает ветки `elif`/`else` ниже текущей линии: линии, которые начинаются с `?`.
///
/// `else` (`? {}`) завершает цепочку: линия `?` после него начинает новую цепочку.
/// 
/// Сдвигает `lineIndex` на количество прочитанных линий, чтобы их не читали повторно.
fn collectConditionBranches(
  parentLink: &Arc<RwLock<Structure>>,
  lineIndex: *mut usize
) -> Vec<ConditionBranch>
{
  let mut branches: Vec<ConditionBranch> = Vec::new();
  {
    let parent: RwLockReadGuard<Structure> = parentLink.read().unwrap();
    let Some(lines) = &parent.lines else { return branches };
    for lineLink in lines.iter().skip( unsafe{ *lineIndex } + 1 )
    {
      let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
      let Some(tokens) = &line.tokens else { break };
      if tokens.is_empty() || *tokens[0].getDataType() != TokenType::Question
      { // Выходим, если в начале линии нет TokenType::Question.
        break;
      }
      // Если в токенах просто TokenType::Question, значит это else.
      let isElse: bool = tokens.len() == 1;
      branches.push(ConditionBranch
      {
        condition: if isElse { None } else { Some(tokens[1..].to_vec()) },
        body: line.lines.clone()
      });
      if isElse {
        break
      }
      //
    }
  }
  unsafe{ *lineIndex += branches.len() }
  branches
}

/// Выполняет тело ветки во временной структуре и возвращает её `result`;
/// 
/// Если `result` нет — пустой `None`-токен.
fn executeConditionBody(
  body: Option< Vec< Arc<RwLock<Line>> > >,
  parentLink: &Arc<RwLock<Structure>>
) -> Token
{
  let blockStructure: Arc<RwLock<Structure>> =
    Arc::new(RwLock::new(Structure::new(
      None,
      StructureMut::Constant,
      StructureType::Method,
      body,
      Some(parentLink.clone())
    )));
  readLines(blockStructure.clone());
  let result: Option<Token> = blockStructure.read().unwrap().result.clone();
  result.unwrap_or_else(|| Token::newEmpty(TokenType::None))
}

/// Единая логика `if`/`elif`/`else`: и для условного блока-statement, и для блока-выражения.
///
/// Ветки проверяются по порядку: блок запускается только по `True`;
/// `False` и `None` (пустота, #39) — идём к следующей ветке.
///
/// Возвращает `result` выполненной ветки (пустой `None`-токен, если `result` нет),
/// либо `None`, если ни одна ветка не подошла.
fn executeConditionChain(
  branches: Vec<ConditionBranch>,
  parentLink: &Arc<RwLock<Structure>>
) -> Option<Token>
{
  for mut branch in branches
  {
    if let Some(conditionTokens) = &mut branch.condition
    {
      let conditionTruth: bool =
      { // Блокировка на родителе живёт только в этом выражении.
        *parentLink.read().unwrap()
          .expression(conditionTokens)
          .getDataType() == TokenType::True
      };
      if !conditionTruth {
        continue
      }
    }
    return Some(executeConditionBody(branch.body, parentLink));
  }
  None
}

/// Читает условный блок (`if`/`elif`/`else`) как statement.
///
/// Первая ветка — текущая линия, остальные ищутся ниже.
/// 
/// Сдвигает `lineIndex` на количество пройденных блоков.
fn executeConditionStatement(
  lineTokens: &[Token],
  lineLines: &Option< Vec< Arc<RwLock<Line>> > >,
  lineIndex: *mut usize,
  parentLink: &Arc<RwLock<Structure>>
) -> bool
{
  let mut branches: Vec<ConditionBranch> = vec![ConditionBranch
  {
    // Если в токенах просто TokenType::Question, значит это else.
    condition: if lineTokens.len() > 1 { Some(lineTokens[1..].to_vec()) } else { None },
    body: lineLines.clone()
  }];
  // Если первый блок уже else, то цепочка закончена: `?` ниже начинает новую.
  if branches[0].condition.is_some()
  {
    branches.extend(collectConditionBranches(parentLink, lineIndex));
  }
  executeConditionChain(branches, parentLink);
  true
}

/// Читает `?` в токенах как блок-выражение `a := ? True { = 20 }`.
///
/// Выполняет цепочку: первая ветка — `?` из этих токенов и `bodyLines`
/// (вложения текущей линии), остальные (`elif`/`else`) — линии `?` ниже.
/// Возвращает токены, в которых `?` + условие заменены результатом цепочки.
/// `?` без условия — это else: блок выполняется безусловно.
///
/// Возвращает `None`, если блока-выражения нет — тогда это обычные токены.
fn replaceConditionExpression(
  tokens: &[Token],
  bodyLines: &Option< Vec< Arc<RwLock<Line>> > >,
  parentLink: &Arc<RwLock<Structure>>,
  lineIndex: *mut usize
) -> Option< Vec<Token> >
{
  let questionIndex: usize = tokens.iter().position(|t| *t.getDataType() == TokenType::Question)?;
  // Если после `?` нет условия, то это else: блок выполняется безусловно `a := ? { = 1 }`.
  let hasCondition: bool = questionIndex + 1 < tokens.len();

  let mut branches: Vec<ConditionBranch> = vec![ConditionBranch {
    condition: if hasCondition { Some(tokens[questionIndex + 1..].to_vec()) } else { None },
    body: bodyLines.clone()
  }];
  // Если первый блок уже else, то цепочка закончена: `?` ниже начинает новую.
  if hasCondition {
    branches.extend(collectConditionBranches(parentLink, lineIndex));
  }

  let result: Token = executeConditionChain(branches, parentLink)
    .unwrap_or_else(|| Token::newEmpty(TokenType::None));
  let mut replaced: Vec<Token> = tokens[..questionIndex].to_vec();
  replaced.push(result);
  Some(replaced)
}

/// Читает линейную запись; блок-выражение в правой части `a := ? True { = 20 }`
/// предварительно заменяется своим результатом.
fn linearLine(
  lineTokens: &[Token],
  lineLines: &Option< Vec< Arc<RwLock<Line>> > >,
  parentLink: Arc<RwLock<Structure>>,
  lineIndex: *mut usize
) -> bool
{
  match replaceConditionExpression(lineTokens, lineLines, &parentLink, lineIndex)
  {
    Some(tokens) => linearStructure(&tokens, parentLink, true),
    None => linearStructure(lineTokens, parentLink, false)
  }
}

// =================================================================================================

/// Эта функция ищет структуры;
///
/// Это может быть:
/// - Вложенная структура (array/vector/list ...)   todo необходимо вынести в отдельный метод
/// - Линейное выражение (a = 10)
/// - Условный блок (if/elif/else)
pub(super) fn searchStructure(line: &RwLockReadGuard<Line>, parentLink: Arc<RwLock<Structure>>, lineIndex: *mut usize) -> bool
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
    let newStructureLink: Arc<RwLock<Structure>> = 
      Arc::new(RwLock::new(Structure::new(
        None, // Анонимный
        StructureMut::Constant,
        StructureType::Method,
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
    let newStructureLink: Arc<RwLock<Structure>> = 
      Arc::new(RwLock::new(
      Structure::new(
        None, // Анонимный
        StructureMut::Constant,
        StructureType::Method,
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
            StructureType::Method,
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
    // это значит, что это либо структура, либо линейная запись.
    //
    // Если в токенах есть оператор (`:=` или `=`) — это линейная запись,
    // даже если есть вложенные линии (например `a := ? True { = 20 }`).
    let hasOperator: bool = lineTokens.iter()
      .any(|t| isMathOperator(*t.getDataType()));
    if hasOperator
    { // Линейная запись с оператором.
      return linearLine(lineTokens, &lineLines, parentLink, lineIndex);
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
        let isFfi: bool = if unsafe{ *lineIndex } == 0
        { false } else
        {
          if let Some(siblingLines) = &parentLink.read().unwrap().lines
          {
            let prevLine: RwLockReadGuard<Line> = siblingLines[unsafe{ *lineIndex } -1].read().unwrap();
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
                        if *token.getDataType() == TokenType::Word
                          && token.getData().toString().unwrap_or_default() == "ffi"
                        { found = true; }
                      }
                    }
                  }
                  found
                  //
                } else { false }
              }
              //
            } else { false }
          } else { false }
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
            StructureType::Method, // todo По идее надо вычислять из синтаксиса.
            Some(lineLines),
            Some(parentLink.clone())
          );
        newStructure.isFfiBlock = isFfi;

        // Ставим параметры структуры, если они были
        if let Some(parameters) = &parameters
        {
          for parameter in parameters
          {
            newStructure.pushStructure(
              Arc::new(RwLock::new(Structure::new(
                parameter.0.toString(),
                StructureMut::Constant, // todo По идее надо еще читать правила mut (в getStructureParameters).
                parameter.1.clone(),
                None,
                None
              )))
            );
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
    { // Это линейная запись.
      return linearLine(lineTokens, &lineLines, parentLink, lineIndex);
    }
  } else
  // В том случае, если это не структура и не линейная запись, 
  // мы видим `TokenType::Question` в начале строки и есть вложения у этой линии, 
  // то это условное вложение.
  if *firstTokenType == TokenType::Question && !lineLines.is_none()
  {
    return executeConditionStatement(lineTokens, &lineLines, lineIndex, &parentLink);
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
      StructureType::Method,
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
        dataType.clone(),
        Some(lines),
        Some( mainStructureLink.clone() ) // Ссылаемся на родителя.
      )))
    );
  }

  // -----------------------------------------------------------------------------------------------
  // Примитивы (базовые структуры языка).

  for (name, dataType) in TypesPrimitives
  {
    mainStructure.pushStructure(
      Arc::new(RwLock::new(Structure::new(
        Some(String::from(*name)),
        StructureMut::Constant, // Неизменяемая.
        dataType.clone(),
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
        StructureType::Method,
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
/// 
/// todo: исправить переполнение стека.
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
  if isFfi {
    scopeStack::enterFfiBlock();
  }

  let (lineIndex, linesLength): (*mut usize, usize) =
  {
    let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap(); // Читаем структуру.
    (
      // todo Тут есть вопросы по поводу адреса. Разве нельзя было просто ссылку или что-то безопаснее?
      { &structure.lineIndex as *const usize as *mut usize }, // Возвращаем ссылку на этот индекс.
      if let Some(lines) = &structure.lines
      {
        // Если линии есть, значит получаем сколько их необходимо прочитать.
        lines.len()
      } else { return } // Если нет линий, то не читаем.
    )
    //
  };

  // Выполнение программы происходит до тех пор,
  // пока не будет всё прочитано, либо
  // пока не будет вызван _exitCode на true.
  let mut lineLink: Arc< RwLock<Line> >;

  while !_exit.load(Ordering::Relaxed) && unsafe{ *lineIndex < linesLength }
  { // Если мы читаем строки, то создаём сразу ссылку на текущую линию.
    
    lineLink =
    { // Получаем её через чтение текущей структуры;
      // Берём линию по индексу линии (она точно будет, поскольку выше мы это проверили).
      let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
      let lines: &Vec< Arc<RwLock<Line>> > = structure.lines.as_ref().unwrap();
      lines[unsafe{ *lineIndex }].clone() // Клонируем нужную линию по индексу.
    };
    let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
    
    // После чего проверяем, если линия пустая на токены, то не читаем и идём дальше.
    if line.tokens.is_none()
    {
      unsafe{ *lineIndex += 1 }
      continue;
    }
    
    // Если всё хорошо, то начинаем читать через специальные функции.
    // Ищем структуры.
    if !searchStructure(&line, structureLink.clone(), lineIndex)
    { // Читаем return.

      if !searchReturn(&line, structureLink.clone(), lineIndex)
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
      // Если после этого у метода появился `result` — внутри блока был `:=`,
      // который поднялся до этого метода. Прерываем чтение.
      if structureLink.read().unwrap().result.is_some() {
        break
      }
    }

    // Идём дальше
    unsafe{ *lineIndex += 1 }
  }

  // Сбрасываем указатель линий для текущей структуры на 0
  // Для того чтобы можно было запускать повторно.
  unsafe{ *lineIndex = 0 }

  // FFI-обёртка: снимаем свой слот со стека.
  // Внутри scope дропается (если был создан) вместе с библиотеками
  // и AllocatedMemory, привязанными к нему. Делаем это ПОСЛЕ сброса
  // lineIndex, чтобы можно было перезапустить блок — повторный запуск
  // снова пушнет свой слот.
  if isFfi {
    scopeStack::exitFfiBlock();
  }
}

// =================================================================================================

/// Запускает код и возвращает основную структуру.
pub fn runCode(code: &str) -> Arc<RwLock<Structure>>
{
  let mut buffer: Vec<u8> = code.as_bytes().to_vec();
  runCodeBytes(&mut buffer)
}

/// Запускает код и возвращает основную структуру.
pub fn runCodeBytes(buffer: &mut Vec<u8>) -> Arc<RwLock<Structure>>
{
  parseLines( readTokensSimple(buffer) )
}

/// Запускает `.rt` файл и возвращает основную структуру.
///
/// В отличие от обёртки `runCode("m := import(\"…\")")` не опирается на
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
  let previousPath: Option<Arc<String>> = crate::_sourcePath.read().unwrap().clone();
  *crate::_sourcePath.write().unwrap() = Some(Arc::new(absolutePath));
  let result: Arc<RwLock<Structure>> = runCodeBytes(&mut buffer);
  *crate::_sourcePath.write().unwrap() = previousPath;
  result
}

// =================================================================================================
