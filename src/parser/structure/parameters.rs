/* /parser/structure/parameters
  Хранит параметры без просчитывания их заранее
*/
use crate::parser::structure::Structure;
use crate::tokenizer::token::Token;

#[derive(Clone, Debug)]
pub struct Parameters
{
  values: Option< Vec<Token> >,
}

impl Parameters
{
  /// Создает новую структуру
  pub fn new(values: Option< Vec<Token> >) -> Self
  {
    Self { values }
  }

  /// Проверяет, есть ли значения
  pub fn isNone(&self) -> bool {
    self.values.is_none()
  }

  /// Получает параметр по индексу, если он существует
  pub fn get(&self, index: usize) -> Option<&Token>
  {
    self.values.as_ref()?.get(index)
  }

  /// Получает параметр по индексу, если он существует и
  /// вычисляет его значение его выражения
  pub fn getExpression(&self, structure: &Structure, index: usize) -> Option<Token>
  {
    /* todo dataMutability
    match self.get(index)
    {
      None => None, // Элемента не было
      Some(token) =>
      { // Возвращаем результат выражения
        match &token.tokens
        {
          None =>
          { // Один токен
            Some(structure.expression(
              &mut vec![token.clone()]
            )) // Клонируется, поскольку может использоваться многократно
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
    */
    return None;
  }

  /// Возвращает все параметры, если они есть
  pub fn getAll(&self) -> Option< &Vec<Token> >
  {
    self.values.as_ref()
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
}
