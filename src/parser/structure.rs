/* /parser/structure
  структура, которая представляет свободную ячейку данных в памяти;
  имеет свои настройки, место хранения.
*/

pub mod parameters;

use crate::{
  tokenizer::{line::*, token::*, readTokens},
  parser::{value::*, uf64::*},
};

use std::{
  sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::parser::structure::parameters::Parameters;

// Addition for Structure ==========================================================================
/// Вычисляет по математической операции значение и тип нового токена из двух
pub fn calculate(op: &TokenType, leftToken: &Token, rightToken: &Token) -> Token 
{
  // Получаем значение левой части выражения
  let leftTokenDataType: TokenType = leftToken.getDataType().clone();
  let leftValue: Value = getValue(leftToken.getData().toString().unwrap_or_default(), &leftTokenDataType);
  // Получаем значение правой части выражения
  let rightTokenDataType: TokenType = rightToken.getDataType().clone();
  let rightValue: Value = getValue(rightToken.getData().toString().unwrap_or_default(), &rightTokenDataType);
  // Получаем значение выражения, а также предварительный тип
  let mut resultType: TokenType = TokenType::UInt;
  let resultValue: String = match *op 
  {
    TokenType::Plus     => (leftValue + rightValue).to_string(),
    TokenType::Minus    => (leftValue - rightValue).to_string(),
    TokenType::Multiply => (leftValue * rightValue).to_string(),
    TokenType::Divide   => (leftValue / rightValue).to_string(),
    TokenType::Inclusion => 
    { 
      resultType = TokenType::Bool;
      match leftValue.toBool() || rightValue.toBool() 
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    TokenType::Joint => 
    { 
      resultType = TokenType::Bool;
      match leftValue.toBool() && rightValue.toBool() 
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    TokenType::Equals => 
    { 
      resultType = TokenType::Bool;
      match leftValue == rightValue 
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    TokenType::NotEquals => 
    { 
      resultType = TokenType::Bool;
      match leftValue != rightValue
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    TokenType::GreaterThan => 
    { 
      resultType = TokenType::Bool;
      match leftValue > rightValue 
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    TokenType::LessThan => 
    { 
      resultType = TokenType::Bool;
      match leftValue < rightValue 
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    TokenType::GreaterThanOrEquals => 
    { 
      resultType = TokenType::Bool;
      match leftValue >= rightValue 
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    TokenType::LessThanOrEquals => 
    { 
      resultType = TokenType::Bool;
      match leftValue <= rightValue 
      {
        true  => String::from("1"),
        false => String::from("0")
      }
    }
    _ => "0".to_string(),
  };
  // После того как значение было получено,
  // Смотрим какой точно тип выдать новому токену
  // todo: if -> match
  match resultType != TokenType::Bool 
  {
    false => {}
    true => 
    {
      if leftTokenDataType == TokenType::String || rightTokenDataType == TokenType::String
      {
        resultType = TokenType::String;
      } else
      if matches!(leftTokenDataType, TokenType::Int | TokenType::UInt) &&
          rightTokenDataType == TokenType::Char
      { //
        resultType = leftTokenDataType.clone();
      } else
      if leftTokenDataType == TokenType::Char
      {
        resultType = TokenType::Char;
      } else
      if leftTokenDataType == TokenType::Float || rightTokenDataType == TokenType::Float
      {
        resultType = TokenType::Float;
      } else
      if leftTokenDataType == TokenType::UFloat || rightTokenDataType == TokenType::UFloat
      {
        resultType = TokenType::UFloat;
      } else
      if leftTokenDataType == TokenType::Int || rightTokenDataType == TokenType::Int
      {
        resultType = TokenType::Int;
      }
    }
  }
  // return
  Token::new(resultType, resultValue)
}
/// Зависимость для calculate;
/// Считает значение левой и правой части выражения
fn getValue(tokenData: String, tokenDataType: &TokenType) -> Value 
{
  match tokenDataType
  {
    TokenType::None =>
    {
      Value::None()
    }
    TokenType::Int =>
    {
      tokenData.parse::<i64>()
        .map(Value::Int)
        .unwrap_or(Value::Int(0))
    },
    TokenType::UInt =>
    {
      tokenData.parse::<u64>()
        .map(Value::UInt)
        .unwrap_or(Value::UInt(0))
    },
    TokenType::Float =>
    {
      tokenData.parse::<f64>()
        .map(Value::Float)
        .unwrap_or(Value::Float(0.0))
    },
    TokenType::UFloat =>
    {
      tokenData.parse::<f64>()
        .map(uf64::from)
        .map(Value::UFloat)
        .unwrap_or(Value::UFloat(uf64::from(0.0)))
    },
    TokenType::Char =>
    { // todo: добавить поддержку операций с TokenType::formattedChar
      tokenData.parse::<char>()
        .map(|x| Value::Char(x))
        .unwrap_or(Value::Char('\0'))
    },
    TokenType::String =>
    {
      tokenData.parse::<String>()
        .map(|x| Value::String(x))
        .unwrap_or(Value::String("".to_string()))
    },
    TokenType::Bool =>
    {
      match tokenData == "0"
      {
        true  => Value::UInt(0),
        false => Value::UInt(1)
      }
    },
    _ => Value::UInt(0)
  }
}

// StructureMut ====================================================================================
/// Обозначает уровень изменения структуры
#[derive(PartialEq)]
#[derive(Clone)]
pub enum StructureMut
{
  /// Ожидает первое значение и превратится в Constant
  Final,
  /// Не может быть изменена, присваивается в момент создания
  Constant,
  /// Может изменять только значение
  Variable,
  /// Может изменять и значение и тип данных
  Dynamic
}
impl ToString for StructureMut
{ // todo convert -> fmt::Display ?
  fn to_string(&self) -> String
  {
    match self
    {
      StructureMut::Final => String::from("Final"),
      StructureMut::Constant => String::from("Constant"),
      StructureMut::Variable => String::from("Variable"),
      StructureMut::Dynamic => String::from("Dynamic"),
    }
  }
}

// DataType ========================================================================================
/// Тип данных структуры
#[derive(PartialEq)]
#[derive(Clone)]
pub enum StructureType
{
// primitives
  None,
  Any,
  Link,

  Bool,

  UInt,
  Int,

  UFloat,
  Float,

  // todo
  //Rational,
  //Complex,

  Char,
  String,
  RawString,

  FormattedChar,
  FormattedString,
  FormattedRawString,

  Method,

  List, // todo List<Type>

  // todo
  // Date

// custom
  /// Позволяет создавать пользовательские типы
  Custom(String),
}
impl ToString for StructureType
{ // todo convert -> fmt::Display ?
  fn to_string(&self) -> String
  {
    match self
    { // primitives
      StructureType::None => String::from("None"),
      StructureType::Any => String::from("Any"),
      StructureType::Link => String::from("Link"),

      StructureType::Bool => String::from("Bool"),

      StructureType::UInt => String::from("UInt"),
      StructureType::Int => String::from("Int"),

      StructureType::UFloat => String::from("UFloat"),
      StructureType::Float => String::from("Float"),

      StructureType::Char => String::from("Char"),
      StructureType::String => String::from("String"),
      StructureType::RawString => String::from("RawString"),

      StructureType::FormattedChar => String::from("FormattedChar"),
      StructureType::FormattedString => String::from("FormattedString"),
      StructureType::FormattedRawString => String::from("FormattedRawString"),

      StructureType::Method => String::from("Method"),

      StructureType::List => String::from("List"),

      // custom
      StructureType::Custom(value) => value.clone(),
    }
  }
}

pub trait ToTokenType
{
  /// Преобразует StructureType в TokenType
  fn toTokenType(&self) -> TokenType;
}

impl ToTokenType for StructureType
{
  fn toTokenType(&self) -> TokenType
  {
    match self
    {
      StructureType::UInt => TokenType::UInt,
      StructureType::Int => TokenType::Int,
      StructureType::UFloat => TokenType::UFloat,
      StructureType::Float => TokenType::Float,
      StructureType::String => TokenType::String,
      StructureType::Char => TokenType::Char,
      // StructureType::Rational => TokenType::Rational,
      // StructureType::Complex => TokenType::Complex,
      _ => TokenType::None, // todo ?
    }
  }
}
// Structure =======================================================================================
/// Свободная структура данных
#[derive(Clone)]
pub struct Structure 
{
  /// Уникальное имя;
  /// Если не будет указано, значит это временная структура
  pub name: Option<String>,

  /// Уровень изменения
  pub mutable: StructureMut,

  /// Тип данных
  pub dataType: StructureType,

  /// Ссылки на вложенные линии
  pub lines: Option< Vec< Arc<RwLock<Line>> > >,

  /// Входные параметры
  /// todo Не используется в коде
  pub parameters: Parameters,

  /// Выходной результат
  /// None => procedure
  /// else => function
  pub result: Option<Token>,

  /// Ссылки на вложенные структуры
  pub structures: Option< Vec< Arc<RwLock<Structure>> > >,

  /// Ссылка на родителя
  pub parent: Option< Arc<RwLock<Structure>> >,

  /// todo Комментарий + возможно не нужно т.к. можно лучше
  pub lineIndex: usize,
}
impl Structure 
{
  pub fn new
  (
    name:     Option<String>,
    mutable:  StructureMut,
    dataType: StructureType,
    lines:    Option< Vec< Arc<RwLock<Line>> > >,
    parent:   Option< Arc<RwLock<Structure>> >,
  ) -> Self 
  {
    Structure 
    {
      name,
      mutable,
      dataType,
      lines,
      parameters: Parameters::new(None),
      result: None,
      structures: None,
      parent,
      lineIndex: 0
    }
  }

  /// Ищет структуру по имени (даже если это ссылка)
  ///
  /// Пример: "parent.child.grandchild" будет искать:
  ///   1. "parent" в корневых структурах
  ///   2. "child" в дочерних структурах "parent"
  ///   3. "grandchild" в дочерних структурах "child"
  /// 
  /// todo Не смотрит выше self. Должен ли?
  pub fn getStructureByName(&self, name: &str) -> Option<Arc<RwLock<Structure>>>
  {
    // Разбиваем имя на части по разделителю '.'
    let segments: Vec<String> = name.split('.')
      .map(|s| s.to_string())
      .collect();

    if segments.is_empty() {
      return None;
    }

    let mut current_structure: Option<Arc<RwLock<Structure>>> = None;

    for segment in segments.iter() {
      let children_opt: Option<Vec<Arc<RwLock<Structure>>>> = match &current_structure {
        None => self.structures.clone(), // Клонируем Arc, нет проблем с временем жизни
        Some(structure) => {
          let structure_guard = structure.read().unwrap();
          structure_guard.structures.clone() // Клонируем внутренний вектор
        }
      };

      let mut found = false;
      let mut next_structure: Option<Arc<RwLock<Structure>>> = None;

      if let Some(children) = children_opt {
        for child in children {
          let child_guard = child.read().unwrap();
          if let Some(child_name) = &child_guard.name {
            if child_name == segment {
              found = true;
              next_structure = Some(child.clone());
              break;
            }
          }
        }
      }

      if !found {
        return None;
      }

      current_structure = next_structure;
    }

    current_structure
  }

  /// Добавляет новую вложенную структуру в текущую структуру
  pub fn pushStructure(&mut self, structure: Structure) -> ()
  { 
    match self.structures.is_none() 
    {
      true => 
      { // Если не было ещё структур, то создаём новый вектор
        self.structures = Some( vec!(Arc::new(RwLock::new(structure))) );
      } 
      false =>
      { // Если уже есть структуры, то просто push делаем
        match self.structures
        { None => {} Some(ref mut structures) =>
        {
          structures.push( Arc::new(RwLock::new(structure)) );
        }}
      }
    }
  }

  /// Выполняет операцию со структурой,
  /// для этого требует левую и правую часть выражения,
  /// кроме того, требует передачи родительской структуры,
  /// чтобы было видно возможные объявления в ней
  pub fn structureOp(&self, structureLink: Arc<RwLock<Structure>>, op: TokenType, leftPartMutable: StructureMut, rightPart: Vec<Token>) -> ()
  {
    match op
    { // Принимаем только математические операции
      TokenType::Equals |
      TokenType::PlusEquals |
      TokenType::MinusEquals |
      TokenType::MultiplyEquals |
      TokenType::DivideEquals => {},
      _ => return
    }

    match op == TokenType::Equals 
    {
      true => 
      { // Приравнивание правой части выражения к левой части выражения

        // todo должен быть вариант с вложением ?
        // Если нет вложений

        let mut rightPartValue: Token = self.expression(&mut rightPart.clone());

        let mut structure: RwLockWriteGuard<'_, Structure> = structureLink.write().unwrap();

        // Изменяем тип структуры если он не был указан
        match
          structure.dataType == StructureType::None ||
          leftPartMutable == StructureMut::Dynamic // Dynamic может изменить dataType просто так
        {
          true =>
          {
            match leftPartMutable != StructureMut::Variable
            { false => {} true =>
            { // Будет присвоено только Final | Dynamic
              structure.dataType = rightPartValue.getDataType().toStructureType();
            }}
          }
          false =>
          { // Требуется выполнить преобразование в указанный тип данных
            rightPartValue.setDataType(structure.dataType.toTokenType());
          }
        }

        match leftPartMutable == StructureMut::Final
        { false => {} true =>
        { // Изменяем mutable если это был Final
          structure.mutable = StructureMut::Constant;
        }}

        // Приравниваем новое значение структуре
        structure.lines =
          Some(vec![
            Arc::new(RwLock::new(
              Line
              {
                tokens: Some(vec![ rightPartValue ]),
                indent: None,
                lines:  None,
                parent: None
              }
            ))
          ]);
      }  
      false =>
      { // Иные операторы, например += -= *= /=
        // получаем левую и правую часть
        // todo сейчас тут много ошибок
        let leftValue: Token = 
        {
          let structure: RwLockReadGuard<'_, Structure> = structureLink.read().unwrap();
          match &structure.lines
          {
            None => Token::newEmpty(TokenType::None),
            Some(lines) =>
              match lines.len() > 0
              {
                false => Token::newEmpty(TokenType::None),
                true =>
                  self.expression(
                    &mut lines[0].read().unwrap()
                      .tokens.clone()
                      .unwrap_or_default() // todo плохо
                )
              }
          }
          //
        };
        let rightPart: Token = self.expression(&mut rightPart.clone()); // todo: возможно не надо клонировать токены, но скорее надо

        // Далее обрабатываем саму операцию
        let mut structure: RwLockWriteGuard<'_, Structure> = structureLink.write().unwrap();
        match op 
        { // Определяем тип операции
          TokenType::PlusEquals => 
          { 
            structure.lines = 
              Some(vec![
                Arc::new(RwLock::new( 
                  Line {
                    tokens: Some(vec![ calculate(&TokenType::Plus, &leftValue, &rightPart) ]),
                    indent: None,
                    lines:  None,
                    parent: None
                  }
                ))
              ]);
          }
          _ => {} // todo: Дописать другие варианты; а также добавит для них отдельные тесты
        }
        //if op == TokenType::PlusEquals     { structure.value = calculate(&TokenType::Plus,     &leftValue, &rightValue); } else 
        //if op == TokenType::MinusEquals    { structure.value = calculate(&TokenType::Minus,    &leftValue, &rightValue); } else 
        //if op == TokenType::MultiplyEquals { structure.value = calculate(&TokenType::Multiply, &leftValue, &rightValue); } else 
        //if op == TokenType::DivideEquals   { structure.value = calculate(&TokenType::Divide,   &leftValue, &rightValue); }
      }
    }
  }

  /// Вычисляем значение для struct имени типа TokenType::Word
  fn replaceStructureByName(&self, value: &mut Vec<Token>, index: usize) -> ()
  {
    fn setNone(value: &mut Vec<Token>, index: usize) 
    { // Возвращаем пустое значение
      value[index].setData(None);
      value[index].setDataType(TokenType::None);
    }

    match value[index].getData().toString() 
    {
      None => { setNone(value, index); } // Ошибка имени структуры
      Some(structureName) => 
      {
        match self.getStructureByName(&structureName) 
        {
          None => { setNone(value, index); } // Не нашли структуру
          Some(structureLink) => 
          {
            let structure: RwLockReadGuard<'_, Structure> = structureLink.read().unwrap();
            // Если это просто обращение к имени структуры
            match &structure.lines
            { None => {} Some(lines) =>
            {
              let structureLinesLen: usize = lines.len();
              match structureLinesLen
              {
                1 =>
                { // Структура с одним вложением
                  let tokens: &mut Vec<Token> =
                    &mut lines[0]
                      .read().unwrap()
                      .tokens.clone().unwrap_or_default(); // todo плохо
                  let _ = drop(structure);
                  let result: Token = self.expression(tokens);
                  value[index].setData    ( result.getData().clone() );
                  value[index].setDataType( result.getDataType().clone() );
                }
                structureLinesLen if structureLinesLen > 1 =>
                { // Это структура с вложением
                  let mut linesResult: Vec<Token> = Vec::new();
                  for line in lines
                  {
                    let tokens: &mut Vec<Token> =
                      &mut line.read().unwrap()
                        .tokens.clone().unwrap_or_default(); // todo плохо
                    linesResult.push( self.expression(tokens) );
                  }
                  value[index] = Token::newNesting(
                    vec![
                      Line
                      {
                        tokens: Some(linesResult),
                        indent: None,
                        lines: None,
                        parent: None
                      }
                    ]
                  );
                  value[index].setDataType( TokenType::Link ); // todo: Речь не о Link, а об Array?
                }
                _ => { setNone(value, index); } // В структуре не было вложений
              }
              //
            }}
            //
          }
        }
        //
      }
    }
    //
  }

  /// Получает значение из ссылки на структуру;
  /// Ссылка на структуру может состоять как из struct name, так и просто из цифр.
  pub fn linkExpression(&self, currentStructureLink: Option< Arc<RwLock<Structure>> >, link: &mut Vec<String>, parameters: Option< Vec<Token> >) -> Token
  { // Обработка динамического выражение
    match link[0].starts_with('[')
    { false => {} true => { // Получаем динамическое выражение между []
      link[0] = format!("{{{}}}", &link[0][1..link[0].len()-1]);
      // Получаем новую строку значения из обработки выражения
      link[0] = self.formatQuote(link[0].clone());
    }}
    // Обработка пути
    match link[0].parse::<usize>() 
    { // Проверяем тип
      Ok(lineNumber) => 
      { // Если мы нашли цифру в ссылке, значит это номер на линию в структуре;
        // Номер ссылается только на пространство currentStructureLink
        link.remove(0);

        if let Some(ref currentStructureLock) = currentStructureLink 
        { // Это структура, которая была передана предыдущем уровнем ссылки;
          // Только в ней мы можем найти нужную линию
          let currentStructure: RwLockReadGuard<'_, Structure> = currentStructureLock.read().unwrap(); // todo: это можно вынести в временный блок

          match &currentStructure.lines
          { None => {} Some(lines) =>
          {
            if let Some(line) = lines.get(lineNumber)                                   //       для получения линии и выхода из read().unwrap()
            { // Тогда просто берём такую линию по её номеру
              let mut lineTokens: Vec<Token> =
              {
                line.read().unwrap()
                  .tokens.clone().unwrap_or_default() // todo плохо
              };

              match lineTokens.len() > 0
              { // Проверяем количество токенов, чтобы понять, можем ли мы вычислить что-то;
                false =>
                { // В линии нет токенов, нам нечего вычислять
                  return Token::newEmpty( TokenType::None );
                }
                true =>
                { // В линии есть хотя бы 1 токен
                  if link.len() != 0
                  { // Если дальше есть продолжение ссылки
                    link.insert(0, lineTokens[0].getData().toString().unwrap_or_default());

                    // То мы сначала проверяем что такая структура есть во внутреннем пространстве
                    match currentStructure.getStructureByName(
                      &lineTokens[0].getData().toString().unwrap_or_default()
                    )
                    { None => {} Some(_) =>
                    {
                      let _ = drop(currentStructure);
                      return currentStructureLock.read().unwrap()
                        .linkExpression(None, link, parameters);
                    }}
                    // А если такой ссылки там не было, то значит она в self
                    let _ = drop(currentStructure);
                    return self.linkExpression(currentStructureLink, link, parameters);
                  } else
                  if let Some(_) = parameters
                  { // Если это был просто запуск метода, то запускаем его
                    let _ = drop(currentStructure);

                    let mut parametersToken: Token = Token::newNesting( Vec::new() ); // todo: add parameters
                    parametersToken.setDataType( TokenType::CircleBracketBegin );

                    let mut expressionTokens: Vec<Token> = vec![
                      Token::new( TokenType::Word, lineTokens[0].getData() ),
                      parametersToken
                    ];

                    return currentStructureLock.read().unwrap()
                      .expression(&mut expressionTokens);
                  } else
                  { // если дальше нет продолжения ссылки
                    match *lineTokens[0].getDataType() == TokenType::Word
                    {
                      false =>
                      { // Если это не слово, то смотрим на результат expression
                        return self.expression(&mut lineTokens);
                      }
                      true =>
                      { // Если это слово, то это либо ссылка т.к. там много значений в ней;
                        // Либо это структура с одиночным вложением и мы можем его забрать сейчас.

                        match currentStructure.getStructureByName(
                          &lineTokens[0].getData().toString().unwrap_or_default()
                        )
                        { None => {} Some(childStructureLink) =>
                        { // Пробуем проверить что там 1 линия вложена в структуре;
                          // После чего сможем посчитать её значение.
                          let childStructure: RwLockReadGuard<'_, Structure> = childStructureLink.read().unwrap();
                          match lines.len() == 1
                          { false => {} true =>
                          {
                            match &childStructure.lines
                            { None => {} Some(lines) =>
                            {
                              match lines.get(0)
                              { None => {} Some(line) =>
                              { // По сути это просто 0 линия через expression
                                let mut lineTokens: Vec<Token> =
                                  {
                                    line.read().unwrap()
                                      .tokens.clone().unwrap_or_default() // todo плохо
                                  };
                                let _ = drop(childStructure);
                                return self.expression(&mut lineTokens);
                                //
                              }}
                              //
                            }}
                            //
                          }}
                          //
                        }}
                        // Если ничего не получилось, значит оставляем ссылку
                        return Token::new( TokenType::Link, lineTokens[0].getData() );
                      }
                    }
                    //
                  }
                }
                //
              }
            }
            //
          }}
          //
        }
      }
      Err(_) => 
      { // Если мы не нашли цифры в ссылке, значит это просто struct name;
        // Они работают в пространстве первого self, но могут и внутри себя,
        // поэтому блок далее определяет ссылку на необходимую структуру;
        let structureLink: Option< Arc<RwLock<Structure>> > =
          match currentStructureLink
          { // Если нет в локальном окружении, то просто берём из self
            None => self.getStructureByName(&link[0]),
            Some(currentStructureLink) => 
            { // Если есть в локальном окружении
              let structure: RwLockReadGuard<'_, Structure> = currentStructureLink.read().unwrap();
              let hasLines: bool = 
              {
                let childStructureLink: Option< Arc<RwLock<Structure>> > = structure.getStructureByName(&link[0]);
                match childStructureLink 
                { None => false, Some(childStructureLink) =>
                {
                  match &childStructureLink.read().unwrap().lines
                  { None => false, Some(lines) =>
                  {
                    match lines.len() != 0
                    {
                      true  => true,
                      false => false
                    }
                    //
                  }}
                  //
                }}
                //
              };

              match hasLines
              {
                true  => structure.getStructureByName(&link[0]),
                false => self.getStructureByName(&link[0])
              }
            }
          };
        // Далее мы работаем с полученной ссылкой пространства;
        link.remove(0);
        match structureLink
        {
          None => {}
          Some(structureLink) => 
          { // Это структура которую мы нашли по имени в self пространстве
            match link.len() == 0
            { // Закончилась ли ссылка?
              false =>
              { // Если нет, значит продолжаем её чтение
                return self.linkExpression(Some(structureLink), link, parameters);
              }  
              true =>
              { // Если это конец, то берём последнюю структуру и работаем с ней
                let structure: RwLockReadGuard<'_, Structure> = structureLink.read().unwrap();
                match &structure.lines
                { None => {} Some(lines) =>
                {
                  match lines.len() == 1
                  {
                    true =>
                    { // Если это просто одиночное значение, то просто выдаём его
                      // По сути это просто 0 линия через expression
                      let mut lineTokens: Vec<Token> =
                      {
                        lines[0].read().unwrap()
                          .tokens.clone().unwrap_or_default() // todo плохо
                      };
                      let _ = drop(structure);
                      return self.expression(&mut lineTokens);
                    }
                    false => match parameters
                    { // Здесь могут быть параметры функции или Some(vec![]) для процедуры;
                      // В ином случае, это просто ссылка;
                      None =>
                      { // Если это просто ссылка, то оставляем её
                        return Token::new( TokenType::Link, structure.name.clone().unwrap_or_default() ); // todo плохо
                      }
                      Some(parameters) =>
                      { // Если это был просто запуск метода, то запускаем его
                        let mut parametersToken: Token = Token::newNesting(
                          vec![
                            Line
                            {
                              tokens: Some(parameters),
                              indent: None,
                              lines: None,
                              parent: None
                            }
                          ]
                        );
                        parametersToken.setDataType( TokenType::CircleBracketBegin );

                        let mut expressionTokens: Vec<Token> = vec![
                          Token::new( TokenType::Word, structure.name.clone().unwrap_or_default() ), // todo плохо
                          parametersToken
                        ];

                        match structure.parent.clone()
                        { None => {} Some(structureParent) =>
                        {
                          let _ = drop(structure);
                          return structureParent.read().unwrap()
                            .expression(&mut expressionTokens);
                        }}

                        return Token::newEmpty(TokenType::None);
                      }
                      //
                    }
                  }
                  //
                }}
                //
              }
            }
            //
          }
        }
        //
      }
    }
    // если всё было плохо, то просто используем пустой результат
    Token::newEmpty(TokenType::None)
  }

  /// Принимает formatQuote типы и получает возможное значение обычной строки;
  /// В основном всё сводится к получению токенов в {} через Token::readTokens(),
  /// после чего результат проходит через expression и мы получаем обычную строку на выходе.
  fn formatQuote(&self, tokenData: String) -> String 
  {
    let mut result:           String    = String::new(); // Cтрока которая будет получена в конце
    let mut expressionBuffer: String    = String::new(); // Буфер для выражения между {}
    let mut expressionRead:   bool      = false;         // Флаг чтения в буфер выражения

    let chars:       Vec<char> = tokenData.chars().collect(); // Всех символы в строке
    let charsLength: usize     = chars.len();                 // Количество всех символов в строке

    let mut i:      usize = 0; // Указатель на текущий символ
    let mut c:      char;      // Текущий символ

    while i < charsLength 
    { // Читаем символы
      c = chars[i];
      match c 
      {
        '{' =>
        { // Начинаем чтение выражения
          expressionRead = true;
        }
        '}' =>
        { // Заканчиваем чтение выражения
          expressionRead = false;
          expressionBuffer += "\n"; // Это нужно чтобы успешно завершить чтение линии Tokenizer::readTokens()

          let mut expressionBufferTokens: Vec<Token> = 
          {
            readTokens(
              expressionBuffer.as_bytes().to_vec(), 
              false
            )[0] // Получаем результат выражения в виде ссылки на буферную линию
              .read().unwrap() // Читаем ссылку и
              .tokens.clone()  // получаем все токены линии
              .unwrap_or_default() // todo плохо
          };
          // Отправляем все токены линии как выражение
          match self.expression(&mut expressionBufferTokens).getData().toString() 
          { None => {} Some(expressionData) =>
          { // Записываем результат посчитанный между {}
            result += &expressionData;
          }}
          // Обнуляем буфер, вдруг далее ещё есть выражения между {}
          expressionBuffer = String::new();
        }
        _ => 
        { // Запись символов кроме {}
          match expressionRead 
          {
            true => 
            { // Если флаг чтения активен, то записываем символы выражения
              expressionBuffer.push(c);
            }  
            false => 
            { // Если флаг чтения не активен, то это просто символы
              result.push(c);
            }
          }
        }
      }
      // Продолжаем чтение символов строки
      i += 1;
    }
    // Отдаём новую строку
    result
  }

  /// Получает параметры структуры вычисляя их значения;
  ///
  /// todo: требует пересмотра
  pub fn getStructureParameters(&self, value: &mut Vec<Token>) -> Vec<Token> 
  {
    let mut result: Vec<Token> = Vec::new();

    let mut expressionBuffer: Vec<Token> = Vec::new(); // buffer of current expression
    for (l, token) in value.iter().enumerate() 
    { // read tokens
      match *token.getDataType() == TokenType::Comma || l+1 == value.len()
      {
        true => 
        { // comma or line end
          match *token.getDataType() != TokenType::Comma
          { false => {} true =>
          {
            expressionBuffer.push( token.clone() );
          }}

          // todo: зачем это?
          /*
          if expressionBuffer.len() == 3 
          {
            if let Some(expressionData) = expressionBuffer[2].getData() 
            {
              expressionBuffer[0].setDataType( Some(getStructureResultType(expressionData)) );
            }
          }
          */
          result.push( expressionBuffer[0].clone() );

          expressionBuffer.clear();
        }  
        false => 
        { // push new expression token
          expressionBuffer.push( token.clone() );
        }
      }
    }

    result
  }

  /// Получает параметры при вызове структуры в качестве метода
  ///
  /// todo типы данных в параметрах
  pub fn getCallParameters(&self, value: &mut Vec<Token>, i: usize, valueLength: &mut usize) -> Parameters
  {
    let mut result: Option< Vec<Line> > = None;

    // Проверка и получение скобки
    let bracketToken: Option<&Token> = value.get(i+1);
    match bracketToken
    { None => {} Some(bracketToken) =>
    {

      // Проверка, что это круглая скобка
      match bracketToken.getDataType() != &TokenType::CircleBracketBegin
      {
        false => {}
        true => return Parameters::new(None)
      }

      // Получаем линии
      result = bracketToken.lines.clone(); // todo Тут точно клонирование?
    }}
    
    // Удаление скобки
    value.remove(i+1);
    *valueLength -= 1;

    //
    Parameters::new(result)
  }

  /// Основная функция, которая получает результат выражения состоящего из токенов;
  /// Сначала она проверяет что это single токен, но если нет,
  /// то в цикле перебирает возможные варианты
  pub fn expression(&self, value: &mut Vec<Token>) -> Token 
  {
    let mut valueLength: usize = value.len(); // Получаем количество токенов в выражении
    // todo: Возможно следует объединить с нижним циклом, всё равно проверять токены по очереди
    // 1 токен
    // todo: возможно стоит сразу проверять что тут не Figure, Square, Circle скобки
    match valueLength == 1
    { false => {} true =>
    { // Если это выражение с 1 токеном, то
      match *value[0].getDataType()
      { // Проверяем возможные варианты
        TokenType::None =>
        {
          value[0].setDataType(TokenType::None);
        }
        TokenType::Link =>
        { // Если это TokenType::Link, то
          let data: String = value[0].getData().toString().unwrap_or_default(); // token data
          let mut link: Vec<String> =
            data.split('.')
              .map(|s| s.to_string())
              .collect();
          let linkResult: Token  = self.linkExpression(None, &mut link, None); // Получаем результат от data
          match *linkResult.getDataType() // Предполагаем изменение dataType
          {
            TokenType::Word =>
            { // Если это TokenType::Word то теперь это будет TokenType::Link
              value[0].setDataType( TokenType::Link );
            }
            _ =>
            { // Если это другие типы, то просто ставим новый dataType
              value[0].setDataType( linkResult.getDataType().clone() );
            }
          }
          value[0].setData( linkResult.getData() ); // Ставим новый data
        }
        TokenType::Word =>
        { // Если это TokenType::Word, то
          let data:       String = value[0].getData().toString().unwrap_or_default(); // token data
          let linkResult: Token = self.linkExpression(None, &mut vec![data], None); // Получаем результат от data
          value[0].setDataType( linkResult.getDataType().clone() ); // Ставим новый dataType
          value[0].setData( linkResult.getData() );  // Ставим новый data
        }
        TokenType::FormattedRawString | TokenType::FormattedString | TokenType::FormattedChar =>
        { // Если это форматные варианты Char, String, RawString
          match value[0].getData().toString()
          { None => {} Some(valueData) =>
          { // Получаем data этого токена и сразу вычисляем его значение
            value[0].setData( self.formatQuote(valueData) );
            // Получаем новый тип без formatted
            match *value[0].getDataType()
            {
              TokenType::FormattedRawString => { value[0].setDataType(TokenType::RawString); }
              TokenType::FormattedString    => { value[0].setDataType(TokenType::String); }
              TokenType::FormattedChar      => { value[0].setDataType(TokenType::Char); }
              _ => { value[0].setDataType(TokenType::None); }
            }
          }}
        }
        _ => {} // Идём дальше;
      }
      return value[0].clone(); // Возвращаем результат в виде одного токена
    }}

    // Если это выражение не из одного токена,
    // то следует проверять каждый токен в цикле и
    // производить соответствующие операции
    let mut i: usize = 0; // указатель на текущий токен

    while i < valueLength 
    { // Проверяем на использование методов,
      // на использование ссылок на структуру,
      // на использование простого выражения в скобках
      match *value[i].getDataType()
      {
        TokenType::None =>
        {
          value[i].setDataType(TokenType::None);
        }
        TokenType::FormattedRawString | TokenType::FormattedString | TokenType::FormattedChar =>
        { // Если это форматные варианты Char, String, RawString;
          match value[0].getData().toString() 
          { None => {} Some(valueData) =>
          { // Получаем data этого токена и сразу вычисляем его значение
            value[0].setData( self.formatQuote(valueData) );
            // Получаем новый тип без formatted
            match *value[0].getDataType()
            {
              TokenType::FormattedRawString => { value[0].setDataType(TokenType::RawString); }
              TokenType::FormattedString    => { value[0].setDataType(TokenType::String); }
              TokenType::FormattedChar      => { value[0].setDataType(TokenType::Char); }
              _ => { value[0].setDataType(TokenType::None); }
            }
          }}
        }
        TokenType::Link =>
        { // Это ссылка на структуру, может выдать значение, запустить метод и т.д;
          let parameters: Parameters = self.getCallParameters(value, i, &mut valueLength);

          let     data: String = value[i].getData().toString().unwrap_or_default();
          let mut link: Vec<String> =
            data.split('.')
              .map(|s| s.to_string())
              .collect();

          // todo вернуть обратно
          //let linkResult: Token = self.linkExpression(None, &mut link, parameters.getAll());
          //value[i].setDataType( linkResult.getDataType() );
          //value[i].setData( linkResult.getData() );
        } 
        TokenType::Minus =>
        { // это выражение в круглых скобках, но перед ними отрицание -
          match
            i+1 < valueLength &&
            *value[i+1].getDataType() == TokenType::CircleBracketBegin
          { false => {} true =>
          { // считаем выражение внутри скобок
            value[i+1] =
            {
              match &value[i+1].lines
              {
                None => Token::newEmpty(TokenType::None),
                Some(lines) => match lines[0].tokens.clone() // todo Может быть не 0
                {
                  Some(mut tokenTokens) =>
                  { // если получилось то оставляем его
                    self.expression(&mut tokenTokens)
                  }
                  None =>
                  { // если не получилось, то просто None
                    Token::newEmpty(TokenType::None)
                  }
                }
              }
            };
            // Меняем отрицание
            let tokenData: String = value[i+1].getData().toString().unwrap_or_default();
            match tokenData.starts_with(|c: char| c == '-')
            {
              true =>
              { // Если это было отрицательное выражение, то делаем его положительным
                value[i+1].setData(
                  tokenData.chars().skip(1).collect::<String>()
                );
                value[i].setDataType(TokenType::Plus);
              }
              false =>
              { // Если это не было отрицательным выражением, то делаем его отрицательным
                //value[i+1].setData(
                //  format!("-{}", tokenData)
                //);
                value[i+1].setData(
                  format!("{}", tokenData)
                );
              }
            }

            i += 1; // Мы уже посчитали скобку
          }}
        }
        TokenType::CircleBracketBegin =>
        { // Это просто выражение в круглых скобках
          value[i] =
          {
            match &value[i].lines
            {
              None => Token::newEmpty(TokenType::None),
              Some(lines) => match lines[0].tokens.clone() // todo Может быть не 0
              {
                Some(mut tokenTokens) =>
                { // Если получилось то оставляем его
                  self.expression(&mut tokenTokens)
                }
                None =>
                { // Если не получилось, то просто None
                  Token::newEmpty(TokenType::None)
                }
              }
            }
          };
        }
        _ =>
        { // Это либо метод, либо просто слово-структура
          match i+1 < valueLength && *value[i+1].getDataType() == TokenType::CircleBracketBegin
          {
            true =>
            { // Запускает метод; но он может быть либо обычный, либо из ссылки;
              let structureName:String = value[i].getData().toString().unwrap_or_default();
              let mut runBasicMethod: bool = true;
              match self.getStructureByName(&structureName)
              {
                None => {} // Если структуры не было, то пропускаем;
                Some(structureLink) =>
                { // Мы должны проверить, что структура имеет только одно вложение;
                  let structure: RwLockReadGuard<'_, Structure> = structureLink.read().unwrap();
                  match &structure.lines
                  {
                    None => {} // Если линий нет, то пропускаем
                    Some(lines) =>
                    {
                      match lines.len() == 1
                      {
                        false => {} // Если вложений больше 1, то пропускаем;
                        true =>
                        {
                          let line: RwLockReadGuard<'_, Line> = lines[0].read().unwrap();
                          match &line.tokens
                          { None => {} Some(tokens) =>
                          {
                            match tokens.len() == 1
                            {
                              false => {} // Если больше одного токена, то пропускаем;
                              true =>
                              {
                                // todo: Вообще должна быть проверка на TokenType::Link
                                match *tokens[0].getDataType() == TokenType::Word
                                {
                                  false => {} // Если этот один токен не был ссылкой, то пропускаем;
                                  true =>
                                  {
                                    self.linkExpression(
                                      None,
                                      &mut [
                                        tokens[0].getData().toString().unwrap_or_default()
                                      ].to_vec(),
                                      Some(vec![]) // todo: Передать параметры функции
                                    );
                                    runBasicMethod = false; // Запуск метода по ссылке
                                  }
                                }
                                //
                              }
                            }
                            //
                          }}
                          //
                        }
                      }
                      //
                    }
                  }
                  //
                }
              }
              match runBasicMethod
              { false => {} true =>
              { // Запуск обычного метода
                self.functionCall(value, &mut valueLength, i);
              }}
            }
            false => match *value[i].getDataType()
            { // Вычисляем значение для struct имени только при типе TokenType::Word
              TokenType::Word =>
              {
                self.replaceStructureByName(value, i);
              }
              _ => {}
            }
          }
        }
      }
      i += 1;
    }

    // Далее идут варианты математических и логических операций
    // Проверка на логические операции 1
    // todo Работало за другие операторы (преждевременно + или -)
    //self.expressionOp(value, &mut valueLength,
    //  &[TokenType::Equals, TokenType::NotEquals,
    //    TokenType::GreaterThan, TokenType::LessThan,
    //    TokenType::GreaterThanOrEquals, TokenType::LessThanOrEquals]
    //);

    // Проверка на логические операции 2
    // todo Работало за другие операторы (преждевременно + или -)
    //self.expressionOp(value, &mut valueLength,
    //  &[TokenType::Inclusion, TokenType::Joint]
    //);

    // Проверка * и /
    // todo Работало за другие операторы (преждевременно + или -)
    //self.expressionOp(value, &mut valueLength, &[TokenType::Multiply, TokenType::Divide]);

    // Проверка + и -
    self.expressionOp(value, &mut valueLength, &[TokenType::Plus, TokenType::Minus]);

    // Конец чтения выражения
    match valueLength != 0
    {
      true =>
      { // В том случае, если мы имеем всё ещё значение,
        // значит просто вернём 0 элемент, чтобы избавиться от него
        value[0].clone()
      }
      false =>
      { // Если всё пусто, значит пусто
        Token::newEmpty(TokenType::None)
      }
    }
  }

  /// Получает значение операции по левому и правому выражению; Это зависимость для expression.
  /// Кроме того, может обрабатывать отрицание при использовании TokenType::Minus
  fn expressionOp(&self, value: &mut Vec<Token>, valueLength: &mut usize, operations: &[TokenType])
  {
    let mut i: usize = 0;
    let mut token: Token;
    let mut tokenType: &TokenType;

    while i < *valueLength
    { // Проверка на логические операции
      match *valueLength == 1
      { false => {} true =>
      {
        break;
      }}
      match i == 0
      { false => {} true =>
      {
        i += 1; continue;
      }}

      token = value[i].clone();
      tokenType = token.getDataType();
      match i+1 < *valueLength && operations.contains(tokenType)
      {
        true =>
        {
          value[i-1] = calculate(tokenType, &value[i-1], &value[i+1]);

          value.remove(i); // remove op
          value.remove(i); // remove right value
          *valueLength -= 2;
          continue;
        }
        // value -value2
        false => match matches!(*tokenType, TokenType::Int | TokenType::Float)
        { false => {} true =>
        {
          value[i-1] = calculate(&TokenType::Plus, &value[i-1], &value[i]);

          value.remove(i); // remove UInt
          *valueLength -= 1;
          continue;
        }}
      }

      i += 1;
    }
  }
}
