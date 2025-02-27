/* /parser/structure/parameters
  Хранит параметры без просчитывания их заранее
*/

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

  /// Возвращает все параметры, если они есть
  pub fn getAll(&self) -> Option<&Vec<Token>>
  {
    self.values.as_ref()
  }
}
