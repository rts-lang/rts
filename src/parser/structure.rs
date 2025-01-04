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

// вычисляет по математической операции значение и тип нового токена из двух
pub fn calculate(op: &TokenType, leftToken: &Token, rightToken: &Token) -> Token 
{
  // получаем значение левой части выражения
  let leftTokenDataType: TokenType = leftToken.getDataType().unwrap_or_default();
  let leftValue: Value = getValue(leftToken.getData().unwrap_or_default(), &leftTokenDataType);
  // получаем значение правой части выражения
  let rightTokenDataType: TokenType = rightToken.getDataType().unwrap_or_default();
  let rightValue: Value = getValue(rightToken.getData().unwrap_or_default(), &rightTokenDataType);
  // получаем значение выражения, а также предварительный тип
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
  // после того как значение было получено,
  // смотрим какой точно тип выдать новому токену
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
      if leftTokenDataType == TokenType::Float  || rightTokenDataType == TokenType::Float 
      {
        resultType = TokenType::Float;
      } else
      if leftTokenDataType == TokenType::UFloat || rightTokenDataType == TokenType::UFloat 
      {
        resultType = TokenType::UFloat;
      } else
      if leftTokenDataType == TokenType::Int    || rightTokenDataType == TokenType::Int 
      { 
        resultType = TokenType::Int;
      }
    }
    false => {}
  }
  return Token::new( Some(resultType), Some(resultValue) );
}
// зависимость для calculate
// считает значение левой и правой части выражения
fn getValue(tokenData: String, tokenDataType: &TokenType) -> Value 
{
  return
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
    };
}

// get structure result type
// todo: вообще лучше бы это было в самом Token,
//       поскольку там есть перевод уже TokenType -> String
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

#[derive(Clone)]
pub struct Structure 
{
  pub           name: String,                        // unique name
                                                     // todo: Option
  pub          lines: Vec< Arc<RwLock<Line>> >,      // nesting lines
                                                     // todo: Option
  pub     parameters: Option< Vec<Token> >,          // parameters
  pub         result: Option<Token>,                 // result type
      // if result type = None, => procedure
      // else => function
  pub     structures: Option< Vec< Arc<RwLock<Structure>> > >,
  pub         parent: Option< Arc<RwLock<Structure>> >,

  pub      lineIndex: usize,
}
impl Structure 
{
  pub fn new
  (
      name: String,
     lines: Vec< Arc<RwLock<Line>> >,
    parent: Option< Arc<RwLock<Structure>> >,
  ) -> Self 
  {
    Structure 
    {
              name,
             lines,
        parameters: None, // todo: remove
            result: None,
        structures: None,
            parent,
      lineIndex: 0
    }
  }

  // ищет структуру по имени и возвращает либо None, либо ссылку на неё
  pub fn getStructureByName(&self, name: &str) -> Option< Arc<RwLock<Structure>> > 
  {
    match &self.structures 
    {
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
      None => {}
    }
    // check the parent structure if it exists
    match &self.parent 
    {
      Some(parentLink) => 
      {
        parentLink.read().unwrap()
          .getStructureByName(name)
      }
      None => { None }
    } 
  }

  // добавляет новую вложенную структуру в текущую структуру
  pub fn pushStructure(&mut self, structure: Structure) -> ()
  { 
    match self.structures.is_none() 
    {
      true => 
      { // если не было ещё структур,
        // то создаём новый вектор
        self.structures = Some( vec!(Arc::new(RwLock::new(structure))) );
      } 
      false => if let Some(ref mut structures) = self.structures 
      { // если уже есть структуры,
        // то просто push делаем
        structures.push( Arc::new(RwLock::new(structure)) );
      }
    }
  }

  // get structure nesting
  // todo: описание
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
          Some(nestingLine) => 
          {
            nestingLine.write().unwrap()
              .tokens = newTokens;
          }
          None => {}
        }
      }
    }
  }

  // выполняет операцию со структурой,
  // для этого требует левую и правую часть выражения,
  // кроме того, требует передачи родительской структуры,
  // чтобы было видно возможные объявления в ней
  pub fn structureOp(&self, structureLink: Arc<RwLock<Structure>>, op: TokenType, leftValue: Vec<Token>, rightValue: Vec<Token>) -> ()
  {
    match op {
      TokenType::Equals | TokenType::PlusEquals | TokenType::MinusEquals | 
      TokenType::MultiplyEquals | TokenType::DivideEquals => {},
      _ => {return},
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
            true => {
              match value.tokens 
              {
                Some(mut valueTokens) => 
                {
                  structureNesting.push( 
                    self.expression(&mut valueTokens) 
                  );
                }
                None => {}
              }
            }
            false => {}
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

        // Далее обрабатываем саму операцию;
        let mut structure: RwLockWriteGuard<'_, Structure> = structureLink.write().unwrap();
        match op 
        { // Определяем тип операции;
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

  // Вычисляем значение для struct имени типа TokenType::Word 
  fn replaceStructureByName(&self, value: &mut Vec<Token>, index: usize) -> ()
  {
    fn setNone(value: &mut Vec<Token>, index: usize) 
    { // Возвращаем пустое значение
      value[index].setData    (None);
      value[index].setDataType(None);
    }

    match value[index].getData() 
    {
      Some(structureName) => 
      {
        match self.getStructureByName(&structureName) 
        {
          Some(structureLink) => 
          {
            let structure: RwLockReadGuard<'_, Structure> = structureLink.read().unwrap();
            { // Если это просто обращение к имени структуры
              let structureLinesLen: usize = structure.lines.len();
              match structureLinesLen 
              {
                1 =>
                { // структура с одним вложением
                  let tokens: &mut Vec<Token> = &mut structure.lines[0]
                                                  .read().unwrap()
                                                  .tokens.clone();
                  let _ = drop(structure);
                  let result: Token = self.expression(tokens);
                  value[index].setData    ( result.getData().clone() );
                  value[index].setDataType( result.getDataType().clone() );
                } 
                structureLinesLen if structureLinesLen > 1 =>
                { // это структура с вложением
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
          } 
          None => { setNone(value, index); } // Не нашли структуру
        }
      } 
      None => { setNone(value, index); } // Ошибка имени структуры
    }
  }

  /* Получает значение из ссылки на структуру;
     Ссылка на структуру может состоять как из struct name, так и просто из цифр.
  */
  fn linkExpression(&self, currentStructureLink: Option< Arc<RwLock<Structure>> >, link: &mut Vec<String>, parameters: Option< Vec<Token> >) -> Token
  {
    // Обработка динамического выражение
    match link[0].starts_with('[')
    {
      true => 
      { // Получаем динамическое выражение между []
        link[0] = format!("{{{}}}", &link[0][1..link[0].len()-1]);
        // Получаем новую строку значения из обработки выражения
        link[0] = self.formatQuote(link[0].clone());
      }
      false => {}
    }
    // Обработка пути
    match link[0].parse::<usize>() 
    { // проверяем тип
      Ok(lineNumber) => 
      { // если мы нашли цифру в ссылке, значит это номер на линию в структуре;
        // номер ссылкается только на пространство currentStructureLink
        link.remove(0);

        if let Some(ref currentStructureLock) = currentStructureLink 
        { // это структура, которая была передана предыдущем уровнем ссылки;
          // только в ней мы можем найти нужную линию
          let currentStructure: RwLockReadGuard<'_, Structure> = currentStructureLock.read().unwrap(); // todo: это можно вынести в временный блок
          if let Some(line) = currentStructure.lines.get(lineNumber)                                   //       для получения линии и выхода из read().unwrap()
          { // тогда просто берём такую линию по её номеру
            let mut lineTokens: Vec<Token> = 
              {
                line.read().unwrap()
                  .tokens.clone()
              };

            match lineTokens.len() > 0 
            { // Проверяем количество токенов, чтобы понять,
              // можем ли мы вычислить что-то;
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
                    Some(_) =>
                    {
                      let _ = drop(currentStructure);
                      return currentStructureLock.read().unwrap()
                        .linkExpression(None, link, parameters);
                    }
                    None => {}
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
                    true => 
                    { // Если это слово, то это либо ссылка т.к. там много значений в ней;
                      // Либо это структура с одиночным вложением и мы можем его забрать сейчас.

                      match currentStructure.getStructureByName( 
                        &lineTokens[0].getData().unwrap_or_default() 
                      )
                      { // Пробуем проверить что там 1 линия вложена в структуре;
                        // После чего сможем посчитать её значение.
                        Some(childStructureLink) => 
                        {
                          let childStructure: RwLockReadGuard<'_, Structure> = childStructureLink.read().unwrap();
                          match childStructure.lines.len() == 1 
                          {
                            true => 
                            {
                              match childStructure.lines.get(0) 
                              { // По сути это просто 0 линия через expression
                                Some(line) => 
                                { 
                                  let mut lineTokens: Vec<Token> = 
                                    {
                                      line.read().unwrap()
                                        .tokens.clone()
                                    };
                                  let _ = drop(childStructure);
                                  return self.expression(&mut lineTokens);
                                }
                                None => {}
                              }
                            }
                            false => {}
                          }
                        }
                        None => {}
                      }
                      // Если ничего не получилось, значит оставляем ссылку
                      return Token::new( Some(TokenType::Link), lineTokens[0].getData() );
                    }  
                    false => 
                    { // Если это не слово, то смотрим на результат expression
                      return self.expression(&mut lineTokens);
                    }
                  }
                }
              }  
              false => 
              { // В линии нет точенов, нам нечего вычислять
                return Token::newEmpty( Some(TokenType::None) );
              }
            }
          }
          //
        }
      }
      Err(_) => 
      { // если мы не нашли цифры в ссылке, значит это просто struct name;
        // они работают в пространстве первого self, но могут и внутри себя
        let structureLink: Option< Arc<RwLock<Structure>> > =
          match currentStructureLink
          {
            Some(currentStructureLink) => 
            { // если есть в локальном окружении
              let structure: RwLockReadGuard<'_, Structure> = currentStructureLink.read().unwrap();
              let hasLines: bool = 
              {
                let childStructureLink: Option< Arc<RwLock<Structure>> > = structure.getStructureByName(&link[0]);
                match childStructureLink 
                {
                  Some(childStructureLink) => 
                  {
                    match 
                      childStructureLink.read().unwrap()
                        .lines.len() != 0 
                    {
                      true  => { true } 
                      false => { false }
                    }
                  } 
                  None => { false }
                }
              };

              match hasLines
              {
                true  => { structure.getStructureByName(&link[0]) }  
                false => { self.getStructureByName(&link[0]) }
              }
            }  
            None => 
            { // если нет в локальном окружении, 
              // то просто берём из self
              self.getStructureByName(&link[0]) 
            }
          };
        //
        link.remove(0);
        match structureLink
        {
          Some(structureLink) => 
          { // Это структура которую мы нашли по имени в self пространстве
            match link.len() != 0
            { // Закончилась ли ссылка?
              true => 
              { // Если нет, значит продолжаем её чтение
                return self.linkExpression(Some(structureLink), link, parameters);
              }  
              false => 
              { // Если это конец, то берём последнюю структуру и работаем с ней
                let structure: RwLockReadGuard<'_, Structure> = structureLink.read().unwrap();
                match structure.lines.len() == 1 
                {
                  true => 
                  { // Если это просто одиночное значение, то просто выдаём его
                    match structure.lines.get(0) 
                    {
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
                      None => {}
                    }
                  } 
                  false => match parameters 
                  {
                    Some(_) =>
                    { // Если это был просто запуск метода, то запускаем его
                      let mut parametersToken: Token = Token::newNesting( Some(Vec::new()) ); // todo: add parameters
                      parametersToken.setDataType( Some(TokenType::CircleBracketBegin) );

                      let mut expressionTokens: Vec<Token> = vec![
                        Token::new( Some(TokenType::Word), Some(structure.name.clone()) ),
                        parametersToken
                      ];

                      match structure.parent.clone()
                      {
                        Some(structureParent) => 
                        {
                          let _ = drop(structure);
                          return structureParent.read().unwrap()
                            .expression(&mut expressionTokens);
                        }
                        None => {}
                      }

                      return Token::newEmpty( Some(TokenType::None) );
                    }  
                    None => 
                    { // Если это просто ссылка, то оставляем её
                      return Token::new( Some(TokenType::Link), Some(structure.name.clone()) );
                    }
                  }
                  //
                }
              }
              //
            }
          }
          None => {}
        }
        //
      }
    }
    // если всё было плохо, то просто используем пустой результат
    Token::newEmpty( Some(TokenType::None) )
  }

  /* Принимает formatQuote типы и получает возможное значение обычной строки;
     В основном всё сводится к получению токенов в {} через Token::readTokens(),
     после чего результат проходит через expression и мы получаем обычную строку на выходе.
  */
  fn formatQuote(&self, tokenData: String) -> String 
  {
    let mut result:           String    = String::new(); // это строка которая будет получена в конце
    let mut expressionBuffer: String    = String::new(); // буфер для выражения между {}
    let mut expressionRead:   bool      = false;         // флаг чтения в буфер выражения

    let chars:       Vec<char> = tokenData.chars().collect(); // всех символы в строке
    let charsLength: usize     = chars.len();                 // количество всех символов в строке

    let mut i:      usize = 0; // указатель на текущий символ
    let mut c:      char;      // текущий символ

    while i < charsLength 
    { // читаем символы
      c = chars[i];
      match c 
      {
        '{' =>
        { // начинаем чтение выражения
          expressionRead = true;
        }
        '}' =>
        { // заканчиваем чтение выражения
          expressionRead = false;
          expressionBuffer += "\n"; // это нужно чтобы успешно завершить чтение линии Tokenizer::readTokens()

          let mut expressionBufferTokens: Vec<Token> = 
          {
            readTokens(
              expressionBuffer.as_bytes().to_vec(), 
              false
            )[0] // Получаем результат выражения в виде ссылки на буферную линию
              .read().unwrap() // Читаем ссылку и
              .tokens.clone()  // получаем все токены линии
          };
          // отправляем все токены линии как выражение
          match self.expression(&mut expressionBufferTokens).getData() 
          {
            Some(expressionData) => 
            { // записываем результат посчитанный между {}
              result += &expressionData;
            }
            None => {}
          }
          // обнуляем буфер, вдруг далее ещё есть выражения между {}
          expressionBuffer = String::new();
        }
        _ => 
        { // запись символов кроме {}
          match expressionRead 
          {
            true => 
            { // если флаг чтения активен, то записываем символы выражения
              expressionBuffer.push(c);
            }  
            false => 
            { // если флаг чтения не активен, то это просто символы
              result.push(c);
            }
          }
        }
      }
      // продолжаем чтение символов строки
      i += 1;
    }
    // отдаём новую строку
    result
  }

  /* Получает параметры структуры вычисляя их значения;
     todo: требует пересмотра
  */
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

  /* Получает параметры при вызове структуры в качестве метода;
     т.е. получает переданные значения через expression
  */
  fn getCallParameters(&self, value: &mut Vec<Token>, i: usize) -> Option< Vec<Token>  >
  {
    let mut result: Vec<Token> = Vec::new();

    match value.get(i+1).map(|v| &v.tokens) 
    {
      Some(tokens) => 
      {
        match tokens
        {
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
                    true  => { expressionBuffer.push( token.clone() ); }
                    false => {}
                  }
                  result.push( self.expression(&mut expressionBuffer) );
                  expressionBuffer.clear();
                }  
                false => 
                { // push new expression token
                  expressionBuffer.push( token.clone() );
                }
              }
            }
          }
          None => {}
        }
      }
      None => {}
    }

    match result.len() == 0 
    {
      true  => { None }
      false => { Some(result) }
    }
  }

  /* Основная функция, которая получает результат выражения состоящего из токенов;
     Сначала она проверяет что это single токен, но если нет, 
     то в цикле перебирает возможные варианты
  */
  pub fn expression(&self, value: &mut Vec<Token>) -> Token 
  {
    let mut valueLength: usize = value.len(); // получаем количество токенов в выражении
    // todo: Возможно следует объединить с нижним циклом, всё равно проверять токены по очереди
    // 1 токен
    'isSingleToken: 
    { // если это будет не одиночный токен, 
      // то просто выйдем отсюда
      // todo: возможно стоит сразу проверять что тут не Figure, Square, Circle скобки
      match valueLength == 1 
      {
        true => 
        { // если это выражение с 1 токеном, то;
          match value[0].getDataType().unwrap_or_default()
          { // проверяем возможные варианты;
            TokenType::Link =>
            { // если это TokenType::Link, то;
              let data: String = value[0].getData().unwrap_or_default();                // token data
              let mut link: Vec<String> = data.split('.')
                                            .map(|s| s.to_string())
                                            .collect();
              let linkResult: Token  = self.linkExpression(None, &mut link, None);      // получаем результат от data
              let linkType:   TokenType = linkResult.getDataType().unwrap_or_default(); // предполагаем изменение dataType
              match linkType 
              {
                TokenType::Word => 
                { // если это TokenType::Word то теперь это будет TokenType::Link
                  value[0].setDataType( Some(TokenType::Link) );
                }  
                _ => 
                { // если это другие типы, то просто ставим новый dataType
                  value[0].setDataType( linkResult.getDataType() );
                }
              }
              value[0].setData( linkResult.getData() ); // ставим новый data
            }
            TokenType::Word =>
            { // если это TokenType::Word, то;
              let data:       String = value[0].getData().unwrap_or_default();      // token data
              let linkResult: Token  = self.linkExpression(None, &mut vec![data], None); // получаем результат от data
              value[0].setDataType( linkResult.getDataType() );                     // ставим новый dataType
              value[0].setData(     linkResult.getData() );                         // ставим новый data
            }
            TokenType::FormattedRawString | TokenType::FormattedString | TokenType::FormattedChar =>
            { // если это форматные варианты Char, String, RawString;
              match value[0].getData() 
              {
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
                None => {}
              }
            }
            _ => { break 'isSingleToken; } // выходим т.к. все варианты не прошли
          }
          return value[0].clone(); // возвращаем результат в виде одного токена
        }
        false => {}
      }
    }

    // если это выражение не из одного токена,
    // то следует проверять каждый токен в цикле и
    // производить соответствующие операции
    let mut i: usize = 0; // указатель на текущий токен

    while i < valueLength 
    { // проверяем на использование методов,
      // на использование ссылок на структуру,
      // на использование простого выражения в скобках
      match value[i].getDataType().unwrap_or_default() 
      {
        TokenType::FormattedRawString | TokenType::FormattedString | TokenType::FormattedChar =>
        { // если это форматные варианты Char, String, RawString;
          match value[0].getData() 
          {
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
            None => {}
          }
        }
        TokenType::Word =>
        { // это либо метод, либо просто слово-структура
          match i+1 < valueLength && value[i+1].getDataType().unwrap_or_default() == TokenType::CircleBracketBegin 
          {
            true => 
            { // Запускает метод
              self.functionCall(value, &mut valueLength, i);
            }  
            false => 
            { // Вычисляем значение для struct имени типа TokenType::Word 
              self.replaceStructureByName(value, i);
            }
          }
        } 
        TokenType::Link =>
        { // это ссылка на структуру
          let expressions: Option< Vec<Token> > = self.getCallParameters(value, i);
          // todo: здесь надо написать вариант в котором ссылку вызвали с параметрами
          match expressions 
          {
            Some(expressions) => 
            { // если имеются параметры
              //let data: String = value[0].getData().unwrap_or_default();
              //value[0].setDataType( Some(TokenType::String) );
              //value[0].setData(Some(
              //  self.linkExpression(&mut data.split('.').collect(), None)
              //));
            }  
            None => 
            { // без параметров
              let     data: String = value[i].getData().unwrap_or_default();
              let mut link: Vec<String> = data.split('.')
                                            .map(|s| s.to_string())
                                            .collect();
              let linkResult: Token = 
                match i+1 < valueLength && value[i+1].getDataType().unwrap_or_default() == TokenType::CircleBracketBegin 
                {
                  true => 
                  { // если это запуск процедуры
                    self.linkExpression(None, &mut link, Some(vec![]))
                  }  
                  false => 
                  { // если это обычная ссылка
                    self.linkExpression(None, &mut link, None)
                  }
                };
              value[i].setDataType( linkResult.getDataType() );
              value[i].setData(     linkResult.getData() );
            }
          }
        } 
        TokenType::Minus =>
        { // это выражение в круглых скобках, но перед ними отрицание -
          match i+1 < valueLength && value[i+1].getDataType().unwrap_or_default() == TokenType::CircleBracketBegin 
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
              // удаляем скобки
              value.remove(i+1); // remove UInt
              valueLength -= 1;
              // меняем отрицание
              let tokenData: String = value[i].getData().unwrap_or_default();
              match tokenData.starts_with(|c: char| c == '-') 
              {
                true => 
                { // если это было отрицательное выражение,
                  // то делаем его положительным
                  value[i].setData( 
                    Some( tokenData.chars().skip(1).collect() ) 
                  );
                }  
                false => 
                { // если это не было отрицательным выражением,
                  // то делаем его отрицательным
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
        { // это просто выражение в круглых скобках
          value[i] = 
            match value[i].tokens.clone() 
            {
              Some(mut tokenTokens) => 
              { // если получилось то оставляем его
                self.expression(&mut tokenTokens)
              } 
              None => 
              { // если не получилось, то просто None
                Token::newEmpty(None)
              }
            }
        }
        _ => {}
      }
      i += 1;
    }

    // далее идут варианты математических и логических операций

    // проверка на логические операции 1
    self.expressionOp(value, &mut valueLength, 
      &[TokenType::Equals, TokenType::NotEquals, 
        TokenType::GreaterThan, TokenType::LessThan,
        TokenType::GreaterThanOrEquals, TokenType::LessThanOrEquals]
    );

    // проверка на логические операции 2
    self.expressionOp(value, &mut valueLength, 
      &[TokenType::Inclusion, TokenType::Joint]
    );
    
    // проверка * и /
    self.expressionOp(value, &mut valueLength, &[TokenType::Multiply, TokenType::Divide]);
    
    // проверка + и -
    self.expressionOp(value, &mut valueLength, &[TokenType::Plus, TokenType::Minus]);

    // конец чтения выражения
    match valueLength != 0 
    {
      true => 
      { // в том случае, если мы имеем всё ещё значение,
        // значит просто вернём 0 элемент, чтобы избавиться от него
        value[0].clone()
      }  
      false => 
      { // а если всё пусто, ну значит пусто
        Token::newEmpty(None)
      }
    }
  }

  /* Получает значение операции по левому и правому выражению;
     Это зависимость для expression.
     Кроме того, может обрабатываеть отрицание при использовании TokenType::Minus
  */
  fn expressionOp(&self, value: &mut Vec<Token>, valueLength: &mut usize, operations: &[TokenType]) 
  {
    let mut i: usize = 0;
    let mut token: Token;
    let mut tokenType: TokenType;

    while i < *valueLength 
    { // проверка на логические операции
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
      match i+1 < *valueLength && matches!(tokenType, ref operations) 
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
        false => if matches!(TokenType::Minus, ref operations) && matches!(tokenType, TokenType::Int | TokenType::Float) 
        {
          value[i-1] = calculate(&TokenType::Plus, &value[i-1], &value[i]);

          value.remove(i); // remove UInt
          *valueLength -= 1;
          continue;
        }
      }

      i += 1;
    }
  }

  /* Запускает функцию;
     Функция - это такая структура, которая возвращает значение.

     Но кроме того, запускает не стандартные методы; 
     В нестандартных методах могут быть процедуры, которые не вернут результат.

     todo: вынести все стандартные варианты в отдельный модуль
  */
  pub fn functionCall(&self, value: &mut Vec<Token>, valueLength: &mut usize, i: usize) -> ()
  {
    // todo: uint float ufloat ...
    match value[i].getData() // todo: проверка на нижний регистр
    {
      Some(structureName) => 
      { // 
        let expressions: Option< Vec<Token> > = self.getCallParameters(value, i);
        // далее идут базовые методы;
        // эти методы ожидают аргументов
        'basicMethods: 
        { // это позволит выйти, если мы ожидаем не стандартные варианты
          match expressions
          {
            Some(ref expressions) => 
            { // далее просто сверяем имя структуры в поисках базовой
              match structureName.as_str() 
              { // проверяем на сходство стандартных функций
                "UInt" =>
                { // получаем значение выражения в типе
                  // todo: Float, UFloat
                  value[i].setDataType( Some(TokenType::UInt ) );
                  value[i].setData    ( Some(expressions[0].getData().unwrap_or_default()) );
                } 
                "Int" =>
                { // получаем значение выражения в типе
                  value[i].setDataType( Some(TokenType::Int ) );
                  value[i].setData    ( Some(expressions[0].getData().unwrap_or_default()) );
                } 
                "String" =>
                { // получаем значение выражение в типе String
                  // todo: подумать над formatted типами
                  value[i].setDataType( Some(TokenType::String ) );
                  value[i].setData    ( Some(expressions[0].getData().unwrap_or_default()) );
                } 
                "Char" =>
                { // получаем значение выражения в типе Char
                  // todo: проверить работу
                  value[i].setDataType( Some(TokenType::Char) );
                  value[i].setData( 
                    Some(
                      (expressions[0].getData().unwrap_or_default()
                          .parse::<u8>().unwrap() as char
                      ).to_string()
                    ) 
                  );
                } 
                "type" =>
                { // todo: создать resultType() ?
                  // для возвращения результата ожидаемого структурой
                  value[i].setDataType( Some(TokenType::String) );
                  value[i].setData    ( Some(expressions[0].getDataType().unwrap_or_default().to_string()) );
                } 
                "randUInt" if expressions.len() > 1 =>
                { // возвращаем случайное число типа UInt от min до max
                  let min: usize = 
                    match expressions[0].getData() 
                    {
                      Some(expressionData) => { expressionData.parse::<usize>().unwrap_or_default() }
                      None => { 0 }
                    };
                  let max: usize = 
                    match expressions[1].getData() 
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
                  match expressions[0].getDataType().unwrap_or_default() 
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
                          expressions[0].getData().unwrap_or_default()
                            .chars().count().to_string()
                        ) 
                      );
                    }
                    _ => 
                    { // Получаем размер вложений в структуре
                      // Результат только в UInt
                      value[i].setDataType( Some(TokenType::UInt) );
                      // Получаем значение
                      match self.getStructureByName(&expressions[0].getData().unwrap_or_default()) 
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
                { // получаем результат ввода

                  // результат может быть только String
                  value[i].setDataType( Some(TokenType::String) );

                  match expressions[0].getData() 
                  {
                    Some(expressionData) => 
                    { // это может быть выведено перед вводом;
                      // todo: возможно потом это лучше убрать,
                      //       т.к. программист сам может вызвать 
                      //       такое через иные методы
                      print!("{}",expressionData);
                      io::stdout().flush().unwrap(); // forced withdrawal of old
                    }
                    None => {}
                  }

                  let mut valueBuffer: String = String::new(); // временный буффер ввода
                  match io::stdin().read_line(&mut valueBuffer) 
                  { // читаем ввод
                    Ok(_) => 
                    { // успешно ввели и записали
                      value[i].setData( 
                        Some( valueBuffer.trim_end().to_string() )
                      );
                    }
                    Err(_) =>
                    { // не удалось ввести, пустая строка
                      value[i].setData( Some(String::new()) );
                    }
                  }
                } 
                "exec" => 
                { // Запускает что-то и возвращает строковый output работы
                  let data: String = expressions[0].getData().unwrap_or_default();
                  let mut parts: SplitWhitespace<'_> = data.split_whitespace();

                  let command: &str      = parts.next().expect("No command found in expression"); // todo: no errors
                  let    args: Vec<&str> = parts.collect();

                  let output: Output = 
                    Command::new(command)
                      .args(&args)
                      .output()
                      .expect("Failed to execute process"); // todo: no errors
                  let outputString: String = String::from_utf8_lossy(&output.stdout).to_string();
                  match !outputString.is_empty() 
                  {
                    true => 
                    { // result
                      value[i].setData    ( Some(outputString.trim_end().to_string()) );
                      value[i].setDataType( Some(TokenType::String) );
                    }
                    false => {}
                  }
                }
                "execs" => 
                { // Запускает что-то и возвращает кодовый результат работы
                  // todo: Возможно изменение: Следует ли оставлять вывод stdout & stderr ?
                  //       -> Возможно следует сделать отдельные методы для подобных операций.
                  let data: String = expressions[0].getData().unwrap_or_default();
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
              // если всё было успешно, то сдвигаем всё до 1 токена;
              // этот токен останется с полученным значением
              *valueLength -= 1;
              value.remove(i+1);
              return;
            }
            None => {}
          }
        }
        // если код не завершился ранее, то далее идут custom методы;
        { // передаём параметры, они также могут быть None
          self.procedureCall(&structureName, expressions);
          // если всё было успешно, то сдвигаем всё до 1 токена;
          *valueLength -= 1;
          value.remove(i+1);
          // после чего решаем какой результат оставить
          match self.getStructureByName(&structureName) 
          {
            Some(structureLink) => 
            { // по результату структуры, определяем пустой он или нет
              match 
                &structureLink.read().unwrap()
                  .result 
              {
                Some(result) => 
                { // результат не пустой, значит оставляем его
                  value[i].setData    ( result.getData() );
                  value[i].setDataType( result.getDataType().clone() );
                }  
                None => 
                { // если результата структуры не было, 
                  // значит это была действительно процедура
                  value[i].setData    ( None );
                  value[i].setDataType( None );
                }
              }
            }
            None => {}
          }
        }
        // заканчиваем чтение методов
      }
      None => {}
    }
  }

  /* Запускает стандартные процедуры; 
     Процедура - это такая структура, которая не возвращает результат.

     Но кроме того, запускает не стандартные методы; 
     Из нестандартных методов, процедуры могут вернуть результат, в таком случае, их следует считать функциями.

     todo: вынести все стандартные варианты в отдельный модуль
  */
  pub fn procedureCall(&self, structureName: &str, expressions: Option< Vec<Token> >) -> ()
  { 
    if structureName.starts_with(|c: char| c.is_lowercase()) 
    { // если название в нижнем регистре - то это точно процедура
      match structureName 
      { // проверяем на сходство стандартных функций
        "println" =>
        { // println
          match expressions 
          {
            Some(expressions) => 
            { // todo: вывод всех expressions
              formatPrint( &format!("{}\n",&expressions[0].getData().unwrap_or_default()) );
            } 
            None => 
            { // в том случае, если мы не получили выводимое выражение
              println!();
            }
          }
          io::stdout().flush().unwrap(); // forced withdrawal of old
        }
        "print" =>
        { // print
          match expressions 
          {
            Some(expressions) => 
            { // todo: вывод всех expressions
              formatPrint( &expressions[0].getData().unwrap_or_default() );
            }  
            None => 
            { // в том случае, если мы не получили выводимое выражение
              print!("");
            }
          }
          io::stdout().flush().unwrap(); // forced withdrawal of old
        }
        "clear" =>
        { // clear
          let _ = Command::new("clear")
            .status(); // игнорируем ошибки
          // todo: однако можно выдавать результат boolean при ошибке
        }
        "go" =>
        { // запускаем линию выше заново
          match &self.parent 
          {
            Some(parentLink) => 
            {
              let (mut lineIndex, lineLink): (usize, Arc<RwLock<Line>>) = 
              { // это более безопасный вариант, чтобы использование parent закончилось
                // перед дальнейшим использованием ссылки на него
                let parent: RwLockReadGuard<'_, Structure> = parentLink.read().unwrap();
                let lineIndexBuffer: usize = parent.lineIndex;

                // Получаем ссылку на линию
                (lineIndexBuffer, parent.lines[lineIndexBuffer].clone())
              };
              // используем линию parent а также сам parent для нового запуска
              searchStructure(
                lineLink.clone(), 
                parentLink.clone(), 
                &mut lineIndex,
                false
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
          match expressions 
          {
            Some(expressions) => 
            { // expression value
              let valueNumber: u64 = 
                expressions[0].getData().unwrap_or_default()
                  .parse::<u64>().unwrap_or_default(); // todo: depends on Value.rs
              match valueNumber > 0 
              {
                true  => { sleep( Duration::from_millis(valueNumber) ); }
                false => {}
              }
            }
            None => {} // если не было параметров, то просто пропускаем
          }
        }
        "exit" =>
        { // Завершает программу с определённым кодом или кодом ошибки;
          unsafe{ _exit = true; } // В любом случае мы завершаем программу
          match expressions 
          {
            Some(expressions) => 
            {
              unsafe
              { // Либо это ожидаемый в параметрах код завершения;
                _exitCode = 
                  expressions[0]
                    .getData().unwrap_or_default()
                    .parse::<i32>().unwrap_or(1);
              }
            }
            None => 
            {
              unsafe
              { // Либо это возврат кода ошибки;
                _exitCode = 1; 
              }
            }
          }
        }
        _ =>
        { // если не было найдено совпадений среди стандартных процедур,
          // значит это нестандартный метод.
          match self.getStructureByName(&structureName) 
          {
            Some(calledStructureLink) => 
            { // после получения такой нестандартной структуры по имени, 
              // мы смотрим на её параметры
              match expressions 
              {
                Some(expressions) => 
                {
                  let calledStructure: RwLockWriteGuard<'_, Structure> = calledStructureLink.write().unwrap();
                  for (l, parameter) in expressions.iter().enumerate() 
                  {
                    match &calledStructure.structures
                    {
                      Some(calledStructureStructures) => 
                      {
                        let parameterResult: Token = self.expression(&mut vec![parameter.clone()]);
                        match calledStructureStructures.get(l) 
                        {
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
                          None => {}
                        }
                        // 
                      }
                      None => {}
                    }
                    //
                  }
                }
                None => {}
              }
              // запускаем новую структуру
              readLines(calledStructureLink, false);
            }
            None => {}
          }
        } // конец custom метода
      }
      // всё успешно, это была стандартная процедура
    } // если название структуры не в нижнем регистре
  }
}
