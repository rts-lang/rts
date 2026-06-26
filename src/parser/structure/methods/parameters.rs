use crate::parser::bytes::Bytes;
use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::StructureType;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Хранит параметры из токенов без просчитывания их заранее
#[derive(Clone)]
pub struct Parameters
{
  values: Option< Vec<Line> >,
}

impl Parameters
{
  // ===============================================================================================
  
  /// Создает новую структуру
  pub fn new(values: Option< Vec<Line> >) -> Self
  {
    Self { values }
  }

  /// Проверяет, есть ли значения
  pub fn isNone(&self) -> bool {
    self.values.is_none()
  }

  // ===============================================================================================

  /// Получает параметр по индексу, если он существует
  pub fn get(&self, index: usize) -> Option<&Line>
  {
    self.values.as_ref()?.get(index)
  }

  /// Возвращает все параметры, если они есть
  pub fn getAll(&self) -> Option< &Vec<Line> >
  {
    self.values.as_ref()
  }

  // ===============================================================================================
  
  /// Получает параметр по индексу, если он существует и
  /// вычисляет его значение его выражения
  pub fn getExpression(&self, structure: &Structure, index: usize) -> Option<Token>
  {
    match self.get(index)
    {
      None => None, // Элемента не было
      Some(line) =>
      { // Возвращаем результат выражения
        match &line.tokens // todo Может быть не 0
        {
          None =>
          { // Один токен
          //  Some(structure.expression(
          //    &mut vec![token.clone()]
          //  )) // Клонируется, поскольку может использоваться многократно
            None // todo По идее здесь только None, т.к. ветка пустая ?
          }
          Some(tokens) =>
          { // Выражение из токенов
            Some(structure.expression(
              &mut tokens.clone() // Клонируется, поскольку может использоваться многократно
            ))
          }
          //
        }
      }
      //
    }
    
  }
  
  /// Возвращает все параметры, если они есть и
  /// вычисляет для них значения их выражений
  pub fn getAllExpressions(&self, structure: &Structure) -> Option< Vec<Token> >
  {
    let mut tokens: Vec<Token> = Vec::new();

    for index in 0..self.getAll()?.len()
    {
      match self.getExpression(structure, index)
      {
        None => {} // Если элемент отсутствует, то просто идём дальше
        Some(token) => tokens.push(token.clone()), // Добавляем результаты
      }
    }
    
    Some(tokens)  // Возвращаем все токены
  }

  // ===============================================================================================
}

// =================================================================================================

impl Structure 
{
  /// Получает параметры структуры вычисляя их значения
  pub fn getStructureParameters(&self, value: &mut Vec<Token>) -> Vec<(Bytes, StructureType)> 
  {
    let mut result: Vec<(Bytes, StructureType)> = Vec::new();
    
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
          
          // todo Тут еще надо определять structure mutable
          
          // Это типизация параметра
          if expressionBuffer.len() == 3 
          {
            let parameterType: StructureType = expressionBuffer[2].getStructureTypeSimple();
            result.push((
              expressionBuffer[0].getData(),
              parameterType
            ));
          } else {
            result.push((
              expressionBuffer[0].getData(),
              StructureType::Any
            ));
          }
          
          //
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
}

// =================================================================================================