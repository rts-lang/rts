/* /parser/structure/parameters
  Хранит параметры без просчитывания их заранее
*/
use crate::parser::structure::Structure;
use crate::tokenizer::line::Line;
use crate::tokenizer::token::Token;

#[derive(Clone)]
pub struct Parameters
{
  values: Option< Vec<Line> >,
}

impl Parameters
{
  /// Создает новую структуру
  pub fn new(values: Option< Vec<Line> >) -> Self
  {
    Self { values }
  }

  /// Проверяет, есть ли значения
  pub fn isNone(&self) -> bool {
    self.values.is_none()
  }

  /// Получает параметр по индексу, если он существует
  pub fn get(&self, index: usize) -> Option<&Line>
  {
    self.values.as_ref()?.get(index)
  }

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

  /// Возвращает все параметры, если они есть
  pub fn getAll(&self) -> Option< &Vec<Line> >
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
