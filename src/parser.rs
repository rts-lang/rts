/* /parser
  предоставляет механизмы для парсинга токенов,
  что позволяет запускать получившиеся структуры.
*/

pub mod value;
pub mod uf64;
pub mod structure;
mod procedureCall;
mod functionCall;

use crate::{
  logger::*,
  _argc, _argv, _debugMode, _exit,
  parser::structure::*,
  tokenizer::{token::*, line::*}
};

use std::{
  time::{Instant, Duration},
  sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

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
fn searchReturn(lineLink: Arc<RwLock<Line>>, structureLink: Arc<RwLock<Structure>>) -> bool 
{
  let mut lineTokens: Vec<Token> = 
  { // Читаемая линия, на которой мы сейчас находимся
    match lineLink.read().unwrap().tokens.clone()
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
/// Эта функция ищет структуры;
///
/// Это может быть:
/// - Вложенная структура (array/vector/list ...)
/// - Линейное выражение (a = 10)
/// - Условный блок (if/elif/else)
fn searchStructure(lineLink: Arc<RwLock<Line>>, parentLink: Arc<RwLock<Structure>>, lineIndex: *mut usize) -> bool
{
  // todo: line можно вынести, чтобы потом не было .read().unwrap();
  //       для этого надо сразу забрать все нужные значения здесь.
  let line: RwLockReadGuard<'_, Line> = lineLink.read().unwrap(); // Текущая линия
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
        match lineTokens[0].getData() 
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
                    parameter.getData(),
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
              None    => None,
            };

            { // добавляем новую структуру в родителя
              parentLink.write().unwrap()
                .pushStructure(newStructure);
            }
            // просматриваем строки этой новой структуры;
            // todo: в целом, это можно заменить на чтение при первом обращении к структуре;
            //       сейчас же все структуры читаются (подготавливаются),
            //       если попали на lineIndex указатель.
            readLines(
              parentLink.read().unwrap()
                .getStructureByName(&newStructureName).unwrap(), // todo: плохой вариант, можно лучше
            );
            return true;
          }
        }
      }
      None =>
      { // Это линейная запись
        let mut opType: TokenType = TokenType::None; // Математический оператор
        let mut opPos:  usize     = 0;               // Место, где находится оператор
        for (i, lineToken) in lineTokens.iter().enumerate()
        { // читаем линию, и ищем чтобы TokenType в opType совпал с математическим
          // после чего выходим отсюда и остаётся позиция найденного оператора в opPos
          opType = lineToken.getDataType().clone();
          match isMathOperator(opType.clone()) 
          { false => {} true =>
          {
            opPos = i+1;
            break;
          }}
        }

        // Получаем mutable и dataType для структуры
        let leftValueTokens: Vec<Token> =
          match opPos == 0
          {
            true =>
            { // Это вариант Final, но необходимо проверить, что это не что-то другое
              let lineTokensLen: usize = lineTokens.len();
              match lineTokensLen == 3
              {
                false =>
                {
                  match lineTokensLen == 1
                  {
                    false => return false, // Это что-то другое
                    true => {} // Это Final без указания типа данных
                  }
                }
                true =>
                { // Предполагается Final с указанием типа данных
                  match *lineTokens[1].getDataType() == TokenType::Colon
                  {
                    false => return false, // Это что-то другое
                    true => {}
                  }
                  // todo наверное здесь нужно ещё проверять 2 токен, что там за тип ?
                }
              }
              lineTokens.to_vec()
            },
            false => lineTokens[0..opPos-1].to_vec() // Это Constant | Variable | Dynamic
          };
        let mut leftValueMutable: StructureMut = StructureMut::Constant;
        let mut leftValueDataType: StructureType = StructureType::None;
        { // Определяем тип данных и тип модификатора у левой части выражения
          let leftValueTokensLength:usize = leftValueTokens.len();
          for i in 0..leftValueTokensLength
          {
            match i == 1
            { false => {} true =>
            { // Определяем тип модификатора у левой части выражения
              leftValueMutable =
                match leftValueTokens[1].getDataType()
                {
                  TokenType::DoubleTilde => StructureMut::Dynamic,
                  TokenType::Tilde => StructureMut::Variable,
                  _ => StructureMut::Constant
                };
            }}
            // Определяем тип данных у левой части выражения
            match *leftValueTokens[i].getDataType() == TokenType::Colon
            { false => {} true =>
            { // Если это : то следом должен быть тип данных
              match leftValueTokensLength > i
              { false => {} true =>
              { // Токен с типом данных существует
                let dataType:&TokenType = leftValueTokens[i+1].getDataType();
                leftValueDataType = dataType.toStructureType();
              }}
            }}
            //
          }
          //
        }

        // Получаем имя первого токена, чтобы знать с кем мы работаем
        match lineTokens[0].getData()
        { None => {} Some(structureName) =>
        { // Это левая часть линейной записи
          // todo: возможно сократить? это просто один токен из structureName
          let leftPart:  Option< Vec<Token> > = Some( leftValueTokens );
          // Это правая (рабочая) запись линейной части
          let rightPart: Option< Vec<Token> > =
            match opPos == 0
            {
              true => None, // У Final нет правой части выражения
              false => // Это Constant | Variable | Dynamic
                Some( lineTokens[opPos..lineTokensLength].to_vec() )
            };

          // Получаем родительскую структуру;
          // Ищем в родительской структуре, есть ли там похожая на structureName
          let structureLink: Option< Arc<RwLock<Structure>> > =
          {
            parentLink.read().unwrap()
              .getStructureByName(&structureName)
          };
          match structureLink
          {
            // todo проверка на opType, потому что может быть неправильное выражение
            //      и его даже не следует обрабатывать
            Some(structureLink) =>
            { // Если мы нашли такую, то значит работаем уже с существующей структурой
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
                  match (leftPart, rightPart)
                  {
                    (Some(leftPart), Some(rightPart)) =>
                    {
                      parent.structureOp(
                        structureLink,
                        opType,
                        structureMut,
                        leftPart,
                        rightPart
                      );
                    }
                    _ => {}
                  }
                  //
                }
              }
              //
            }
            None =>
            { // Если мы не нашли похожую, то создаём новую
              // и работаем с правой частью выражения
              let mut tokens: Option< Vec<Token> > =
                match rightPart
                {
                  None =>
                  { // Если правого выражения не было, то это Final
                    leftValueMutable = StructureMut::Final;
                    None
                  }
                  Some(rightValue) =>
                  { // Если правое выражение существует
                    Some(rightValue) // Constant | Variable | Dynamic
                  }
                };

              let calculateRightValueNow: bool = leftValueMutable != StructureMut::Final;

              // Закидываем новую структуру в родительскую структуру
              let mut parentStructure: RwLockWriteGuard<'_, Structure> = parentLink.write().unwrap();

              // Вычисляем правое выражение сразу?
              match calculateRightValueNow
              { false => {} true =>
              { // Только для константной структуры значение определяется сразу
                let hasTokens: bool = tokens.is_none();
                let mut value: Token = parentStructure.expression(&mut tokens.unwrap());
                match leftValueDataType == StructureType::None
                {
                  true =>
                  { // Тип вычисляется только если он не был изначально определён;
                    // Вычисляется он по типу из результата правой части выражения
                    leftValueDataType = value.getDataType().toStructureType();
                  }
                  false =>
                  { // Требуется выполнить преобразование в указанный тип данных
                    value.setDataType(leftValueDataType.toTokenType());
                  }
                }
                //
                tokens =
                  match hasTokens
                  { true => None, false =>
                  {
                    Some(vec![ value ])
                  }}
              }}

              // Создаём структуру
              parentStructure.pushStructure(
                Structure::new(
                  Some(structureName),
                  leftValueMutable,
                  leftValueDataType,
                  Some(vec![
                    Arc::new(RwLock::new(
                    Line {
                      tokens: tokens,
                      indent: None,
                      lines:  None,
                      parent: None
                    }
                    ))
                  ]),
                  None
                )
              );
            }
          } //
          return true;
        }}
        //
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
                    .expression(&mut conditionTokens).getData(); // и её токены.
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
  static ref _main: Arc<RwLock<Structure>> = Arc::new(
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
                  Some(unsafe{_argc.to_string()}) 
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
                Some(String::from(a)) 
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
  let mut lineLink: Arc<RwLock<Line>>;
  while unsafe{_exit == false} && unsafe{*lineIndex < linesLength}
  { // Если мы читаем строки, то создаём сразу ссылку на текущую линию
    lineLink = 
    { // Получаем её через чтение текущей структуры;
      // Берём линию по индексу линии (она точно будет, поскольку выше мы это проверили)
      let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
      let lines: &Vec< Arc<RwLock<Line>> > = structure.lines.as_ref().unwrap();
      lines[unsafe { *lineIndex }].clone() // Клонируем нужную линию по индексу
    };
    // После чего проверяем, если линия пустая на токены, то не читаем и идём дальше
    match
      lineLink.read().unwrap()
        .tokens.is_none()
    { false => {} true =>
    {
      unsafe{*lineIndex += 1}
      continue;
    }}
    // Если всё хорошо, то начинаем читать через специальные функции;
    // Ищем структуры
    match !searchStructure(lineLink.clone(), structureLink.clone(), lineIndex)
    { false => {} true =>
    { // Читаем return

      match !searchReturn(lineLink.clone(), structureLink.clone())
      { false => {} true =>
      { // Ищем линейные выражения

        let tokens: &mut Vec<Token> =
          &mut lineLink.read().unwrap()
            .tokens.clone()
            .unwrap_or_default(); // todo плохо
        structureLink.read().unwrap()
          .expression(tokens);
        // Клонируем токены, для сохранения возможности повторного запуска
      }}
    }}
    // Идём дальше
    unsafe{*lineIndex += 1}
  }
  // Сбрасываем указатель линий для текущей структуры на 0
  // Для того чтобы можно было запускать повторно
  unsafe{*lineIndex = 0}
}