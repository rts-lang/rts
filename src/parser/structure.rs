/* /parser/structure
  структура, которая представляет свободную ячейку данных в памяти;
  имеет свои настройки, место хранения.
*/

use crate::{
  logger::*,
  _exit, _exitCode,
  tokenizer::{line::*, token::*, readTokens},
  parser::{searchStructure, readLines, value::*, uf64::*},
};

use std::{
  io::{self, Write},
  process::{Command, Output, ExitStatus},
  str::SplitWhitespace,
  sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
  thread::sleep,
  time::Duration,
};

use rand::Rng;

// Addition for Structure ==========================================================================
/// Вычисляет по математической операции значение и тип нового токена из двух
pub fn calculate(op: &TokenType, leftToken: &Token, rightToken: &Token) -> Token 
{
  // Получаем значение левой части выражения
  let leftTokenDataType: TokenType = leftToken.getDataType().unwrap_or_default();
  let leftValue: Value = getValue(leftToken.getData().unwrap_or_default(), &leftTokenDataType);
  // Получаем значение правой части выражения
  let rightTokenDataType: TokenType = rightToken.getDataType().unwrap_or_default();
  let rightValue: Value = getValue(rightToken.getData().unwrap_or_default(), &rightTokenDataType);
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
        true  => { String::from("1") } 
        false => { String::from("0") }
      }
    }
    TokenType::Joint => 
    { 
      resultType = TokenType::Bool; 
      match leftValue.toBool() && rightValue.toBool() 
      {
        true  => { String::from("1") } 
        false => { String::from("0") }
      }
    }
    TokenType::Equals => 
    { 
      resultType = TokenType::Bool; 
      match leftValue == rightValue 
      {
        true  => { String::from("1") } 
        false => { String::from("0") }
      }
    }
    TokenType::NotEquals => 
    { 
      resultType = TokenType::Bool; 
      match leftValue != rightValue 
      {
        true  => { String::from("1") } 
        false => { String::from("0") }
      }
    }
    TokenType::GreaterThan => 
    { 
      resultType = TokenType::Bool; 
      match leftValue > rightValue 
      {
        true  => { String::from("1") } 
        false => { String::from("0") }
      }
    }
    TokenType::LessThan => 
    { 
      resultType = TokenType::Bool; 
      match leftValue < rightValue 
      {
        true  => { String::from("1") } 
        false => { String::from("0") }
      }
    }
    TokenType::GreaterThanOrEquals => 
    { 
      resultType = TokenType::Bool; 
      match leftValue >= rightValue 
      {
        true  => { String::from("1") } 
        false => { String::from("0") }
      }
    }
    TokenType::LessThanOrEquals => 
    { 
      resultType = TokenType::Bool; 
      match leftValue <= rightValue 
      {
        true  => { String::from("1") } 
        false => { String::from("0") }
      }
    }
    _ => "0".to_string(),
  };
  // После того как значение было получено,
  // Смотрим какой точно тип выдать новому токену
  // todo: if -> match
  match resultType != TokenType::Bool 
  {
    true => 
    {
      if leftTokenDataType == TokenType::String || rightTokenDataType == TokenType::String 
      { 
        resultType = TokenType::String;
      } else
      if (matches!(leftTokenDataType, TokenType::Int | TokenType::UInt) && 
          rightTokenDataType == TokenType::Char) 
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
    false => {}
  }
  // return
  Token::new( Some(resultType), Some(resultValue) )
}
/// Зависимость для calculate;
/// Считает значение левой и правой части выражения
fn getValue(tokenData: String, tokenDataType: &TokenType) -> Value 
{
  match tokenDataType {
    TokenType::Int    =>
    {
      tokenData.parse::<i64>()
        .map(Value::Int)
        .unwrap_or(Value::Int(0))
    },
    TokenType::UInt   =>
    {
      tokenData.parse::<u64>()
        .map(Value::UInt)
        .unwrap_or(Value::UInt(0))
    },
    TokenType::Float  =>
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
    TokenType::Char  =>
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
    TokenType::Bool   =>
    {
      match tokenData == "0"
      {
        true  => { Value::UInt(0) }
        false => { Value::UInt(1) }
      }
    },
    _ => Value::UInt(0),
  }
}

/// get structure result type
///
/// todo: вообще лучше бы это было в самом Token,
///       поскольку там есть перевод уже TokenType -> String
pub fn getStructureResultType(word: String) -> TokenType 
{
  match word.as_str() 
  {
    "Int"      => TokenType::Int,
    "UInt"     => TokenType::UInt,
    "Float"    => TokenType::Float,
    "UFloat"   => TokenType::UFloat,
    "Rational" => TokenType::Rational,
    "Complex"  => TokenType::Complex,
    "Char"     => TokenType::Char,
    "String"   => TokenType::String,
    "Bool"     => TokenType::Bool,
    _ => TokenType::Custom(word),
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
  /// Может изменять только значение, (зависит от наблюдателя => может меняться со временем)
  Variable,
  /// Может изменять и значение и тип данных, (зависит от наблюдателя => может меняться со временем)
  Dynamic
}

impl ToString for StructureMut
{ // todo convert -> fmt::Display ?
  fn to_string(&self) -> String
  {
    match self
    {
      StructureMut::Final => String::from("final"),
      StructureMut::Constant => String::from("constant"),
      StructureMut::Variable => String::from("variable"),
      StructureMut::Dynamic => String::from("dynamic"),
    }
  }
}

// Structure =======================================================================================
/// Свободная структура данных
#[derive(Clone)]
pub struct Structure 
{
  /// Уникальное имя
  /// todo option
  pub name: String,

  /// Уровень изменения структуры
  pub mutable: StructureMut,

  /// Ссылки на вложенные линии
  /// todo option
  pub lines: Vec< Arc<RwLock<Line>> >,

  /// Входные параметры
  pub parameters: Option< Vec<Token> >,

  /// Выходной результат
  /// None => procedure
  /// else => function
  pub result: Option<Token>,

  /// Ссылки на вложенные структуры
  pub structures: Option< Vec< Arc<RwLock<Structure>> > >,

  /// Ссылка на родителя
  pub parent: Option< Arc<RwLock<Structure>> >,

  /// todo comment
  pub lineIndex: usize,
}
impl Structure 
{
  pub fn new
  (
    name:    String,
    mutable: StructureMut,
    lines:   Vec< Arc<RwLock<Line>> >,
    parent:  Option< Arc<RwLock<Structure>> >,
  ) -> Self 
  {
    Structure 
    {
      name,
      mutable,
      lines,
      parameters: None, // todo: remove ?
      result: None,
      structures: None,
      parent,
      lineIndex: 0
    }
  }

  /// Ищет структуру по имени и возвращает либо None, либо ссылку на неё
  pub fn getStructureByName(&self, name: &str) -> Option< Arc<RwLock<Structure>> > 
  {
    match &self.structures 
    {
      None => {}
      Some(someStructures) => 
      {
        for childStructureLink in someStructures 
        {
          match name == childStructureLink.read().unwrap().name 
          {
            true  => { return Some( childStructureLink.clone() ); }
            false => {}
          }
        }
      }
    }
    // check the parent structure if it exists
    match &self.parent 
    {
      None => None,
      Some(parentLink) => 
      {
        parentLink.read().unwrap()
          .getStructureByName(name)
      }
    } 
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
      false => if let Some(ref mut structures) = self.structures 
      { // Если уже есть структуры, то просто push делаем
        structures.push( Arc::new(RwLock::new(structure)) );
      }
    }
  }

  /// get structure nesting
  ///
  /// todo: описание
  fn setStructureNesting(&self, structureNesting: &Vec<Token>, structureLines: &Vec< Arc<RwLock<Line>> >, newTokens: Vec<Token>) -> () 
  {
    match structureNesting.len()-1 > 1 
    {
      true => 
      { // go next
        //let nextStructureNesting: &[Token] = &structureNesting[1..];
        // todo: ?
      }  
      false => 
      {
        match structureLines.get( 
          // Получаем номер линии
          structureNesting[0]
            .getData().unwrap_or_default()
            .parse::<usize>().unwrap_or_default()
        ) 
        {
          None => {}
          Some(nestingLine) => 
          {
            nestingLine.write().unwrap()
              .tokens = newTokens;
          }
        }
      }
    }
  }

  /// Выполняет операцию со структурой,
  /// для этого требует левую и правую часть выражения,
  /// кроме того, требует передачи родительской структуры,
  /// чтобы было видно возможные объявления в ней
  pub fn structureOp(&self, structureLink: Arc<RwLock<Structure>>, op: TokenType, leftValue: Vec<Token>, rightValue: Vec<Token>) -> ()
  {
    match op
    {
      TokenType::Equals | TokenType::PlusEquals | TokenType::MinusEquals | 
      TokenType::MultiplyEquals | TokenType::DivideEquals => {},
      _ => { return },
    }

    // calculate new values
    /*
    let rightValue: Token = 
      if let Some(mut opValueTokens) = rightValue.tokens.clone() 
      {
        self.expression(&mut opValueTokens)
      } else 
      { // error
        Token::newEmpty(None)
      };
    */
    // =
    match op == TokenType::Equals 
    {
      true => 
      { // Если это простое приравнивание к структуре
        let mut structureNesting: Vec<Token> = Vec::new();
        for value in leftValue 
        {
          match value.getDataType().unwrap_or_default() == TokenType::SquareBracketBegin 
          {
            false => {}
            true =>
            {
              match value.tokens 
              {
                None => {}
                Some(mut valueTokens) => 
                {
                  structureNesting.push( 
                    self.expression(&mut valueTokens) 
                  );
                }
              }
            }
          }
        }
        match structureNesting.len() > 0 
        {
          true => 
          { // nesting
            self.setStructureNesting(
              &structureNesting, 
              &structureLink.read().unwrap().lines, 
              rightValue
            );
          }
          false => 
          { // not nesting
            let mut structure: RwLockWriteGuard<'_, Structure> = structureLink.write().unwrap();
            structure.lines = 
              vec![ 
                Arc::new(RwLock::new( 
                  Line {
                    tokens: vec![ self.expression(&mut rightValue.clone()) ],
                    indent: 0,
                    lines:  None,
                    parent: None
                  }
                ))
              ];
          }
        }
      }  
      false =>
      { // Иные операторы, например += -= *= /=
        // получаем левую и правую часть
        let leftValue: Token = 
        {
          let structure: RwLockReadGuard<'_, Structure> = structureLink.read().unwrap();
          match structure.lines.len() > 0
          {
            true => 
            {
              self.expression(
                &mut structure.lines[0].read().unwrap()
                  .tokens.clone())
            }  
            false => 
            {
              Token::newEmpty(Some(TokenType::None))
            }
          }
        };
        let rightValue: Token = self.expression(&mut rightValue.clone()); // todo: возможно не надо клонировать токены, но скорее надо

        // Далее обрабатываем саму операцию
        let mut structure: RwLockWriteGuard<'_, Structure> = structureLink.write().unwrap();
        match op 
        { // Определяем тип операции
          TokenType::PlusEquals => 
          { 
            structure.lines = 
              vec![ 
                Arc::new(RwLock::new( 
                  Line {
                    tokens: vec![ calculate(&TokenType::Plus, &leftValue, &rightValue) ],
                    indent: 0,
                    lines:  None,
                    parent: None
                  }
                ))
              ];
          }
          _ => {} // todo: Дописать другие варианты;
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
      value[index].setData    (None);
      value[index].setDataType(None);
    }

    match value[index].getData() 
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
            { // Если это просто обращение к имени структуры
              let structureLinesLen: usize = structure.lines.len();
              match structureLinesLen 
              {
                1 =>
                { // Структура с одним вложением
                  let tokens: &mut Vec<Token> = &mut structure.lines[0]
                                                  .read().unwrap()
                                                  .tokens.clone();
                  let _ = drop(structure);
                  let result: Token = self.expression(tokens);
                  value[index].setData    ( result.getData().clone() );
                  value[index].setDataType( result.getDataType().clone() );
                } 
                structureLinesLen if structureLinesLen > 1 =>
                { // Это структура с вложением
                  let mut linesResult: Vec<Token> = Vec::new();
                  for line in &structure.lines 
                  {
                    let tokens: &mut Vec<Token> =
                      &mut line.read().unwrap()
                        .tokens.clone();
                    linesResult.push( self.expression(tokens) );
                  }
                  value[index] = Token::newNesting( Some(linesResult) );
                  value[index].setDataType( Some(TokenType::Link) ); // todo: Речь не о Link, а об Array?
                } 
                _ => { setNone(value, index); } // В структуре не было вложений
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

  /// Получает значение из ссылки на структуру;
  /// Ссылка на структуру может состоять как из struct name, так и просто из цифр.
  fn linkExpression(&self, currentStructureLink: Option< Arc<RwLock<Structure>> >, link: &mut Vec<String>, parameters: Option< Vec<Token> >) -> Token
  { // Обработка динамического выражение
    match link[0].starts_with('[')
    {
      false => {}
      true => 
      { // Получаем динамическое выражение между []
        link[0] = format!("{{{}}}", &link[0][1..link[0].len()-1]);
        // Получаем новую строку значения из обработки выражения
        link[0] = self.formatQuote(link[0].clone());
      }
    }
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
          if let Some(line) = currentStructure.lines.get(lineNumber)                                   //       для получения линии и выхода из read().unwrap()
          { // Тогда просто берём такую линию по её номеру
            let mut lineTokens: Vec<Token> = 
            {
              line.read().unwrap()
                .tokens.clone()
            };

            match lineTokens.len() > 0 
            { // Проверяем количество токенов, чтобы понять, можем ли мы вычислить что-то;
              false =>
              { // В линии нет токенов, нам нечего вычислять
                return Token::newEmpty( Some(TokenType::None) );
              }
              true => 
              { // В линии есть хотя бы 1 токен
                if link.len() != 0 
                { // Если дальше есть продолжение ссылки
                  link.insert(0, lineTokens[0].getData().unwrap_or_default());

                  // То мы сначала проверяем что такая структура есть во внутреннем пространстве
                  match currentStructure.getStructureByName( 
                    &lineTokens[0].getData().unwrap_or_default() 
                  )
                  {
                    None => {}
                    Some(_) =>
                    {
                      let _ = drop(currentStructure);
                      return currentStructureLock.read().unwrap()
                        .linkExpression(None, link, parameters);
                    }
                  }
                  // А если такой ссылки там не было, то значит она в self
                  let _ = drop(currentStructure);
                  return self.linkExpression(currentStructureLink, link, parameters);
                } else 
                if let Some(_) = parameters
                { // Если это был просто запуск метода, то запускаем его
                  let _ = drop(currentStructure);
                  
                  let mut parametersToken: Token = Token::newNesting( Some(Vec::new()) ); // todo: add parameters
                  parametersToken.setDataType( Some(TokenType::CircleBracketBegin) );

                  let mut expressionTokens: Vec<Token> = vec![
                    Token::new( Some(TokenType::Word), lineTokens[0].getData() ),
                    parametersToken
                  ];

                  return currentStructureLock.read().unwrap()
                    .expression(&mut expressionTokens);
                } else 
                { // если дальше нет продолжения ссылки
                  match lineTokens[0].getDataType().unwrap_or_default() == TokenType::Word 
                  {
                    false =>
                    { // Если это не слово, то смотрим на результат expression
                      return self.expression(&mut lineTokens);
                    }
                    true => 
                    { // Если это слово, то это либо ссылка т.к. там много значений в ней;
                      // Либо это структура с одиночным вложением и мы можем его забрать сейчас.

                      match currentStructure.getStructureByName( 
                        &lineTokens[0].getData().unwrap_or_default() 
                      )
                      { // Пробуем проверить что там 1 линия вложена в структуре;
                        // После чего сможем посчитать её значение.
                        None => {}
                        Some(childStructureLink) => 
                        {
                          let childStructure: RwLockReadGuard<'_, Structure> = childStructureLink.read().unwrap();
                          match childStructure.lines.len() == 1 
                          {
                            false => {}
                            true => 
                            {
                              match childStructure.lines.get(0) 
                              { // По сути это просто 0 линия через expression
                                None => {}
                                Some(line) => 
                                { 
                                  let mut lineTokens: Vec<Token> = 
                                    {
                                      line.read().unwrap()
                                        .tokens.clone()
                                    };
                                  let _ = drop(childStructure);
                                  return self.expression(&mut lineTokens);
                                  //
                                }
                              }
                              //
                            }
                          }
                          //
                        }
                      }
                      // Если ничего не получилось, значит оставляем ссылку
                      return Token::new( Some(TokenType::Link), lineTokens[0].getData() );
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
                {
                  None => false,
                  Some(childStructureLink) => 
                  {
                    match 
                      childStructureLink.read().unwrap()
                        .lines.len() != 0 
                    {
                      true  => true,
                      false => false
                    }
                  }
                }
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
                match structure.lines.len() == 1 
                {
                  true => 
                  { // Если это просто одиночное значение, то просто выдаём его
                    match structure.lines.get(0) 
                    {
                      None => {}
                      Some(line) => 
                      { // По сути это просто 0 линия через expression
                        let mut lineTokens: Vec<Token> = 
                          {
                            line.read().unwrap()
                              .tokens.clone()
                          };
                        let _ = drop(structure);
                        return self.expression(&mut lineTokens);
                      }
                    }
                  } 
                  false => match parameters
                  { // Здесь могут быть параметры функции или Some(vec![]) для процедуры;
                    // В ином случае, это просто ссылка;
                    None =>
                    { // Если это просто ссылка, то оставляем её
                      return Token::new( Some(TokenType::Link), Some(structure.name.clone()) );
                    }
                    Some(_) =>
                    { // Если это был просто запуск метода, то запускаем его
                      let mut parametersToken: Token = Token::newNesting( parameters );
                      parametersToken.setDataType( Some(TokenType::CircleBracketBegin) );

                      let mut expressionTokens: Vec<Token> = vec![
                        Token::new( Some(TokenType::Word), Some(structure.name.clone()) ),
                        parametersToken
                      ];

                      match structure.parent.clone()
                      {
                        None => {}
                        Some(structureParent) => 
                        {
                          let _ = drop(structure);
                          return structureParent.read().unwrap()
                            .expression(&mut expressionTokens);
                        }
                      }

                      return Token::newEmpty( Some(TokenType::None) );
                    }
                  }
                  //
                }
              }
              //
            }
          }
        }
        //
      }
    }
    // если всё было плохо, то просто используем пустой результат
    Token::newEmpty( Some(TokenType::None) )
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
          };
          // Отправляем все токены линии как выражение
          match self.expression(&mut expressionBufferTokens).getData() 
          {
            None => {}
            Some(expressionData) => 
            { // Записываем результат посчитанный между {}
              result += &expressionData;
            }
          }
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
      match token.getDataType().unwrap_or_default() == TokenType::Comma || l+1 == value.len() 
      {
        true => 
        { // comma or line end
          match token.getDataType().unwrap_or_default() != TokenType::Comma 
          {
            true  => { expressionBuffer.push( token.clone() ); }
            false => {}
          }

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

  /// Получает параметры при вызове структуры в качестве метода;
  /// т.е. получает переданные значения через expression
  fn getCallParameters(&self, value: &mut Vec<Token>, i: usize, valueLength: &mut usize) -> Option< Vec<Token> >
  {
    match value.len() < 2 || value[i+1].getDataType().unwrap_or_default() != TokenType::CircleBracketBegin
    { // Проверяем, что обязательно существует круглая скобка рядом
      false => {}
      true => { return None; }
    }

    let mut result: Vec<Token> = Vec::new();
    match value.get(i+1).map(|v| &v.tokens) 
    { // Начинаем читать вложения в круглых скобках
      None => {}
      Some(tokens) => 
      {
        match tokens
        {
          None => {}
          Some(tokens) => 
          { // get bracket tokens
            let mut expressionBuffer: Vec<Token> = Vec::new(); // buffer of current expression
            for (l, token) in tokens.iter().enumerate() 
            { // read tokens
              match token.getDataType().unwrap_or_default() == TokenType::Comma || l+1 == tokens.len() 
              {
                true => 
                { // comma or line end
                  match token.getDataType().unwrap_or_default() != TokenType::Comma 
                  {
                    false => {}
                    true  => { expressionBuffer.push( token.clone() ); }
                  }
                  result.push( self.expression(&mut expressionBuffer) );
                  expressionBuffer.clear();
                }  
                false => 
                { // push new expression token
                  expressionBuffer.push( token.clone() );
                }
              }
              //
            }
          }
          //
        }
        // Удаляем скобки
        value.remove(i+1);
        *valueLength -= 1;
      }
      //
    }

    match result.len() == 0 
    {
      true  => { Some(vec![]) }
      false => { Some(result) }
    }
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
    {
      false => {}
      true =>
      { // Если это выражение с 1 токеном, то
        match value[0].getDataType().unwrap_or_default()
        { // Проверяем возможные варианты
          TokenType::Link =>
          { // Если это TokenType::Link, то
            let data: String = value[0].getData().unwrap_or_default(); // token data
            let mut link: Vec<String> =
              data.split('.')
                .map(|s| s.to_string())
                .collect();
            let linkResult: Token  = self.linkExpression(None, &mut link, None); // Получаем результат от data
            let linkType:   TokenType = linkResult.getDataType().unwrap_or_default(); // Предполагаем изменение dataType
            match linkType
            {
              TokenType::Word =>
              { // Если это TokenType::Word то теперь это будет TokenType::Link
                value[0].setDataType( Some(TokenType::Link) );
              }
              _ =>
              { // Если это другие типы, то просто ставим новый dataType
                value[0].setDataType( linkResult.getDataType() );
              }
            }
            value[0].setData( linkResult.getData() ); // Ставим новый data
          }
          TokenType::Word =>
          { // Если это TokenType::Word, то
            let data:       String = value[0].getData().unwrap_or_default();// token data
            println!("!!! A1 {}",data);
            let linkResult: Token  = self.linkExpression(None, &mut vec![data], None); // Получаем результат от data
            value[0].setDataType( linkResult.getDataType() ); // Ставим новый dataType
            value[0].setData( linkResult.getData() );  // Ставим новый data
          }
          TokenType::FormattedRawString | TokenType::FormattedString | TokenType::FormattedChar =>
          { // Если это форматные варианты Char, String, RawString
            match value[0].getData()
            {
              None => {}
              Some(valueData) =>
              { // Получаем data этого токена и сразу вычисляем его значение
                value[0].setData( Some(self.formatQuote(valueData)) );
                // Получаем новый тип без formatted
                match value[0].getDataType().unwrap_or_default()
                {
                  TokenType::FormattedRawString => { value[0].setDataType( Some(TokenType::RawString) ); }
                  TokenType::FormattedString    => { value[0].setDataType( Some(TokenType::String) ); }
                  TokenType::FormattedChar      => { value[0].setDataType( Some(TokenType::Char) ); }
                  _ => { value[0].setDataType( None ); }
                }
              }
            }
          }
          _ => {} // Идём дальше;
        }
        return value[0].clone(); // Возвращаем результат в виде одного токена
      }
    }

    // Если это выражение не из одного токена,
    // то следует проверять каждый токен в цикле и
    // производить соответствующие операции
    let mut i: usize = 0; // указатель на текущий токен

    while i < valueLength 
    { // Проверяем на использование методов,
      // на использование ссылок на структуру,
      // на использование простого выражения в скобках
      match value[i].getDataType().unwrap_or_default() 
      {
        TokenType::FormattedRawString | TokenType::FormattedString | TokenType::FormattedChar =>
        { // Если это форматные варианты Char, String, RawString;
          match value[0].getData() 
          {
            None => {}
            Some(valueData) => 
            { // Получаем data этого токена и сразу вычисляем его значение
              value[0].setData( Some(self.formatQuote(valueData)) );
              // Получаем новый тип без formatted
              match value[0].getDataType().unwrap_or_default() 
              {
                TokenType::FormattedRawString => { value[0].setDataType( Some(TokenType::RawString) ); }
                TokenType::FormattedString    => { value[0].setDataType( Some(TokenType::String) ); }
                TokenType::FormattedChar      => { value[0].setDataType( Some(TokenType::Char) ); }
                _ => { value[0].setDataType( None ); }
              }
            }
          }
        }
        TokenType::Link =>
        { // Это ссылка на структуру, может выдать значение, запустить метод и т.д;
          let expressions: Option< Vec<Token> > = self.getCallParameters(value, i, &mut valueLength);

          let     data: String = value[i].getData().unwrap_or_default();
          let mut link: Vec<String> =
            data.split('.')
              .map(|s| s.to_string())
              .collect();

          let linkResult: Token = self.linkExpression(None, &mut link, expressions);
          value[i].setDataType( linkResult.getDataType() );
          value[i].setData( linkResult.getData() );
        } 
        TokenType::Minus =>
        { // это выражение в круглых скобках, но перед ними отрицание -
          match
            i+1 < valueLength &&
            value[i+1].getDataType().unwrap_or_default() == TokenType::CircleBracketBegin
          {
            true => 
            { // считаем выражение внутри скобок
              value[i] = 
                match value[i+1].tokens.clone() 
                {
                  Some(mut tokenTokens) => 
                  { // если получилось то оставляем его
                    self.expression(&mut tokenTokens)
                  } 
                  None => 
                  { // если не получилось, то просто None
                    Token::newEmpty(None)
                  }
                };
              // Удаляем скобки
              value.remove(i+1); // remove UInt
              valueLength -= 1;
              // Меняем отрицание
              let tokenData: String = value[i].getData().unwrap_or_default();
              match tokenData.starts_with(|c: char| c == '-') 
              {
                true => 
                { // Если это было отрицательное выражение, то делаем его положительным
                  value[i].setData( 
                    Some( tokenData.chars().skip(1).collect() ) 
                  );
                }  
                false => 
                { // Если это не было отрицательным выражением, то делаем его отрицательным
                  value[i].setData( 
                    Some( format!("-{}", tokenData) )
                  );
                }
              }
            }
            false => {}
          }
        } 
        TokenType::CircleBracketBegin => 
        { // Это просто выражение в круглых скобках
          value[i] = 
            match value[i].tokens.clone() 
            {
              Some(mut tokenTokens) => 
              { // Если получилось то оставляем его
                self.expression(&mut tokenTokens)
              } 
              None => 
              { // Если не получилось, то просто None
                Token::newEmpty(None)
              }
            }
        }
        _ =>
        { // Это либо метод, либо просто слово-структура
          match i+1 < valueLength && value[i+1].getDataType().unwrap_or_default() == TokenType::CircleBracketBegin
          {
            true =>
            { // Запускает метод; но он может быть либо обычный, либо из ссылки;
              let structureName:String = value[i].getData().unwrap_or_default();
              let mut runBasicMethod: bool = true;
              match self.getStructureByName(&structureName)
              {
                None => {} // Если структуры не было, то пропускаем;
                Some(structureLink) =>
                { // Мы должны проверить, что структура имеет только одно вложение;
                  let structure: RwLockReadGuard<'_, Structure> = structureLink.read().unwrap();
                  match structure.lines.len() == 1
                  {
                    false => {} // Если вложений больше 1, то пропускаем;
                    true =>
                    {
                      let line: RwLockReadGuard<'_, Line> = structure.lines[0].read().unwrap();
                      match line.tokens.len() == 1
                      {
                        false => {} // Если больше одного токена, то пропускаем;
                        true =>
                        {
                          // todo: Вообще должна быть проверка на TokenType::Link
                          match line.tokens[0].getDataType().unwrap_or_default() == TokenType::Word
                          {
                            false => {} // Если этот один токен не был ссылкой, то пропускаем;
                            true =>
                            {
                              self.linkExpression(
                                None,
                                &mut [
                                  line.tokens[0].getData().unwrap_or_default()
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
                    }
                  }
                  //
                }
              }
              match runBasicMethod
              { // Запуск обычного метода;
                false => {}
                true =>
                {
                  self.functionCall(value, &mut valueLength, i);
                }
              }
            }
            false => match value[i].getDataType().unwrap_or_default()
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
    self.expressionOp(value, &mut valueLength, 
      &[TokenType::Equals, TokenType::NotEquals, 
        TokenType::GreaterThan, TokenType::LessThan,
        TokenType::GreaterThanOrEquals, TokenType::LessThanOrEquals]
    );

    // Проверка на логические операции 2
    self.expressionOp(value, &mut valueLength, 
      &[TokenType::Inclusion, TokenType::Joint]
    );
    
    // Проверка * и /
    self.expressionOp(value, &mut valueLength, &[TokenType::Multiply, TokenType::Divide]);
    
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
        Token::newEmpty(None)
      }
    }
  }

  /// Получает значение операции по левому и правому выражению; Это зависимость для expression.
  /// Кроме того, может обрабатывать отрицание при использовании TokenType::Minus
  fn expressionOp(&self, value: &mut Vec<Token>, valueLength: &mut usize, operations: &[TokenType]) 
  {
    let mut i: usize = 0;
    let mut token: Token;
    let mut tokenType: TokenType;

    while i < *valueLength 
    { // Проверка на логические операции
      match *valueLength == 1 
      {
        true  => { break; }
        false => {}
      }
      match i == 0 
      {
        true  => { i += 1; continue; }
        false => {}
      }

      token = value[i].clone();
      tokenType = token.getDataType().unwrap_or_default();
      match i+1 < *valueLength && operations.contains(&tokenType)
      {
        true => 
        {
          value[i-1] = calculate(&tokenType, &value[i-1], &value[i+1]);
          
          value.remove(i); // remove op
          value.remove(i); // remove right value
          *valueLength -= 2;
          continue;
        } 
        // value -value2
        false => match matches!(tokenType, TokenType::Int | TokenType::Float)
        {
          false => {}
          true =>
          {
            value[i-1] = calculate(&TokenType::Plus, &value[i-1], &value[i]);

            value.remove(i); // remove UInt
            *valueLength -= 1;
            continue;
          }
        }
      }

      i += 1;
    }
  }

  /// Запускает функцию;
  ///
  /// Функция - это такая структура, которая возвращает значение.
  ///
  /// Но кроме того, запускает не стандартные методы;
  /// В нестандартных методах могут быть процедуры, которые не вернут результат.

  /// todo: вынести все стандартные варианты в отдельный модуль
  ///
  /// todo: когда будет вынесено, то должна ожидать тип данных, который должен в Tokenizer::getWord() тоже
  pub fn functionCall(&self, value: &mut Vec<Token>, valueLength: &mut usize, i: usize) -> ()
  {
    let parameters: Option< Vec<Token> > = self.getCallParameters(value, i, valueLength);
    match parameters
    { // Запуск методов может содержать передаваемые параметры при обращении;
      None => {}
      Some(ref parameters) =>
      {
        match value[i].getData()
        {
          None =>
          { // Вариант в котором тип токена может быть типом данных => это cast в другой тип;
            match value[i].getDataType().unwrap_or_default()
            {
              TokenType::UInt =>
              { // Получаем значение выражения в типе
                // todo: Float, UFloat
                value[i].setDataType( Some(TokenType::UInt ) );
                value[i].setData    ( Some(parameters[0].getData().unwrap_or_default()) );
              }
              TokenType::Int =>
              { // Получаем значение выражения в типе
                value[i].setDataType( Some(TokenType::Int ) );
                value[i].setData    ( Some(parameters[0].getData().unwrap_or_default()) );
              }
              TokenType::String =>
              { // Получаем значение выражение в типе String
                // todo: подумать над formatted типами
                value[i].setDataType( Some(TokenType::String ) );
                value[i].setData    ( Some(parameters[0].getData().unwrap_or_default()) );
              }
              TokenType::Char =>
              { // Получаем значение выражения в типе Char
                // todo: проверить работу
                value[i].setDataType( Some(TokenType::Char) );
                value[i].setData(
                  Some(
                    (parameters[0].getData().unwrap_or_default()
                      .parse::<u8>().unwrap() as char
                    ).to_string()
                  )
                );
              }
              _ => {} // todo: Возможно custom варианты преобразований из custom
            }
          }
          Some(structureName) =>
          { // Вариант в котором это обращение к стандартной или custrom функции;
            // todo: проверка на нижний регистр

            // Далее идут базовые методы;
            // Эти методы ожидают аргументов
            'basicMethods:
            { // Это позволит выйти, если мы ожидаем не стандартные варианты
              match structureName.as_str()
              { // Проверяем на сходство стандартных функций

                // todo: создать resultType() ?
                //       для возвращения результата ожидаемого структурой

                "type" =>
                { // Возвращает тип данных переданной структуры
                  value[i].setDataType( Some(TokenType::String) );
                  value[i].setData    ( Some(parameters[0].getDataType().unwrap_or_default().to_string()) );
                }
                "mut" =>
                { // Возвращает уровень модификации переданной структуры
                  value[i].setDataType( Some(TokenType::String) );
                  let result: String =
                    match parameters[0].getData()
                    {
                      None => String::from(""),
                      Some(structureName) =>
                      { // Получили название структуры
                        println!("!!! {}", structureName);
                        match self.getStructureByName(&structureName)
                        {
                          None => String::from(""),
                          Some(structureLink) =>
                          { // Получили ссылку на структуру
                            let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
                            structure.mutable.to_string()
                          }
                        }
                      }
                    };
                  value[i].setData( Some(result) );
                }
                "randUInt" if parameters.len() > 1 =>
                { // Возвращаем случайное число типа UInt от min до max
                  let min: usize =
                    match parameters[0].getData()
                    {
                      Some(expressionData) => { expressionData.parse::<usize>().unwrap_or_default() }
                      None => { 0 }
                    };
                  let max: usize =
                    match parameters[1].getData()
                    {
                      Some(expressionData) => { expressionData.parse::<usize>().unwrap_or_default() }
                      None => { 0 }
                    };
                  let randomNumber: usize =
                    match min < max
                    {
                      true  => { rand::thread_rng().gen_range(min..=max) }
                      false => { 0 }
                    };
                  value[i].setDataType( Some(TokenType::UInt) );
                  value[i].setData    ( Some(randomNumber.to_string()) );
                }
                "len" =>
                { // Получаем размер структуры;
                  match parameters[0].getDataType().unwrap_or_default()
                  {
                    TokenType::None =>
                    { // Результат 0
                      value[i] = Token::new( Some(TokenType::UInt),Some(String::from("0")) );
                    }
                    TokenType::Char =>
                    { // Получаем размер символа
                      value[i] = Token::new( Some(TokenType::UInt),Some(String::from("1")) );
                    }
                    TokenType::String | TokenType::RawString =>
                    { // Получаем размер строки
                      value[i] = Token::new(
                        Some(TokenType::UInt),
                        Some(
                          parameters[0].getData().unwrap_or_default()
                            .chars().count().to_string()
                        )
                      );
                    }
                    _ =>
                    { // Получаем размер вложений в структуре
                      // Результат только в UInt
                      value[i].setDataType( Some(TokenType::UInt) );
                      // Получаем значение
                      match self.getStructureByName(&parameters[0].getData().unwrap_or_default())
                      {
                        Some(structureLink) =>
                        {
                          value[i].setData(
                            Some(
                              structureLink.read().unwrap()
                                .lines.len().to_string()
                            )
                          );
                        }
                        None =>
                        { // Результат 0 т.к. не нашли такой структуры
                          value[i].setData( Some(String::from("0")) );
                        }
                      }
                    }
                  }
                }
                "input" =>
                { // Получаем результат ввода

                  // Результат может быть только String
                  value[i].setDataType( Some(TokenType::String) );

                  match parameters[0].getData()
                  {
                    None => {}
                    Some(parametersData) =>
                    { // Это может быть выведено перед вводом;
                      // todo: возможно потом это лучше убрать,
                      //       т.к. программист сам может вызвать
                      //       такое через иные методы
                      print!("{}",parametersData);
                      io::stdout().flush().unwrap(); // forced withdrawal of old
                    }
                  }

                  let mut valueBuffer: String = String::new(); // временный буффер ввода
                  match io::stdin().read_line(&mut valueBuffer)
                  { // Читаем ввод
                    Ok(_) =>
                    { // Успешно ввели и записали
                      value[i].setData(
                        Some( valueBuffer.trim_end().to_string() )
                      );
                    }
                    Err(_) =>
                    { // Не удалось ввести, пустая строка
                      value[i].setData( Some(String::new()) );
                    }
                  }
                }
                "exec" =>
                { // Запускает что-то и возвращает строковый output работы
                  let data: String = parameters[0].getData().unwrap_or_default();
                  let mut parts: SplitWhitespace<'_> = data.split_whitespace();

                  let command: &str      = parts.next().expect("No command found in parameters"); // todo: no errors
                  let    args: Vec<&str> = parts.collect();

                  let output: Output =
                    Command::new(command)
                      .args(&args)
                      .output()
                      .expect("Failed to execute process"); // todo: no errors
                  let outputString: String = String::from_utf8_lossy(&output.stdout).to_string();
                  match !outputString.is_empty()
                  {
                    false => {}
                    true =>
                    { // result
                      value[i].setData    ( Some(outputString.trim_end().to_string()) );
                      value[i].setDataType( Some(TokenType::String) );
                    }
                  }
                }
                "execs" =>
                { // Запускает что-то и возвращает кодовый результат работы
                  // todo: Возможно изменение: Следует ли оставлять вывод stdout & stderr ?
                  //       -> Возможно следует сделать отдельные методы для подобных операций.
                  let data: String = parameters[0].getData().unwrap_or_default();
                  let mut parts: SplitWhitespace<'_> = data.split_whitespace();

                  let command: &str      = parts.next().expect("No command found in expression"); // todo: no errors
                  let    args: Vec<&str> = parts.collect();

                  let status: ExitStatus =
                    Command::new(command)
                      .args(&args)
                      .stdout(std::process::Stdio::null())
                      .stderr(std::process::Stdio::null())
                      .status()
                      .expect("Failed to execute process"); // todo: no errors
                  value[i].setData    ( Some(status.code().unwrap_or(-1).to_string()) );
                  value[i].setDataType( Some(TokenType::String) );
                }
                _ => { break 'basicMethods; } // Выходим, т.к. ожидается нестандартный метод
              }
              return;
            }
            // Если код не завершился ранее, то далее идут custom методы;
            { // Передаём параметры, они также могут быть None
              self.procedureCall(&structureName, parameters);
              // После чего решаем какой результат оставить
              match self.getStructureByName(&structureName)
              {
                None => {}
                Some(structureLink) =>
                { // По результату структуры, определяем пустой он или нет
                  match
                    &structureLink.read().unwrap()
                      .result
                  {
                    Some(result) =>
                    { // Результат не пустой, значит оставляем его
                      value[i].setData    ( result.getData() );
                      value[i].setDataType( result.getDataType().clone() );
                    }
                    None =>
                    { // Если результата структуры не было,
                      // значит это была действительно процедура
                      value[i].setData    ( None );
                      value[i].setDataType( None );
                    }
                  }
                }
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

  /// Запускает стандартные процедуры;
  /// Процедура - это такая структура, которая не возвращает результат.
  ///
  /// Но кроме того, запускает не стандартные методы;
  /// Из нестандартных методов, процедуры могут вернуть результат, в таком случае, их следует считать функциями.
  ///
  /// todo: вынести все стандартные варианты в отдельный модуль
  pub fn procedureCall(&self, structureName: &str, expressions: &Vec<Token>) -> ()
  { 
    if structureName.starts_with(|c: char| c.is_lowercase()) 
    { // Если название в нижнем регистре - то это точно процедура
      match structureName 
      { // Проверяем на сходство стандартных функций
        "println" =>
        { // println

          // todo: вывод всех expressions
          formatPrint( &format!("{}\n",&expressions[0].getData().unwrap_or_default()) );

          io::stdout().flush().unwrap(); // forced withdrawal of old
        }
        "print" =>
        { // print

          // todo: вывод всех expressions
          formatPrint( &expressions[0].getData().unwrap_or_default() );

          io::stdout().flush().unwrap(); // forced withdrawal of old
        }
        "clear" =>
        { // clear
          let _ = Command::new("clear")
            .status(); // Игнорируем ошибки
          // todo: однако можно выдавать результат boolean при ошибке
        }
        "go" =>
        { // Запускаем линию выше заново
          match &self.parent 
          {
            Some(parentLink) => 
            {
              let (mut lineIndex, lineLink): (usize, Arc<RwLock<Line>>) = 
              { // Это более безопасный вариант, чтобы использование parent закончилось
                // перед дальнейшим использованием ссылки на него
                let parent: RwLockReadGuard<'_, Structure> = parentLink.read().unwrap();
                let lineIndexBuffer: usize = parent.lineIndex;

                // Получаем ссылку на линию
                (lineIndexBuffer, parent.lines[lineIndexBuffer].clone())
              };
              // Используем линию parent а также сам parent для нового запуска
              searchStructure(
                lineLink.clone(), 
                parentLink.clone(), 
                &mut lineIndex,
              ); 
            }
            None => {}
          }
        }
        /*
        "ex" =>
        { // exit block up
          println!("ex"); 
        }
        */
        "sleep" =>
        { // sleep
          let valueNumber: u64 =
            expressions[0].getData().unwrap_or_default()
              .parse::<u64>().unwrap_or_default(); // todo: depends on Value.rs
          match valueNumber > 0
          {
            true  => { sleep( Duration::from_millis(valueNumber) ); }
            false => {}
          }
        }
        "exit" =>
        { // Завершает программу с определённым кодом или кодом ошибки;
          unsafe
          {
            _exit = true;
            _exitCode =
              expressions[0]
                .getData().unwrap_or_default()
                .parse::<i32>().unwrap_or(1);
          }
        }
        _ =>
        { // Если не было найдено совпадений среди стандартных процедур,
          // значит это нестандартный метод.
          match self.getStructureByName(&structureName)
          {
            None => {}
            Some(calledStructureLink) =>
            { // После получения такой нестандартной структуры по имени,
              // мы смотрим на её параметры
              {
                let calledStructure: RwLockReadGuard<'_, Structure> = calledStructureLink.read().unwrap();
                for (l, parameter) in expressions.iter().enumerate()
                {
                  match &calledStructure.structures
                  {
                    None => {}
                    Some(calledStructureStructures) =>
                    {
                      let parameterResult: Token = self.expression(&mut vec![parameter.clone()]);
                      match calledStructureStructures.get(l)
                      {
                        None => {}
                        Some(parameterStructure) =>
                        {
                          let mut parameterStructure: RwLockWriteGuard<'_, Structure> = parameterStructure.write().unwrap();
                          // add new structure
                          parameterStructure.lines =
                            vec![
                              Arc::new(
                              RwLock::new(
                                Line {
                                  tokens: vec![parameterResult],
                                  indent: 0,
                                  lines:  None,
                                  parent: None
                                }
                              ))
                            ];
                        }
                      }
                      //
                    }
                  }
                  //
                }
              }
              // Запускаем новую структуру
              readLines(calledStructureLink.clone());
            }
          }
        } // Конец custom метода
      }
      // Всё успешно, это была стандартная процедура
    } // Если название структуры не в нижнем регистре
  }
}
