/* /parser
  предоставляет механизмы для парсинга токенов,
  что позволяет запускать получившиеся структуры.
*/

pub mod bytes;
pub mod value;
pub mod uf64;
pub mod structure;
mod procedureCall;
mod functionCall;

use crate::{
  logger::*,
  _argc, _argv, _debugMode, _exit,
  parser::structure::*,
  tokenizer::{token::*, line::*},
  parser::bytes::Bytes
};

use std::{
  time::{Instant, Duration},
  sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use crate::tokenizer::splitByType;

/// Проверяет, что переданный dataType является математическим оператором
fn isMathOperator(dataType: TokenType) -> bool 
{
  matches!(dataType, 
    TokenType::Equals         | // =
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
  )
}

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
        let structure: RwLockReadGuard<'_, Structure> = structureLink.read().unwrap();
        structure.expression(&mut lineTokens)
      };

      let mut structure: RwLockWriteGuard<'_, Structure> = structureLink.write().unwrap();

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

/// Читает линейную запись
fn linearStructure(lineTokens: &Vec<Token>, parentLink: Arc<RwLock<Structure>>) -> bool 
{
  // Получаем тип операции
  let opType: TokenType = lineTokens.iter().find_map(|token| 
  {
    match isMathOperator( token.getDataType().clone() ) 
    {
      true => Some(token.getDataType().clone()),
      false => None,
    }
  }).unwrap_or(TokenType::None);
  
  // Получаем левую и правую часть
  let leftValue: Vec<Token>;
  let mut rightValue: Option< Vec<Token> > = None;
  {
    match opType == TokenType::None
    { 
      false => 
      {// Операция есть
        let mut parts: Vec<Line> = splitByType(lineTokens.clone(), &[opType.clone()]); // todo: Тут точно клонирование ?

        leftValue = std::mem::take(&mut parts[0].tokens).unwrap();
        rightValue = std::mem::take(&mut parts[1].tokens);
      }
      true => 
      { // Операции не было
        leftValue = std::mem::take(&mut lineTokens.clone()); // todo: Тут точно клонирование ?
      }
    }
  }
  
  let structureName: String;
  let structureMutability: StructureMut;
  let mut structureType: StructureType;
  { // Определяем тип данных у левой части выражения
    let (structureNameTokens, structureTypeTokens): (Vec<Token>, Option< Vec<Token> >) = 
    {
      let mut parts: Vec<Line> = splitByType(leftValue.clone(), &[TokenType::Colon]);
      match parts.len() == 2 
      {
        false => (std::mem::take(&mut parts[0].tokens).unwrap(), None),
        true => (std::mem::take(&mut parts[0].tokens).unwrap(), std::mem::take(&mut parts[1].tokens))
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
        structureTypeTokens[0].getDataType().toStructureType()
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
      let mut parentStructure: RwLockWriteGuard<Structure> = parentLink.write().unwrap();

      // Вычисляем правое выражение?
      match structureMutability == StructureMut::Final
      { true => {} false =>
      { 
        let hasTokens: bool = rightValue.is_none();
        let mut value: Token = parentStructure.expression(&mut rightValue.unwrap());
        match structureType == StructureType::None
        {
          true =>
          { // Тип вычисляется если он не был изначально определён;
            // Вычисляется он по типу из результата правой части выражения
            structureType = value.getDataType().toStructureType();
          }
          false =>
            match structureMutability == StructureMut::Dynamic
            {
              // Тип вычисляется, если флаг изменяемости Dynamic
              // Вычисляется он по типу из результата правой части выражения
              true => structureType = value.getDataType().toStructureType(),
              // Требуется выполнить преобразование в указанный тип данных
              false => value.setDataType( structureType.toTokenType() )
            }
        }
        //
        rightValue = match hasTokens
        { true => None, false =>
        {
          Some(vec![ value ])
        }}
      }}

      // Создаём структуру
      parentStructure.pushStructure(
        Structure::new(
          Some(structureName),
          structureMutability,
          structureType,
          Some(vec![
            Arc::new(RwLock::new(
              Line {
                tokens: rightValue,
                indent: None,
                lines:  None,
                parent: None // todo Назначить родителя?
              }
            ))
          ]),
          None
        )
      );
      
      //
      return true;
    }
  }
  
  false
}
/// Эта функция ищет структуры;
///
/// Это может быть:
/// - Вложенная структура (array/vector/list ...)
/// - Линейное выражение (a = 10)
/// - Условный блок (if/elif/else)
fn searchStructure(line: &RwLockReadGuard<Line>, parentLink: Arc<RwLock<Structure>>, lineIndex: *mut usize) -> bool
{
  let lineTokens: &Vec<Token> = // Ссылка на токены линии
    match &line.tokens
    {
      None => return false, // Если в линии нет токенов, то мы её не читаем
      Some(tokens) => { tokens }
    };
  let lineTokensLength: usize = lineTokens.len(); // размер токенов линии
  match lineTokensLength == 0
  {
    true => return false, // Если в линии нет токенов, то мы её не читаем
    false => {}
  }

  let firstTokenType: &TokenType = lineTokens[0].getDataType(); // тип первого токена в строке
  let lineLines: Option< Vec< Arc<RwLock<Line>> > > = line.lines.clone(); // вложенные линии

  if *firstTokenType == TokenType::Word
  { // если мы видим TokenType::Word в начале строки, 
    // это значит, что это либо структура, либо линейная запись
    match lineLines
    {
      Some(lineLine) =>
      { // Если в линии есть вложение, то это структура с вложением
        match lineTokens[0].getData().toString()
        { // первый токен - имя структуры
          None => {}
          Some(newStructureName) => 
          { // получаем имя структуры
            let mut newStructureResultType: Option<&TokenType> = None; // результат структуры
            let mut parameters: Option< Vec<Token> > = None; // параметры структуры
            match lineTokensLength > 1 && *lineTokens[1].getDataType() == TokenType::CircleBracketBegin
            {
              true => 
              { // если токенов > 1 и 1 токен это TokenType::CircleBracketBegin 
                // значит это вариант параметры + возможно результат
                /* todo fixTokenNesting
                match lineTokens[1].tokens.clone() 
                {
                  Some(mut lineTokens) => 
                  { // берём вложенные токены в TokenType::CircleBracketBegin 
                    // получаем параметры из этих токенов, давая доступ к родительским структурам
                    parameters = Some(
                      parentLink.read().unwrap() // читаем родительскую структуру
                        .getStructureParameters(&mut lineTokens) 
                    );
                  }
                  None => {}
                }
                */
                // если > 3 (т.е name () -> result)
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
              }  
              false => 
              { // в этом случае это вариант только с результатом структуры
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
              }
            } // если параметров и результата не было, то просто пропускаем

            // создаём новую структуру
            let mut newStructure: Structure = 
              Structure::new(
                Some(newStructureName.clone()),
                StructureMut::Constant,
                StructureType::Method,
                Some(lineLine),
                Some(parentLink.clone())
              );

            // Ставим модификаторы на структуру и
            // параметры структуры, если они были
            match &parameters 
            { None => {} Some(parameters) =>
            {
              for parameter in parameters
              {
                newStructure.pushStructure(
                  Structure::new(
                    parameter.getData().toString(),
                    StructureMut::Constant, // todo не знаю что сюда ставить
                    StructureType::None, // todo не знаю что сюда ставить
                    None,
                    None,
                  )
                );
              }
            }}

            // Ставим результат структуры, если он есть
            newStructure.result = match newStructureResultType
            {
              Some(newStructureResultType) =>
                Some( Token::newEmpty(newStructureResultType.clone()) ),
              None => None,
            };

            { // добавляем новую структуру в родителя
              parentLink.write().unwrap()
                .pushStructure(newStructure);
            }
            // просматриваем строки этой новой структуры;
            // todo: в целом, это можно заменить на чтение при первом обращении к структуре;
            //       сейчас же все структуры читаются (подготавливаются),
            //       если попали на lineIndex указатель.
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
  // в том случае, если это не структура и не линейная запись, 
  // мы видим TokenType::Question в начале строки и есть вложения у этой линии, 
  // то это условное вложение
  if *firstTokenType == TokenType::Question && !lineLines.is_none()
  { // условное вложение запускает код внутри себя, в том случае если её условное выражение = true;
    // если условное выражение = false, то условное вложение не запускается, 
    // но может продолжить запускать блоки ниже, если такие там есть.
    // в этом моменте мы точно уверены что нашли первое условное вложение
    let mut conditions: Vec< Arc<RwLock<Line>> > = Vec::new();
    let mut saveNewLineIndex: usize = 0;  // сдвиг вниз на сколько условных блоков мы увидели
    { // теперь мы ищем все условные вложения ниже
      let lines: Option< Vec< Arc<RwLock<Line>> > > =
      {
        parentLink.read().unwrap() // родительская структура
          .lines.clone()           // родительские линии
      };
      match lines
      { None => {} Some(lines) =>
      {
        let linesLength: usize = lines.len(); // количество линий родительской структуры
        { // смотрим линии внизу
          let mut i: usize = unsafe{*lineIndex};
          while i < linesLength
          { // если line index < lines length, то читаем вниз линии,
            // и если там первый токен не имеет TokenType::Question,
            // или количество токенов == 0, то только в этом случае break;
            // это будет означать, что мы нашли все возможные условные блоки.
            let lineBottomLink: Arc<RwLock<Line>> = lines[i].clone(); // ссылка на нижнюю линию
            { // берём нижнюю линию на чтение
              let bottomLine: RwLockReadGuard<'_, Line> = lineBottomLink.read().unwrap();
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
            // если мы не вышли, значит это условный блок;
            // значит мы его добавляем
            conditions.push(lineBottomLink);
            i += 1;
          }
        }
        // в данном месте мы точно уверенны
        // что conditions.len() > 1 из-за первого блока
        saveNewLineIndex = conditions.len()-1;
        //
      }}
      //
    }
    // после нахождения всех возможных условных блоков,
    // начинаем читать их условия и выполнять
    let mut conditionTruth: bool = false; // заранее создаём true/false ячейку
    for conditionLink in &mut conditions 
    { // итак, мы читает ссылки на условия в цикле;
      // после чего мы берём само условие на чтение
      let condition: RwLockReadGuard<'_, Line> = conditionLink.read().unwrap();
      match &condition.tokens
      { None => {} Some(tokens) =>
      {
        match tokens.len() > 1
        {
          true =>
          { // если условие больше чем просто один токен TokenType::Question,
            // то значит там обычное if/elif условие
            { // проверяем верность условия;
              let mut conditionTokens: Vec<Token> = tokens.clone(); // todo: no clone ? fix its please
              // удаляем TokenType::Question токен
              conditionTokens.remove(0);
              // и проверяем
              conditionTruth =
              { // получаем string ответ от expression, true/false
                let expressionResult: Option<String> =
                  parentLink.read().unwrap() // для этого берём родительскую линию;
                    .expression(&mut conditionTokens)
                    .getData().toString(); // и её токены.
                // итоговый boolean результат
                match expressionResult
                {
                  Some(expressionResult) => { expressionResult == "1" }
                  None => { false }
                }
              };
            }
            // если условие верно
            match conditionTruth
            { false => {} true =>
            { // создаём новую временную структуру условного блока
              let structure: Arc<RwLock<Structure>> =
                Arc::new(RwLock::new(
                  Structure::new(
                    Some(String::from("if-elif")),
                    StructureMut::Constant,
                    StructureType::Method, // todo может быть что-то другое ?
                    condition.lines.clone(),
                    Some(parentLink.clone())
                  )
                ));
              // после создания, читаем эту структуру
              let _ = drop(condition);
              readLines(structure);
              break; // end
            }}
          }
          // в случае если в токенах условия просто TokenType::Question,
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
                  Some(parentLink.clone())
                )
              ));
            // после создания, читаем эту структуру
            let _ = drop(condition);
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

lazy_static! 
{ /// Основная структура; В неё вкладываются остальные;
  /// В эту структуру будут переданы стартовые параметры;
  /// Неизменяемая; Действует во время всей жизни программы;
  pub static ref _main: Arc<RwLock<Structure>> = Arc::new(
    RwLock::new(
      Structure::new(
        Some(String::from("main")),
        StructureMut::Constant,
        StructureType::Method,
        None,
        None
      )
    )
  );
}

/// Это основная функция для парсинга строк;
/// Она разделена на подготовительную часть, и часть запуска readLine()
pub fn parseLines(tokenizerLinesLinks: Vec< Arc<RwLock<Line>> >) -> ()
{ // Начинается подготовка к запуску
  match unsafe{_debugMode} 
  { false => {} true  =>
  {
    logSeparator("Preparation");
  }}

  { // Присваиваем в главную структуру
    let mut main: RwLockWriteGuard<'_, Structure> = _main.write().unwrap();

    // Присваиваем линии от Tokenizer
    main.lines = Some(tokenizerLinesLinks);

    // argc
    main.pushStructure(
      Structure::new(
        Some(String::from("argc")),
        StructureMut::Constant, // Неизменяемая;
        StructureType::UInt,    // Не может быть меньше 0
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
        Some( _main.clone() ), // Ссылаемся на родителя
      )
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
      Structure::new(
        Some(String::from("argv")),
        StructureMut::Constant, // Неизменяемая;
        StructureType::List,    // Список;
        Some(argv),             // В линии структуры добавляем все argv линии;
        Some( _main.clone() ),  // ссылаемся на родителя
      )
    );
  }

  // Выводим arch & argv
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
  let startTime: Instant = Instant::now(); // Получаем текущее время для debug замера
  match unsafe{_debugMode} 
  { false => {} true  =>
  {
    logSeparator("Interpretation");
  }}
  // Передаём ссылку на структуру и запускаем
  readLines(_main.clone());
  // Далее идут debug замеры
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
  let (lineIndex, linesLength): (*mut usize, usize) = 
  {
    let structure: RwLockReadGuard<'_, Structure> = structureLink.read().unwrap(); // Читаем структуру
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
      lines[unsafe { *lineIndex }].clone() // Клонируем нужную линию по индексу
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
}