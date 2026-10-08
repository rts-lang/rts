use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::_sourcePath;
use crate::parser::bytes::Bytes;
use crate::parser::structure::ffi::bridge::{callExternal, callExternalWithScope, FfiExpect};
use crate::parser::structure::ffi::scopeStack;
use crate::parser::structure::methods::parameters::{Parameters};
use crate::parser::structure::structureType::{StructureType};
use crate::parser::structure::tokenValue::calculate::{calculate, normalizeToken};
use crate::tokenizer::tokenizer::readTokensSimple;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::{Token};
use crate::tokenizer::types::tokenType::{TokenType};
// =================================================================================================

/* 
  Структура, которая представляет свободную ячейку данных в памяти;
  имеет свои настройки, место хранения.
*/

// =================================================================================================

/// Обозначает уровень изменения структуры.
#[derive(Clone, PartialEq, Eq)]
pub enum StructureMut
{
  /// Ожидает первое значение и превратится в Constant.
  Final,
  
  /// Не может быть изменена, присваивается в момент создания.
  Constant,
  
  /// Может изменять только значение.
  Variable,
  
  /// Может изменять и значение и тип данных.
  Dynamic
}
impl ToString for StructureMut
{ 
  // todo convert -> fmt::Display ?
  fn to_string(&self) -> String
  {
    match self
    {
      Self::Final => String::from("Final"),
      Self::Constant => String::from("Constant"),
      Self::Variable => String::from("Variable"),
      Self::Dynamic => String::from("Dynamic")
    }
  }
}

// =================================================================================================

/// Свободная структура данных
#[derive(Clone)]
pub struct Structure 
{
  /// Уникальное имя;
  /// 
  /// Если не будет указано, значит это временная структура.
  pub name: Option<String>,

  /// Уровень изменения.
  pub mutable: StructureMut,

  /// Тип данных.
  pub dataType: StructureType,

  /// Набор вариантов, если структура объявлена как объединение типов
  /// `a: U8 | String = ...` (issue #59).
  ///
  /// Само значение лежит в `dataType` — это ровно ОДИН из вариантов,
  /// а `unionTypes` продолжает ограничивать все следующие присваивания.
  /// `None` — обычная структура без объединения.
  pub unionTypes: Option<Vec<StructureType>>,

  /// Ссылки на вложенные линии.
  pub lines: Option< Vec< Arc<RwLock<Line>> > >,

  /// Входные параметры.
  /// 
  /// todo Не используется в коде. Зачем оно тогда здесь было?
  pub parameters: Parameters,

  /// Выходной результат.
  /// 
  /// None => procedure  
  /// else => function
  pub result: Option<Token>,

  /// Ссылки на вложенные структуры.
  pub structures: Arc<RwLock< // Нужно, чтобы не мутировать методы и иметь доступ.
    Option< // Может не быть.
      Vec< Arc<RwLock<Self>> > // Гибкий список вложенных структур.
    >
  >>,

  /// Ссылка на родителя.
  pub parent: Option< Arc<RwLock<Self>> >,

  /// Создана ли структура из блока FFI.
  pub isFfiBlock: bool,

  /// Файл, в котором написан код структуры (`None` — запущенный файл);
  /// 
  /// Запоминается при создании из `_sourcePath`.
  pub sourcePath: Option< Arc<String> >,

  /// todo Комментарий + возможно не нужно т.к. можно лучше
  pub lineIndex: usize
}

impl Structure 
{
  pub fn new
  (
    name: Option<String>,
    mutable: StructureMut,
    dataType: StructureType,
    lines: Option< Vec< Arc<RwLock<Line>> > >,
    parent: Option< Arc<RwLock<Self>> >
  ) -> Self 
  {
    Self 
    {
      name,
      mutable,
      dataType,
      unionTypes: None,
      lines,
      parameters: Parameters::new(None),
      result: None,
      structures: Arc::new(RwLock::new(None)),
      parent,
      isFfiBlock: false,
      sourcePath: _sourcePath.read().unwrap().clone(),
      lineIndex: 0
    }
  }

  // ===============================================================================================
  
  /// todo desc
  pub fn parseLink(linkName: &str) -> Vec<String> 
  {
    linkName
      .split('.')
      .map(|segment: &str| segment.to_string())
      .collect()
  }

  /// Ищет структуру по имени (даже если это ссылка)
  ///
  /// Пример: "parent.child.grandchild" будет искать:
  ///   1. "parent" в корневых структурах
  ///   2. "child" в дочерних структурах "parent"
  ///   3. "grandchild" в дочерних структурах "child"
  /// 
  /// todo Не смотрит выше self. Должен ли?
  pub fn getStructureByName(&self, name: &str) -> Option< Arc<RwLock<Self>> > 
  {
    // "a.b.c" -> ["a", "b", "c"]
    let segments: Vec<String> = Self::parseLink(name);
    
    // Если имя пустое - нечего искать.
    if segments.is_empty() {
      return None
    }

    // Начинаем с корневого уровня (None).
    let mut currentStructure: Option< Arc<RwLock<Self>> > = None;

    // Пошагово проходим по каждому сегменту имени.
    for segment in segments.iter() 
    {
      // Определяем список структур для поиска на текущем уровне:
      // если currentStructure = None, это означает корневой уровень self.structures;
      // иначе — получаем дочерние структуры текущей найденной структуры.
      let childrenLink: Arc<RwLock< Option<Vec< Arc<RwLock<Self>> >> >> = match &currentStructure 
      {
        None => self.structures.clone(), // Корневые структуры.
        Some(structureRef) => {
          let structureGuard: RwLockReadGuard<Self> = structureRef.read().unwrap();
          structureGuard.structures.clone() // Дочерние структуры текущей.
        }
      };
      let childrenOption: RwLockReadGuard< Option<Vec< Arc<RwLock<Self>> >> > = childrenLink.read().unwrap();

      // Флаг найденной структуры.
      let mut found: bool = false;
      // Следующая структура, если сегмент найден.
      let mut nextStructure: Option< Arc<RwLock<Self>> > = None;

      // Обрабатываем наличие дочерних структур.
      if let Some(children) = childrenOption.as_deref()
      {
        for child in children 
        {
          let childGuard: RwLockReadGuard<Self> = child.read().unwrap();
          if let Some(childName) = &childGuard.name 
          {
            if childName == segment
            {
              found = true;
              nextStructure = Some(child.clone());
              break;
            }
          }
          //
        }
      }
      drop(childrenOption);

      // Если не найдено соответствие текущему сегменту - путь невалиден.
      if !found {
        return None
      }

      // Переходим на следующий уровень структуры.
      currentStructure = nextStructure;
    }

    // После успешного прохождения всех сегментов возвращаем найденную структуру.
    currentStructure
  }

  /// Добавляет новую вложенную структуру в текущую структуру;
  /// 
  /// Нет &mut self - что хорошо.
  pub fn pushStructure(&self, structureLink: Arc<RwLock<Self>>) -> ()
  {
    let mut children: RwLockWriteGuard<Option< Vec< Arc<RwLock<Self>> > >> =
      self.structures.write().unwrap();
    
    if let Some(childrenVec) = children.as_mut() 
    { // Если уже есть структуры, то просто push делаем.
      childrenVec.push(structureLink);
    } else 
    { // Если не было ещё структур, то создаём новый вектор.
      *children = Some(vec![structureLink]);
    }
  }

  // ===============================================================================================

  /// Выполняет операцию со структурой.
  /// 
  /// Требует левую и правую часть выражения.
  /// 
  /// Требует передачи родительской структуры,
  /// чтобы было видно возможные объявления в ней.
  /// 
  /// Это работает только для существующих структур.
  /// 
  /// Например обычная `a = 10` первый раз - это линейная запись, а не Op.
  /// 
  /// op это когда `a += 10` например или `a = 20; a = 10`, когда была структура.
  pub fn structureOp(
    &self, 
    structureLink: Arc<RwLock<Self>>, 
    op: TokenType, 
    leftPartMutable: StructureMut, 
    rightPart: Vec<Token>
  ) -> ()
  {
    match op
    { // Принимаем только математические операции.
      TokenType::Equals => {} /* |
      TokenType::PlusEquals |
      TokenType::MinusEquals |
      TokenType::MultiplyEquals |
      TokenType::DivideEquals => {},
      */
      _ => return
    }

    if op == TokenType::Equals
    {
      // Приравнивание правой части выражения к левой части выражения.

      // todo Должен быть вариант с вложением?
      // Если нет вложений

      // Что ждём от FFI-вызова справа: у структуры уже есть тип — результат кастуется к нему;
      // если типа нет (или Dynamic может его менять) — левая часть получит тип от правой.
      let expect: FfiExpect =
      {
        let structure: RwLockReadGuard<Self> = structureLink.read().unwrap();
        if structure.dataType == StructureType::None ||
           structure.unionTypes.is_some() || // У объединения нет одного ABI-типа (#59).
           leftPartMutable == StructureMut::Dynamic
        {
          FfiExpect::Infer
        } else {
          FfiExpect::Typed(structure.dataType.clone())
        }
      };
      let mut rightPartValue: Token = self.expressionWith(&mut rightPart.clone(), &expect);

      // Динамический import(): правая часть — не скаляр,
      // а целая под-структура (модуль со своими подструктурами).
      // 
      // Обычный скалярный путь ниже для этого не подходит — переносим
      // (dataType, lines, structures) временной структуры-модуля
      // напрямую в левую часть присваивания.
      if *rightPartValue.getDataType() == TokenType::Link
      {
        if let Some(markerName) = rightPartValue.getData().toString()
        {
          if let Some(moduleLink) = self.getStructureByName(&markerName)
          {
            if moduleLink.read().unwrap().dataType == StructureType::Custom(String::from("Module"))
            { // Если это модуль.
              
              let moduleGuard: RwLockReadGuard<Self> = moduleLink.read().unwrap();
              let mut structure: RwLockWriteGuard<Self> = structureLink.write().unwrap();

              structure.dataType = moduleGuard.dataType.clone();
              structure.lines = moduleGuard.lines.clone();
              *structure.structures.write().unwrap() = moduleGuard.structures.read().unwrap().clone();

              if leftPartMutable == StructureMut::Final {
                structure.mutable = StructureMut::Constant;
              }
              
              drop(moduleGuard);
              drop(structure);
              return;
              //
            }
          }
          //
        }
      }

      let mut structure: RwLockWriteGuard<Self> = structureLink.write().unwrap();

      // Объединение типов (issue #59): если структура объявлена как `U8 | String`,
      // то каждое присваивание подбирает свой вариант, а не приводится в один
      // зафиксированный тип.
      match structure.unionTypes.clone()
      {
        Some(variants) => 
          if leftPartMutable == StructureMut::Dynamic
          { // `~~` — тип и так меняется, объединение не ограничивает (#22)
            structure.dataType = rightPartValue.getStructureType();
          } else
          {
            structure.dataType = 
              Self::normalizeUnion(&mut rightPartValue, &StructureType::Union(variants));
          }
        None => 
          if structure.dataType == StructureType::None ||
             leftPartMutable == StructureMut::Dynamic // Dynamic может изменить dataType просто так.
          {
            if leftPartMutable != StructureMut::Variable
            { // Будет присвоено только Final | Dynamic.
              structure.dataType = rightPartValue.getStructureType();
            }
          } else
          { // Требуется выполнить преобразование в указанный тип данных.
            // Primitive → natural ABI-форма (normalizeToken не знает примитивов).
            Self::normalizeToken(&mut rightPartValue, structure.dataType.abiType())
          }
      }

      if leftPartMutable == StructureMut::Final
      { // Изменяем mutable если это был Final.
        structure.mutable = StructureMut::Constant;
      }

      // Приравниваем новое значение структуре.
      structure.lines =
        Some(vec![
          Arc::new(RwLock::new(Line
          {
            tokens: Some(vec![ rightPartValue ]),
            lines: None
          }))
        ]);
    } else
    { // Иные операторы, например += -= *= /=
      // получаем левую и правую часть.
      // todo сейчас тут много ошибок.
      let _leftValue: Token = 
      {
        let structure: RwLockReadGuard<Self> = structureLink.read().unwrap();
        if let Some(lines) = &structure.lines
        {
          if !lines.is_empty()
          {
            self.expression(
              &mut lines[0].read().unwrap()
                .tokens.clone()
                .unwrap_or_default() // todo плохо
            )
          } else {
            Token::newEmpty(TokenType::None)
          }
        } else {
          Token::newEmpty(TokenType::None)
        }
        //
      };
      let _rightPart: Token = self.expression(&mut rightPart.clone()); // todo: возможно не надо клонировать токены, но скорее надо.
      
      /* todo Может плохо работать с #85, нужен контроль.
      // Далее обрабатываем саму операцию.
      let mut structure: RwLockWriteGuard<Structure> = structureLink.write().unwrap();
      match op 
      { // Определяем тип операции
        TokenType::PlusEquals => 
        { 
          structure.lines = 
            Some(vec![
              Arc::new(RwLock::new( 
                Line {
                  tokens: Some(vec![ calculate(&TokenType::Plus, &leftValue, &rightPart) ]),
                  // todo Здесь должны быть преобразования типа у структуры.
                  //  Сейчас если станет Int, то у структуры не поменяется U8 на I8.
                  //  + Здесь должна быть normalizeToken когда не Dynamic.
                  indent: None,
                  lines:  None,
                  parent: None
                }
              ))
            ]);
        }
        _ => {} // todo: Дописать другие варианты; а также добавит для них отдельные тесты.
      }
      //if op == TokenType::PlusEquals     { structure.value = calculate(&TokenType::Plus,     &leftValue, &rightValue); } else 
      //if op == TokenType::MinusEquals    { structure.value = calculate(&TokenType::Minus,    &leftValue, &rightValue); } else 
      //if op == TokenType::MultiplyEquals { structure.value = calculate(&TokenType::Multiply, &leftValue, &rightValue); } else 
      //if op == TokenType::DivideEquals   { structure.value = calculate(&TokenType::Divide,   &leftValue, &rightValue); }
      */
    }
  }

  // ===============================================================================================

  /// Вычисляем значение для struct имени типа `TokenType::Word`.
  fn replaceStructureByName(&self, value: &mut [Token], index: usize) -> ()
  {
    fn setNone(value: &mut [Token], index: usize) 
    { // Возвращаем пустое значение.
      value[index].setData(None);
      value[index].setDataType(TokenType::None);
    }

    if let Some(structureName) = value[index].getData().toString() 
    {
      if let Some(structureLink) = self.getStructureByName(&structureName) 
      {
        let structure: RwLockReadGuard<Self> = structureLink.read().unwrap();
        // Если это просто обращение к имени структуры.
        if let Some(lines) = &structure.lines
        {
          let structureLinesLen: usize = lines.len();
          match structureLinesLen
          {
            1 =>
            { // Структура с одним вложением.
              let tokens: &mut Vec<Token> =
                &mut lines[0]
                  .read().unwrap()
                  .tokens.clone().unwrap_or_default(); // todo плохо.
              drop(structure);
              let result: Token = self.expression(tokens);
              value[index].setData    ( result.getData() );
              value[index].setDataType( *result.getDataType() );
            }
            structureLinesLen if structureLinesLen > 1 =>
            { // Это структура с вложением.
              let mut linesResult: Vec<Token> = Vec::new();
              for line in lines
              {
                let tokens: &mut Vec<Token> =
                  &mut line.read().unwrap()
                    .tokens.clone().unwrap_or_default(); // todo плохо.
                linesResult.push( self.expression(tokens) );
              }
              value[index] = Token::newNesting(
                vec![
                  Arc::new(RwLock::new(Line {
                    tokens: Some(linesResult),
                    lines: None
                  }))
                ]
              );
              value[index].setDataType( TokenType::Link ); // todo: Речь не о Link, а об Array?
            }
            _ => { setNone(value, index); } // В структуре не было вложений.
          }
        }
        //
      } else { setNone(value, index); } // Не нашли структуру.
    } else { setNone(value, index); } // Ошибка имени структуры.
    //
  }

  // ===============================================================================================

  /// Получает значение из ссылки на структуру;
  /// 
  /// Ссылка на структуру может состоять как из struct name, так и просто из цифр.
  pub fn linkExpression(
    &self, // Текущая структура - текущее пространство.
    currentStructureLink: Option< Arc<RwLock<Self>> >, // Структура предыдущего уровня ссылки.
    link: &mut Vec<String>, // Осталось читать.
    parameters: Option< Vec<Token> >
  ) -> Token
  { // Обработка динамического выражение.
    if link[0].starts_with('[')
    { // Получаем динамическое выражение между [].
      link[0] = format!("{{{}}}", &link[0][1..link[0].len()-1]);
      // Получаем новую строку значения из обработки выражения.
      link[0] = self.formatQuote(link[0].clone());
    }
    // Обработка пути.
    match link[0].parse::<usize>()
    { // Проверяем тип.
      Ok(lineNumber) => 
      { // Если мы нашли цифру в ссылке, значит это номер на линию в структуре.
        // Номер ссылается только на пространство currentStructureLink.
        link.remove(0);

        if let Some(ref currentStructureLock) = currentStructureLink 
        { // Это структура, которая была передана предыдущем уровнем ссылки.
          // Только в ней мы можем найти нужную линию.
          let currentStructure: RwLockReadGuard<Self> = currentStructureLock.read().unwrap(); // todo: это можно вынести в временный блок.

          if let Some(lines) = &currentStructure.lines
          {
            if let Some(line) = lines.get(lineNumber)                                         // Для получения линии и выхода из read().unwrap().
            { // Тогда просто берём такую линию по её номеру.
              let mut lineTokens: Vec<Token> =
              {
                line.read().unwrap()
                  .tokens.clone().unwrap_or_default() // todo плохо
              };

              if !lineTokens.is_empty()
              { // Проверяем количество токенов, чтобы понять, можем ли мы вычислить что-то.
                
                // В линии есть хотя бы 1 токен.
                if !link.is_empty()
                { // Если дальше есть продолжение ссылки.
                  link.insert(0, lineTokens[0].getData().toString().unwrap_or_default());

                  // То мы сначала проверяем что такая структура есть во внутреннем пространстве.
                  if currentStructure.getStructureByName(
                    &lineTokens[0].getData().toString().unwrap_or_default()
                  ).is_some()
                  {
                    drop(currentStructure);
                    return currentStructureLock.read().unwrap()
                      .linkExpression(None, link, parameters);
                  }
                  // А если такой ссылки там не было, то значит она в self.
                  drop(currentStructure);
                  return self.linkExpression(currentStructureLink, link, parameters);
                } else
                if parameters.is_some()
                { // Если это был просто запуск метода, то запускаем его.
                  drop(currentStructure);

                  let mut parametersToken: Token = Token::newNesting( Vec::new() ); // todo: add parameters.
                  parametersToken.setDataType( TokenType::CircleBracketBegin );

                  let mut expressionTokens: Vec<Token> = vec![
                    Token::new( TokenType::Word, lineTokens[0].getData() ),
                    parametersToken
                  ];

                  return currentStructureLock.read().unwrap()
                    .expression(&mut expressionTokens);
                } else
                { // если дальше нет продолжения ссылки.
                  if *lineTokens[0].getDataType() == TokenType::Word
                  {
                    // Если это слово, то это либо ссылка т.к. там много значений в ней;
                    // Либо это структура с одиночным вложением и мы можем его забрать сейчас.

                    if let Some(childStructureLink) = currentStructure.getStructureByName(
                      &lineTokens[0].getData().toString().unwrap_or_default()
                    )
                    { // Пробуем проверить что там 1 линия вложена в структуре;
                      // После чего сможем посчитать её значение.
                      let childStructure: RwLockReadGuard<Self> = childStructureLink.read().unwrap();
                      if lines.len() == 1
                      {
                        if let Some(lines) = &childStructure.lines
                        {
                          if let Some(line) = lines.first()
                          { // По сути это просто 0 линия через expression.
                            let mut lineTokens: Vec<Token> =
                              line.read().unwrap()
                                .tokens.clone().unwrap_or_default(); // todo плохо
                            drop(childStructure);
                            return self.expression(&mut lineTokens);
                            //
                          }
                        }
                        //
                      }
                    }
                    // Если ничего не получилось, значит оставляем ссылку.
                    drop(currentStructure);
                    return Token::new( TokenType::Link, lineTokens[0].getData() );
                  } else 
                  { // Если это не слово, то смотрим на результат expression.
                    return self.expression(&mut lineTokens);
                  }
                  //
                }
              } else 
              { // В линии нет токенов, нам нечего вычислять.
                return Token::newEmpty( TokenType::None );
              }
            }
            //
          }
        }
        //
      }
      Err(_) => 
      { // Если мы не нашли цифры в ссылке, значит это просто struct name;
        // Они работают в пространстве первого self, но могут и внутри себя,
        // поэтому блок далее определяет ссылку на необходимую структуру;
        let structureLink: Option< Arc<RwLock<Self>> > =
          if let Some(currentStructureLink) = currentStructureLink
          { // Если нет в локальном окружении, то просто берём из self.
            
            // Если есть в локальном окружении.
            let structure: RwLockReadGuard<Self> = currentStructureLink.read().unwrap();
            let hasLines: bool = 
            {
              let childStructureLink: Option< Arc<RwLock<Self>> > = structure.getStructureByName(&link[0]);
              if let Some(childStructureLink) = childStructureLink
              {
                if let Some(lines) = &childStructureLink.read().unwrap().lines {
                  !lines.is_empty()
                } else { false }
                //
              } else { false }
              //
            };

            if hasLines {
              structure.getStructureByName(&link[0])
            } else {
              self.getStructureByName(&link[0])
            }
            //
          } else { self.getStructureByName(&link[0]) };
        // Далее мы работаем с полученной ссылкой пространства.
        link.remove(0);
        if let Some(structureLink) = structureLink
        { // Это структура которую мы нашли по имени в self пространстве.

          // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
          // Обработка нативной библиотеки.
          // Проверяем: остались ли ещё сегменты пути (имя метода).
          if !link.is_empty() 
          {
            // Читаем структуру, которая представляет загруженную библиотеку.
            let structureGuard: RwLockReadGuard<Self> = structureLink.read().unwrap();

            // Если структура имеет тип Pointer — это динамическая библиотека.
            if structureGuard.dataType == StructureType::Pointer
            {
              // Далее извлекаем указатель на библиотеку, сохранённый в lines[0].tokens[0].
              // Библиотека там лежит как токен типа String с путём к файлу библиотеки.

              // Из токена извлекаем сырые байты (путь к библиотеке).
              let bytes: Bytes = {

                // Получаем вектор линий структуры (в нашем случае lines[0] хранит токен).
                let linesVec: &Vec< Arc<RwLock<Line>> > = match &structureGuard.lines {
                  Some(v) => v,
                  None => return Token::newEmpty(TokenType::None)
                };
                // Берём первую линию (индекс 0).
                let lineLock: &Arc<RwLock<Line>> = match linesVec.first() {
                  Some(l) => l,
                  None => return Token::newEmpty(TokenType::None)
                };
                // Читаем линию, чтобы получить её токены.
                let line: RwLockReadGuard<Line> = lineLock.read().unwrap();
                // Токены линии — здесь должен быть один токен типа String.
                let tokensVec: &[Token] = match &line.tokens {
                  Some(t) => t,
                  None => return Token::newEmpty(TokenType::None)
                };
                // Берём первый (и единственный) токен.
                let nativeToken: &Token = match tokensVec.first() {
                  Some(t) => t,
                  None => return Token::newEmpty(TokenType::None)
                };
                // Убеждаемся, что токен действительно типа String.
                if *nativeToken.getDataType() != TokenType::String {
                  return Token::newEmpty(TokenType::None);
                }
                
                nativeToken.getData()
              };
              drop(structureGuard);
              let raw: &[u8] = match bytes.getAll() {
                Some(r) => r,
                None => return Token::newEmpty(TokenType::None)
              };

              // Преобразуем байты в строку (путь).
              let libraryPath: &str = match std::str::from_utf8(raw) {
                Ok(s) => s,
                Err(_) => return Token::newEmpty(TokenType::None)
              };
              
              // Формируем токен Nesting: одна линия с двумя токенами-строками.
              return Token::newNesting(vec![
                Arc::new(RwLock::new(Line {
                  tokens: Some(vec![
                    // Путь к lib, например: "./libprint.so".
                    Token::new(TokenType::String, libraryPath),
                    // Имя метода, который вызывают; например: "method" в lib.method(...).
                    Token::new(TokenType::String, link[0].clone()) // todo Но кстати оно больше не надо будет? зачем тогда .clone.
                  ]),
                  lines: None
                }))
              ]);
              //
            }
          }
          
          // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
          // todo desc
          if link.is_empty()
          { // Закончилась ли ссылка?
            
            // Если это конец, то берём последнюю структуру и работаем с ней.
            let structure: RwLockReadGuard<Self> = structureLink.read().unwrap();
            if let Some(lines) = &structure.lines
            {
              if lines.len() == 1
              { // Если это просто одиночное значение, то просто выдаём его
                // По сути это просто 0 линия через expression.
                //
                // todo Требуется уточнение: сюда попадают две разные структуры,
                //  у обеих lines.len() == 1, а ветка рассчитана только на первую.
                //
                //  a.rt:
                //    c = 10                                        # 1. значение
                //    f(name: String) { println(f"Hi, {name}") }    # 2. функция из одной строки
                //
                //  main.rt:
                //    a.c          — 10. Ради этого ветка и написана:
                //                   у конца ссылки берём значение.
                //    a.f("World") — тоже сюда, parameters = Some(["World"]),
                //                   но ветка их не читает. Тело считается через self.expression()
                //                   в scope вызывающего: name берётся из main.rt, а не из аргумента.
                //
                //  Запуск метода (Some(parameters) → structure.parent.expression) есть
                //  только в ветке false ниже, то есть для тел из 2+ строк.
                //
                //  Чтобы различать чтение и вызов, parameters должен быть None там, где
                //  скобок нет. Сейчас expressionWith передаёт Some(vec![]) и для `a.c + 1`,
                //  поэтому простая проверка на Some здесь сломает чтение значений.
                //

                let mut lineTokens: Vec<Token> =
                {
                  lines[0].read().unwrap()
                    .tokens.clone().unwrap_or_default() // todo плохо.
                };
                drop(structure);
                return self.expression(&mut lineTokens);
              }
              else if let Some(parameters) = parameters
              { // Здесь могут быть параметры функции или Some(vec![]) для процедуры;
                // В ином случае, это просто ссылка.
                
                // Если это был просто запуск метода, то запускаем его.
                let mut parametersToken: Token = Token::newNesting(
                  vec![
                    Arc::new(RwLock::new(Line
                    {
                      tokens: Some(parameters),
                      lines: None
                    }))
                  ]
                );
                parametersToken.setDataType( TokenType::CircleBracketBegin );

                let mut expressionTokens: Vec<Token> = vec![
                  Token::new( TokenType::Word, structure.name.clone().unwrap_or_default() ), // todo плохо.
                  parametersToken
                ];

                if let Some(structureParent) = structure.parent.clone()
                {
                  drop(structure);
                  return structureParent.read().unwrap()
                    .expression(&mut expressionTokens);
                }

                return Token::newEmpty(TokenType::None);
              } else
              { // Если это просто ссылка, то оставляем её.
                return Token::new( TokenType::Link, structure.name.clone().unwrap_or_default() ); // todo плохо.
              }
              //
            }
            
          // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
          } else
          { // Если нет, значит продолжаем её чтение.
            return self.linkExpression(Some(structureLink), link, parameters);
          }
          // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
        }
        //
      }
    }
    // если всё было плохо, то просто используем пустой результат.
    Token::newEmpty(TokenType::None)
  }

  // ===============================================================================================

  /// Принимает formatQuote типы и получает возможное значение обычной строки;
  /// 
  /// В основном всё сводится к получению токенов в {} через Token::readTokens(),
  /// после чего результат проходит через expression и мы получаем обычную строку на выходе.
  fn formatQuote(&self, tokenData: String) -> String 
  {
    let mut result: String = String::new(); // Cтрока которая будет получена в конце.
    let mut expressionBuffer: String = String::new(); // Буфер для выражения между {}.
    let mut expressionRead: bool = false; // Флаг чтения в буфер выражения.

    let chars: Vec<char> = tokenData.chars().collect(); // Всех символы в строке.
    let charsLength: usize = chars.len(); // Количество всех символов в строке.

    let mut i: usize = 0; // Указатель на текущий символ.
    let mut c: char; // Текущий символ.

    while i < charsLength 
    { // Читаем символы.
      c = chars[i];
      match c 
      {
        '{' =>
        { // Начинаем чтение выражения.
          expressionRead = true;
        }
        '}' =>
        { // Заканчиваем чтение выражения.
          expressionRead = false;
          expressionBuffer += "\n"; // Это нужно чтобы успешно завершить чтение линии Tokenizer::readTokens().

          let mut expressionBufferTokens: Vec<Token> = 
          {
            readTokensSimple(&mut expressionBuffer.as_bytes().to_vec())[0] 
              // Получаем результат выражения в виде ссылки на буферную линию.
              .read().unwrap() // Читаем ссылку и
              .tokens.clone()  // получаем все токены линии
              .unwrap_or_default() // todo плохо
          };
          // Отправляем все токены линии как выражение.
          if let Some(expressionData) = 
            self.expression(&mut expressionBufferTokens).getData().toString()
          { // Записываем результат посчитанный между {}.
            result += &expressionData;
          }
          // Обнуляем буфер, вдруг далее ещё есть выражения между {}.
          expressionBuffer = String::new();
        }
        _ => 
        { // Запись символов кроме {}.
          if expressionRead
          { // Если флаг чтения активен, то записываем символы выражения.
            expressionBuffer.push(c);
          } else 
          { // Если флаг чтения не активен, то это просто символы.
            result.push(c);
          }
        }
      }
      
      // Продолжаем чтение символов строки.
      i += 1;
    }
    
    // Отдаём новую строку.
    result
  }
  
  // ===============================================================================================

  /// Основная функция, которая получает результат выражения состоящего из токенов;
  /// Сначала она проверяет что это single токен, но если нет,
  /// то в цикле перебирает возможные варианты
  ///
  /// Значение выражения нужно вызывающему коду, поэтому FFI-вызов внутри
  /// вернёт результат ([`FfiExpect::Infer`]). Для вызовов-операторов и для
  /// присваивания с типом слева используйте [`Structure::expressionWith`].
  pub fn expression(&self, value: &mut Vec<Token>) -> Token 
  {
    self.expressionWith(value, &FfiExpect::Infer)
  }

  /// То же, что [`Structure::expression`], но с явным ожиданием результата FFI-вызова:
  /// - `Discard` — вызов-оператор (`lib.print(x)`), результат не нужен;
  /// - `Infer`   — тип слева не указан (`a = lib.f(x)`), тип берётся от правой части;
  /// - `Typed`   — тип слева указан (`a: I32 = lib.f(x)`), правая часть кастуется к левой.
  ///
  /// Ожидание относится только к вызовам верхнего уровня этого выражения;
  /// вложенные выражения (параметры вызова, скобки) считаются через `expression`.
  /// 
  /// todo Ожидание общее для всех FFI-вызовов верхнего уровня: в `a: I32 = f() + g()` оба читаются как I32
  pub fn expressionWith(&self, value: &mut Vec<Token>, expect: &FfiExpect) -> Token 
  {
    let mut valueLength: usize = value.len(); // Получаем количество токенов в выражении.
    // todo: Возможно следует объединить с нижним циклом, всё равно проверять токены по очереди
    // 1 токен
    // todo: возможно стоит сразу проверять что тут не Figure, Square, Circle скобки
    if valueLength == 1
    { // Если это выражение с 1 токеном, то
      match *value[0].getDataType()
      { // Проверяем возможные варианты.
        TokenType::None => value[0].setDataType(TokenType::None),
        TokenType::Link =>
        { // Если это TokenType::Link, то
          let data: String = value[0].getData().toString().unwrap_or_default(); // token data.
          let mut link: Vec<String> = Self::parseLink(&data);
          let linkResult: Token = self.linkExpression(None, &mut link, None); // Получаем результат от data.
          match *linkResult.getDataType() // Предполагаем изменение dataType.
          {
            TokenType::Word =>
            { // Если это TokenType::Word то теперь это будет TokenType::Link.
              value[0].setDataType( TokenType::Link );
            }
            _ =>
            { // Если это другие типы, то просто ставим новый dataType.
              value[0].setDataType( *linkResult.getDataType() );
            }
          }
          value[0].setData( linkResult.getData() ); // Ставим новый data.
        }
        TokenType::Word =>
        { // Если это TokenType::Word, то
          let data: String = value[0].getData().toString().unwrap_or_default(); // token data.
          let linkResult: Token = self.linkExpression(None, &mut vec![data], None); // Получаем результат от data.
          value[0].setDataType( *linkResult.getDataType() ); // Ставим новый dataType.
          value[0].setData( linkResult.getData() );  // Ставим новый data.
        }
        TokenType::FormattedRawString | TokenType::FormattedString | TokenType::FormattedChar =>
        { // Если это форматные варианты Char, String, RawString.
          if let Some(valueData) = value[0].getData().toString()
          { // Получаем data этого токена и сразу вычисляем его значение.
            value[0].setData( self.formatQuote(valueData) );
            // Получаем новый тип без formatted.
            match *value[0].getDataType()
            {
              TokenType::FormattedRawString => value[0].setDataType(TokenType::RawString),
              TokenType::FormattedString => value[0].setDataType(TokenType::String),
              TokenType::FormattedChar => value[0].setDataType(TokenType::Char),
              _ => value[0].setDataType(TokenType::None)
            }
            //
          }
        }
        TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float =>
        { // Токен бесконечен: сразу обрубаем до потолка u64/i64/f64 (#71),
          // дальше структура приводит уже обрубленное значение к своему типу.
          normalizeToken(&mut value[0]);
        }
        _ => {} // Идём дальше.
      }
      return value[0].clone(); // Возвращаем результат в виде одного токена.
    }

    // Если это выражение не из одного токена,
    // то следует проверять каждый токен в цикле и
    // производить соответствующие операции.
    let mut i: usize = 0; // указатель на текущий токен.

    while i < valueLength
    { // Проверяем на использование методов,
      // на использование ссылок на структуру,
      // на использование простого выражения в скобках.
      match *value[i].getDataType()
      {
        TokenType::None => value[i].setDataType(TokenType::None),
        TokenType::FormattedRawString | TokenType::FormattedString | TokenType::FormattedChar =>
        { // Если это форматные варианты Char, String, RawString.
          if let Some(valueData) = value[0].getData().toString()
          { // Получаем data этого токена и сразу вычисляем его значение.
            value[0].setData( self.formatQuote(valueData) );
            // Получаем новый тип без formatted.
            match *value[0].getDataType()
            {
              TokenType::FormattedRawString => value[0].setDataType(TokenType::RawString),
              TokenType::FormattedString => value[0].setDataType(TokenType::String),
              TokenType::FormattedChar => value[0].setDataType(TokenType::Char),
              _ => value[0].setDataType(TokenType::None)
            }
            //
          }
        }
        TokenType::Link =>
        { // Это ссылка на структуру, может выдать значение, запустить метод и т.д.
          // todo ? хз что это, имелось ввиду не для ffi.
          //let parameters: Parameters = self.getCallParameters(value, i, &mut valueLength);

          let data: String = value[i].getData().toString().unwrap_or_default();
          let mut link: Vec<String> = Self::parseLink(&data);

          // Если следом реальные скобки вызова — вычисляем настоящие аргументы 
          // (а не пустой список), чтобы их можно было прокинуть как в FFI-путь ниже, 
          // так и в обычный nested-вызов через structure.parent (см. import()).
          let hasCallParens: bool =
            i+1 < valueLength && *value[i+1].getDataType() == TokenType::CircleBracketBegin;
          let realParameters: Vec<Token> = if hasCallParens
          {
            let bracketLines: Vec< Arc<RwLock<Line>> > = value[i+1].lines.clone().unwrap_or_default();
            Parameters::new(Some(bracketLines)).getAllExpressions(self).unwrap_or_default()
          } else { Vec::new() };

          let linkResult: Token = self.linkExpression(None, &mut link, Some(realParameters));
            //parameters.getAll()); todo? хз что это, имелось ввиду не для ffi.
          
          // Проверяем, не является ли результат вызовом динамической библиотеки.
          //
          // todo Правда это выглядит криво, вдруг другие nested будут. Мб тип ему сделать? Типо nativeCall.
          'none: 
          {
            if let Some(lines) = &linkResult.lines
            {
              if let Some(firstLineLink) = lines.first()
              {
                let firstLine: RwLockReadGuard<Line> = firstLineLink.read().unwrap();
                if let Some(tokens) = &firstLine.tokens 
                {
                  if tokens.len() == 2
                    && tokens[0].getDataType() == &TokenType::String
                    && tokens[1].getDataType() == &TokenType::String
                  {
                    let libraryPath: String = tokens[0].getData().toString().unwrap();
                    let methodName: String = tokens[1].getData().toString().unwrap();

                    // Получаем аргументы из value[i+1] - скобка;
                    // Без скобок это не вызов, а просто ссылка на метод.
                    if !(i+1 < valueLength && *value[i+1].getDataType() == TokenType::CircleBracketBegin) {
                      break 'none;
                    }
                    let bracketLines: Vec< Arc<RwLock<Line>> > =
                      value[i+1].lines.clone().unwrap_or_default();
                    let parameters: Parameters = Parameters::new(Some(bracketLines));
                    let mut parametersTokens: Vec<Token> = parameters.getAllExpressions(self).unwrap_or_default();

                    // Вызов через FFI.
                    // Если мы внутри FFI блока — используем scope retention. 
                    // Иначе —  временный scope через макрос.
                    // Тип результата задаёт expect: Discard / Infer / Typed (см. bridge::FfiExpect).
                    // todo Заменить string на abi-ffi
                    let ffiResult: Result<Token, String> = if let Some(result) = 
                      scopeStack::withCurrentFfiScope(|scope| {
                        callExternalWithScope(scope, &libraryPath, &methodName, &mut parametersTokens, expect)
                      }) 
                    { result } // Мы внутри FFI блока — scope уже удержан.
                    // Временный scope.
                    else { callExternal(&libraryPath, &methodName, &mut parametersTokens, expect) };
                    
                    //
                    match ffiResult
                    {
                      Ok(resultToken) =>
                      { // Токен результата занимает место ссылки на метод.
                        value[i] = resultToken;
                      }
                      Err(_) =>
                      {
                        value[i].setDataType(TokenType::None);
                        value[i].setData(None);
                      }
                      /* todo Вообще мог быть отдельный флаг для работы - чтобы выводить ошибки.
                           Или можно сделать это частью скрытых полей вывода по типу .error и т.д.    
                      Err(e) => {
                        eprintln!("[rts FFI] lib='{}' method='{}' err='{}'", libraryPath, methodName, e);
                        value[i].setDataType(TokenType::None);
                        value[i].setData(None);
                      }
                      */
                      //
                    }
                    // Скобки с аргументами уже использованы вызовом — убираем их из выражения,
                    // чтобы результат остался единственным значением (`libc.f(1) + 1`).
                    value.remove(i+1);
                    valueLength -= 1;
                  }
                  //
                }
              }
            } else 
            { // Стандартный вариант результата: обычный (не-FFI) nested-вызов
              // через structure.parent внутри linkExpression (см. import()),
              // либо просто значение по ссылке.
              value[i].setDataType( *linkResult.getDataType() );
              value[i].setData( linkResult.getData() );

              // Скобки вызова уже использованы (реальные аргументы вычислены
              // и переданы выше через realParameters) — убираем их из
              // выражения, аналогично FFI-ветке.
              if hasCallParens
              {
                value.remove(i+1);
                valueLength -= 1;
              }
            }
            //
          }
        } 
        TokenType::Minus =>
        { // Это выражение в круглых скобках, но перед ними отрицание -
          if i+1 < valueLength &&
             *value[i+1].getDataType() == TokenType::CircleBracketBegin
          { // Считаем выражение внутри скобок.
            value[i+1] =
            {
              if let Some(lines) = &value[i+1].lines
              {
                let line: RwLockReadGuard<Line> = lines[0].read().unwrap();
                if let Some(mut tokenTokens) = line.tokens.clone() // todo Может быть не 0.
                { // Если получилось то оставляем его.
                  self.expression(&mut tokenTokens)
                } else { Token::newEmpty(TokenType::None) } // Если не получилось, то просто None.
                //
              } else { Token::newEmpty(TokenType::None) }
            };
            // Меняем отрицание.
            let tokenData: String = value[i+1].getData().toString().unwrap_or_default();
            if tokenData.starts_with('-')
            { // Если это было отрицательное выражение, то делаем его положительным.
              value[i+1].setData(
                tokenData.chars().skip(1).collect::<String>()
              );
              value[i].setDataType(TokenType::Plus);
            } else
            { // Если это не было отрицательным выражением, то делаем его отрицательным.
              // todo Что тут?
              //value[i+1].setData(
              //  format!("-{}", tokenData)
              //);
              value[i+1].setData(tokenData);
            }

            i += 1; // Мы уже посчитали скобку.
          }
        }
        TokenType::CircleBracketBegin =>
        { // Это просто выражение в круглых скобках.
          value[i] =
            if let Some(linesLinks) = &value[i].lines
            {
              if !linesLinks.is_empty()
              {
                let line: RwLockReadGuard<Line> = linesLinks[0].read().unwrap();
                if let Some(mut tokenTokens) = line.tokens.clone() // todo Может быть не 0.
                { // Если получилось, то оставляем его.
                  self.expression(&mut tokenTokens)
                } else { Token::newEmpty(TokenType::None) } // Если не получилось, то просто None.
                //
              } else { Token::newEmpty(TokenType::None) } // Линий не было.
              //
            } else { Token::newEmpty(TokenType::None) }
        }
        _ =>
        { // Это либо метод, либо просто слово-структура.
          if i+1 < valueLength && *value[i+1].getDataType() == TokenType::CircleBracketBegin
          {
            // Запускает метод; но он может быть либо обычный, либо из ссылки.
            let structureName:String = value[i].getData().toString().unwrap_or_default();
            let mut runBasicMethod: bool = true;
            if let Some(structureLink) = self.getStructureByName(&structureName)
            { // Мы должны проверить, что структура имеет только одно вложение.
              let structure: RwLockReadGuard<Self> = structureLink.read().unwrap();
              if let Some(lines) = &structure.lines
              {
                
                if lines.len() == 1
                {
                  let line: RwLockReadGuard<Line> = lines[0].read().unwrap();
                  if let Some(tokens) = &line.tokens
                  {
                    
                    if tokens.len() == 1
                    {
                      // todo: Вообще должна быть проверка на TokenType::Link.
                      if *tokens[0].getDataType() == TokenType::Word
                      {
                        self.linkExpression(
                          None,
                          &mut [
                            tokens[0].getData().toString().unwrap_or_default()
                          ].to_vec(),
                          Some(vec![]) // todo: Передать параметры функции.
                        );
                        runBasicMethod = false; // Запуск метода по ссылке.
                      } // Если этот один токен не был ссылкой, то пропускаем.
                    } // Если больше одного токена, то пропускаем.
                    
                  }
                } // Если вложений больше 1, то пропускаем;.
                
                //
              } // Если линий нет, то пропускаем.
            } // Если структуры не было, то пропускаем.
            if runBasicMethod
            { // Запуск обычного метода.
              self.functionCall(value, &mut valueLength, i);
            }
          } else 
          {
            if *value[i].getDataType() == TokenType::Word
            { // Вычисляем значение для struct имени только при типе TokenType::Word.
              self.replaceStructureByName(value, i);
            }
          }
          //
        }
      }
      i += 1;
    }

    // Далее идут варианты математических и логических операций.
    // Проверка /
    // todo * пока отключен: его нужно включать вместе с проверкой Value::Mul
    self.expressionOp(value, &mut valueLength, &[TokenType::Divide]);

    // Проверка + и -
    self.expressionOp(value, &mut valueLength, &[TokenType::Plus, TokenType::Minus]);

    // Проверка сравнений: результат True/False (#39).
    // Идут после арифметики, поэтому `a + 1 = b` читается как `(a + 1) = b`.
    self.expressionOp(value, &mut valueLength,
      &[TokenType::Equals, TokenType::NotEquals,
        TokenType::GreaterThan, TokenType::LessThan,
        TokenType::GreaterThanOrEquals, TokenType::LessThanOrEquals]
    );

    // Проверка на логические операции `&` и `|`.
    //
    // todo #39: не подключены, добавятся вместе с таблицами троичной логики.
    //self.expressionOp(value, &mut valueLength,
    //  &[TokenType::Inclusion, TokenType::Joint]
    //);

    // Конец чтения выражения
    match valueLength != 0
    {
      // В том случае, если мы имеем всё ещё значение,
      // значит просто вернём 0 элемент, чтобы избавиться от него.
      true => value[0].clone(),
      
      // Если всё пусто, значит пусто.
      false => Token::newEmpty(TokenType::None)
    }
  }

  /// Получает значение операции по левому и правому выражению; Это зависимость для expression;
  /// 
  /// Кроме того, может обрабатывать отрицание при использовании TokenType::Minus.
  fn expressionOp(
    &self, 
    value: &mut Vec<Token>, 
    valueLength: &mut usize, 
    operations: &[TokenType]
  ) -> ()
  {
    let mut i: usize = 0;
    let mut token: Token;
    let mut tokenType: &TokenType;

    while i < *valueLength
    { 
      if *valueLength == 1 
      { // Если остался только 1 токен — дальше нечего делать.
        break;
      }
      if i == 0
      { // Если i == 0, не можем начать, потому что нужен оператор.
        i += 1; 
        continue;
      }

      // true - если будет входящий в operations операция.
      token = value[i].clone();
      tokenType = token.getDataType();
      if i+1 < *valueLength && operations.contains(tokenType)
      { // Вычисление заданной операции между двумя операндами.
        value[i-1] = calculate(tokenType, &value[i-1], &value[i+1]);

        value.remove(i); // remove op
        value.remove(i); // remove right value
        *valueLength -= 2;
        continue;
      } else
      // Подразумевается, что нет оператора - поэтому два операнда,
      // поэтому мы можем проверить:
      //   value -value2
      // Потому что минус входит в число и мы можем просто проверить 2 токена.
      // Это то же самое, что: `10-20` = `10+(-20)`.
      //
      // Это только для + и -; в остальных проходах слитый минус (`-6`)
      // обычный операнд и его нужно пропустить: `10 -6 / 2` = `10 + (-6 / 2)`.
      if operations.contains(&TokenType::Plus) &&
         matches!(*tokenType, TokenType::Int | TokenType::Float)
      {
        value[i-1] = calculate(&TokenType::Plus, &value[i-1], &value[i]);

        value.remove(i); // remove UInt.
        *valueLength -= 1;
        continue;
      }

      i += 1;
    }
  }
  
  // ===============================================================================================
}

// =================================================================================================
