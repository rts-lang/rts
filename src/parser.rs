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
    lineLink.read().unwrap()
      // Токены линии на которой мы сейчас находимся
      .tokens.clone()
  };

  // Возвращаем успешно или не успешно мы нашли
  match lineTokens[0].getDataType().unwrap_or_default() == TokenType::Equals
  {
    false =>
    { // Это был не результат, идём дальше
      false
    }
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
          { // Присваиваем новую data результату;
            None => {}
            Some(structureResult) =>
            {
              structureResult.setData( newResultData.getData() );
            }
          }
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
  let line:             RwLockReadGuard<'_, Line> = lineLink.read().unwrap(); // сама линия
  let lineTokens:       &Vec<Token>               = &line.tokens;             // ссылка на токены линии
  let lineTokensLength: usize                     = lineTokens.len();         // размер токенов линии

  let firstTokenType:  TokenType                = lineTokens[0].getDataType().unwrap_or_default(); // тип первого токена в строке
  let lineLines:       Vec< Arc<RwLock<Line>> > = line.lines.clone().unwrap_or(vec![]);            // вложенные линии

  if firstTokenType == TokenType::Word
  { // если мы видим TokenType::Word в начале строки, 
    // это значит, что это либо структура, либо линейная запись
    match lineLines.len() > 0
    {
      true => 
      { // если в линии есть вложение, то это структура
        match lineTokens[0].getData() 
        { // первый токен - имя структуры
          Some(newStructureName) => 
          { // получаем имя структуры
            let mut newStructureResultType: Option<TokenType>    = None; // результат структуры
            let mut parameters:             Option< Vec<Token> > = None; // параметры структуры
            match lineTokensLength > 1 && lineTokens[1].getDataType().unwrap_or_default() == TokenType::CircleBracketBegin 
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
                   lineTokens[2].getDataType().unwrap_or_default() == TokenType::Pointer && 
                   lineTokens[3].getDataType().unwrap_or_default() != TokenType::None
                {
                  false => {} // если результата не было, то просто пропускаем
                  true => 
                  { // в таком случае просто читаем тип результата структуры
                    newStructureResultType = lineTokens[3].getDataType();
                  }
                }
              }  
              false => 
              { // в этом случае это вариант только с результатом структуры
                match lineTokensLength > 2 && 
                   lineTokens[1].getDataType().unwrap_or_default() == TokenType::Pointer && 
                   lineTokens[2].getDataType().unwrap_or_default() != TokenType::None
                {
                  false => {} // если результата не было, то просто пропускаем
                  true => 
                  { // в таком случае просто читаем тип результата структуры
                    newStructureResultType = lineTokens[2].getDataType();
                  }
                }
              }
            } // если параметров и результата не было, то просто пропускаем

            // создаём новую структуру
            let mut newStructure: Structure = 
              Structure::new(
                newStructureName.clone(),
                StructureMut::Constant,
                lineLines,
                Some(parentLink.clone())
              );

            // ставим модификаторы на структуру;
            // параметры структуры, если они были
            match &parameters 
            { 
              Some(parameters) => 
              {
                for parameter in parameters 
                {
                  newStructure.pushStructure(
                    Structure::new(
                      parameter.getData().unwrap_or_default(),
                      StructureMut::Constant,
                      vec![], // todo: add option, pls 
                      None,
                    )
                  );
                }
              }
              None => {}
            }

            // Ставим результат структуры, если он есть
            newStructure.result = match newStructureResultType
            {
              Some(_) => Some( Token::newEmpty(newStructureResultType.clone()) ),
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
          None => {}
        }
      }  
      false =>
      { // если это не структура, значит это линейная запись
        let mut opType: TokenType = TokenType::None; // готовим место для проверки оператора
        let mut opPos:  usize     = 0;               // это будет место, где находится оператор
        for (i, lineToken) in lineTokens.iter().enumerate()
        { // читаем линию, и ищем чтобы TokenType в opType совпал с математическим
          // после чего выходим отсюда и остаётся позиция найденного оператора в opPos
          opType = lineToken.getDataType().unwrap_or_default().clone();
          match isMathOperator(opType.clone()) 
          {
            false => {}
            true => 
            {
              opPos = i+1;
              break;
            }
          }
        }
        
        match lineTokensLength > 1 && opPos > 1
        { // позиция оператора не может быть 0, т.к. по 0 у нас TokenType::Word
          // поэтому мы проверяем позицию > 1 и количество токенов в строке > 1
          false => {}
          true =>
          { // теперь мы точно уверенны, что это линейная запись с математической операцией

            let leftValueTokens:Vec<Token> = lineTokens[0..opPos-1].to_vec();
            let mut leftValueMutable: StructureMut = StructureMut::Constant;
            let mut leftValueDataType: Option<TokenType> = None; // todo не определяет final type
            { // Определяем тип данных и тип модификатора у левой части выражения
              let leftValueTokensLength:usize = leftValueTokens.len();
              let mut dataTypeBeginPos:usize = 1;
              for i in 0..leftValueTokensLength
              {
                match i == 1
                { // Определяем тип модификатора у левой части выражения
                  false => {}
                  true =>
                  {
                    match leftValueTokens[1].getDataType()
                    {
                      None => {}
                      Some(mutableType) =>
                      {
                        dataTypeBeginPos += 1;
                        match mutableType
                        {
                          TokenType::DoubleTilde =>
                          {
                            leftValueMutable = StructureMut::Dynamic;
                          }
                          TokenType::Tilde =>
                          {
                            leftValueMutable = StructureMut::Variable;
                          }
                          _ =>
                          {
                            leftValueMutable = StructureMut::Constant;
                          }
                        }
                      }
                    }
                  }
                }
                match i == dataTypeBeginPos
                { // Определяем тип данных у левой части выражения
                  false => {}
                  true =>
                  { // Если есть 3 токен
                    match leftValueTokens[dataTypeBeginPos].getDataType().unwrap_or_default() == TokenType::Colon
                    {
                      false => {}
                      true =>
                      { // Если это : то следом должен быть тип данных
                        match leftValueTokensLength > dataTypeBeginPos
                        {
                          false => {}
                          true =>
                          { // Токен с типом данных существует
                            let dataType:Option<TokenType> = leftValueTokens[dataTypeBeginPos+1].getDataType();
                            match matches!(dataType.clone().unwrap_or_default(),
                              TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float |
                              TokenType::String | TokenType::Char | TokenType::Rational | TokenType::Complex)
                            {
                              false => {}
                              true =>
                              { // todo: Нужен механизм для проверки на все существующие типы данных примитивов и custom
                                leftValueDataType = dataType;
                              }
                              //
                            }
                          }
                          //
                        }
                      }
                      //
                    }
                  }
                  //
                }
              }
              //
            }
            println!("leftValueMutable {}:{}",leftValueMutable.to_string(), leftValueDataType.unwrap_or_default().to_string());

            match lineTokens[0].getData() 
            { // получаем имя первого токена, чтобы знать с кем мы работаем
              None => {}
              Some(structureName) =>
              { // это левая часть линейной записи
                // todo: возможно сократить? это просто один токен из structureName
                let leftValue:  Option< Vec<Token> > = Some( leftValueTokens );
                // это правая (рабочая) запись линейной части
                let rightValue: Option< Vec<Token> > = Some( lineTokens[opPos..(lineTokensLength)].to_vec() );

                // получаем родительскую структуру
                
                // ищем в родительской структуре, есть ли там похожая на structureName
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
                  { // если мы нашли такую, то значит работаем уже с существующей структурой
                    parentLink.clone()
                      .read().unwrap()
                      .structureOp(
                        structureLink, 
                        opType, 
                        leftValue.unwrap_or(vec![]),
                        rightValue.unwrap_or(vec![])
                    );
                  }
                  None =>
                  { // Если мы не нашли похожую, то создаём новую
                    // и работаем с правой частью выражения
                    let mut tokens: Vec<Token> =
                      match rightValue
                      {
                        None =>
                        { // Если правого выражения не было, то это Final
                          leftValueMutable = StructureMut::Final;
                          vec![]
                        }
                        Some(rightValue) =>
                        { // Если правое выражение существует
                          rightValue // Constant | Variable | Dynamic
                        }
                      };

                    let calculateRightValueNow: bool = leftValueMutable == StructureMut::Constant;

                    // закидываем новую структуру в родительскую структуру
                    let mut parentStructure: RwLockWriteGuard<'_, Structure> = parentLink.write().unwrap();

                    // Вычисляем правое выражение?
                    match calculateRightValueNow
                    {
                      false => {} // Мы ничего не вычисляем сейчас для Variable | Dynamic
                      true =>
                      {
                        tokens = vec![
                          parentStructure.expression(&mut tokens)
                        ];
                      }
                    }

                    // Создаём структуру
                    parentStructure.pushStructure(
                      Structure::new(
                        structureName,
                        leftValueMutable,
                        vec![ Arc::new(RwLock::new(
                          Line {
                            tokens: tokens,
                            indent: 0,
                            lines:  None,
                            parent: None
                          }
                        )) ],
                        None
                      )
                    );
                  }
                } //
                return true;
              }
            } //
          }
        }
        //
      }
    }
  } else 
  // в том случае, если это не структура и не линейная запись, 
  // мы видим TokenType::Question в начале строки и есть вложения у этой линии, 
  // то это условное вложение
  if firstTokenType == TokenType::Question && lineLines.len() > 0
  { // условное вложение запускает код внутри себя, в том случае если её условное выражение = true;
    // если условное выражение = false, то условное вложение не запускается, 
    // но может продолжить запускать блоки ниже, если такие там есть.
    // в этом моменте мы точно уверены что нашли первое условное вложение
    let mut conditions: Vec< Arc<RwLock<Line>> > = Vec::new();
    let mut saveNewLineIndex: usize = 0;  // сдвиг вниз на сколько условных блоков мы увидели
    { // теперь мы ищем все условные вложения ниже
      let lines: Vec< Arc<RwLock<Line>> > = 
      { 
        parentLink.read().unwrap() // родительская структура
          .lines.clone()           // родительские линии
      };
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
            match bottomLine.tokens.len() == 0 
            { // Выходим если линия пустая 
              true  => { break; }
              false => 
              {
                match bottomLine.tokens[0].getDataType().unwrap_or_default() != TokenType::Question 
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
    }
    // после нахождения всех возможных условных блоков,
    // начинаем читать их условия и выполнять
    let mut conditionTruth: bool = false; // заранее создаём true/false ячейку
    for conditionLink in &mut conditions 
    { // итак, мы читает ссылки на условия в цикле;
      // после чего мы берём само условие на чтение
      let condition: RwLockReadGuard<'_, Line> = conditionLink.read().unwrap();
      match condition.tokens.len() > 1 
      {
        true => 
        { // если условие больше чем просто один токен TokenType::Question,
          // то значит там обычное if/elif условие
          { // проверяем верность условия;
            let mut conditionTokens: Vec<Token> = condition.tokens.clone(); // todo: no clone ? fix its please
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
                Some(expressionResult) =>
                {
                  expressionResult == "1"
                }
                None =>
                {
                  false
                }
              }
            };
          }
          // если условие верно
          match conditionTruth 
          {
            true => 
            { // создаём новую временную структуру условного блока
              let structure: Arc<RwLock<Structure>> =
                Arc::new(
                RwLock::new(
                  Structure::new(
                    String::from("if-elif"),
                    StructureMut::Constant,
                    condition.lines.clone().unwrap_or(vec![]),
                    Some(parentLink.clone())
                  )
                ));
              // после создания, читаем эту структуру
              let _ = drop(condition);
              readLines(structure);
              break; // end
            }
            false => {}
          }
        }
        // в случае если в токенах условия просто TokenType::Question,
        // значит это else блок
        false => if !conditionTruth
        { // создаём новую временную структуру условного блока
          let structure: Arc<RwLock<Structure>> =
            Arc::new(
            RwLock::new(
              Structure::new(
                String::from("else"),
                StructureMut::Constant,
                condition.lines.clone().unwrap_or(vec![]),
                Some(parentLink.clone())
              )
            ));
          // после создания, читаем эту структуру
          let _ = drop(condition);
          readLines(structure);
          break; // end
        }
      }
    }

    // и только после прочтения всех блоков, 
    // мы можем сдвигать указатель ниже
    unsafe{*lineIndex += saveNewLineIndex}
    return true;
  }
  return false;
}

lazy_static! 
{ /// Основная структура; В неё вкладываются остальные;
  /// В эту структуру будут переданы стартовые параметры;
  /// Неизменяемая; Действует во время всей жизни программы;
  static ref _main: Arc<RwLock<Structure>> = Arc::new(
    RwLock::new(
      Structure::new(
        String::from("main"),
        StructureMut::Constant,
        Vec::new(),
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
  {
    true  => { logSeparator("Preparation"); }
    false => {}
  }

  { // Присваиваем в главную структуру
    let mut main: RwLockWriteGuard<'_, Structure> = _main.write().unwrap();

    // Присваиваем линии от Tokenizer
    main.lines = tokenizerLinesLinks;

    // argc
    main.pushStructure(
      Structure::new(
        String::from("argc"),
        StructureMut::Constant, // Неизменяемая;
        vec![                    // В линии структуры
          Arc::new(RwLock::new(       // добавляем линию с 1 токеном
            Line
            {
              tokens: vec![
                Token::new( 
                  Some(TokenType::UInt), 
                  Some(unsafe{_argc.to_string()}) 
                )
              ],
              indent: 0,
              lines:  None,
              parent: None
            }
          ))
        ],
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
            tokens: vec![
              Token::new( 
                Some(TokenType::String), 
                Some(String::from(a)) 
              )
            ],
            indent: 0,
            lines:  None,
            parent: None
          }
        ))
      );
    }
    main.pushStructure(
      Structure::new(
        String::from("argv"),
        StructureMut::Constant, // Неизменяемая;
        argv,                         // В линии структуры добавляем все argv линии;
        Some( _main.clone() ),        // ссылаемся на родителя
      )
    );
  }

  // Выводим arch & argv
  unsafe
  {
    match _debugMode
    {
      false => {}
      true =>
      {
        log("ok", &format!("argc [{}]", _argc));
        match _argc > 0
        {
          false => {}
          true =>
          {
            log("ok", &format!("argv {:?}", _argv));
          }
        }
      }
    }
  }

  // Подготовка закончена, читаем линии
  let startTime: Instant = Instant::now(); // Получаем текущее время для debug замера
  match unsafe{_debugMode} 
  {
    true  => { logSeparator("Interpretation"); }
    false => {}
  }
  // Передаём ссылку на структуру и запускаем
  readLines(_main.clone());
  // Далее идут debug замеры
  match unsafe{_debugMode} 
  {
    true => 
    {
      let endTime:  Instant  = Instant::now();    // Получаем текущее время
      let duration: Duration = endTime-startTime; // Получаем сколько всего прошло
      logSeparator("End");
      log("ok",&format!("Parser duration [{:?}]",duration));
    }
    false => {}
  }
}
/// Эта функция занимается чтением блоков по ссылке на них
/// todo: исправить переполнение стека
pub fn readLines(structureLink: Arc<RwLock<Structure>>) -> ()
{ // Получаем сколько линий вложено в структуру
  let (lineIndex, linesLength): (*mut usize, usize) = 
  {
    let structure: RwLockReadGuard<'_, Structure> = structureLink.read().unwrap(); // Читаем структуру
    (
      { &structure.lineIndex as *const usize as *mut usize },
      structure.lines.len()
    )
  };

  // Выполнение программы происходит до тех пор,
  // пока не будет всё прочитано, либо 
  // пока не будет вызван _exitCode на true
  let mut lineLink: Arc<RwLock<Line>>;
  while unsafe{_exit == false} && unsafe{*lineIndex < linesLength}
  { // Если мы читаем строки, то создаём сразу ссылку на текущую линию
    lineLink = 
    { // Получаем её через чтение текущей структуры;
      // Берём линию по индексу линии
      structureLink.read().unwrap()
        .lines[unsafe{*lineIndex}].clone()
    };
    // После чего проверяем, если линия пустая на токены, то не читаем и идём дальше
    match
      lineLink.read().unwrap()
        .tokens.len() == 0
    {
      false => {}
      true => 
      {
        unsafe{*lineIndex += 1}
        continue;
      }
    }
    // Если всё хорошо, то начинаем читать через специальные функции;
    // Ищем структуры
    match !searchStructure(lineLink.clone(), structureLink.clone(), lineIndex)
    {
      false => {}
      true => 
      { // Читаем return
        match !searchReturn(lineLink.clone(), structureLink.clone())
        {
          false => {}
          true =>
          { // Ищем линейные выражения
            structureLink.read().unwrap()
              .expression(
                &mut lineLink.read().unwrap()
                  .tokens.clone()
              );
            // Клонируем токены, для сохранения возможности повторного запуска
          }
        }
      }
    }
    // Идём дальше
    unsafe{*lineIndex += 1}
  }
  // Сбрасываем указатель линий для текущей структуры на 0
  // Для того чтобы можно было запускать повторно
  unsafe{*lineIndex = 0}
}