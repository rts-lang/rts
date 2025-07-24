/* /tokenizer/token
  Token is the smallest unit of data, represents strings, numbers, operators...
*/

use std::fmt;
use crate::parser::structure::StructureType;
use crate::parser::bytes::Bytes;
use crate::tokenizer::line::Line;

// TokenType =======================================================================================
/// Тип элементарной единицы хранения информации
#[derive(PartialEq)]
#[derive(Clone)]
pub enum TokenType
{
// basic
  /// Пустота
  None,
  /// Любой тип данных
  Any,
  /// Обычная связка букв
  Word,
  /// ; или \n
  Endline,
  /// ,
  Comma,
  /// .
  Dot,

  /// \#
  Comment,

// quotes
  /// `
  RawString,
  /// "
  String,
  /// '
  Char,
  /// f``
  FormattedRawString,
  /// f""
  FormattedString,
  /// f''
  FormattedChar,

// single math
  /// +
  Plus,
  /// -
  Minus,
  /// *
  Multiply,
  /// /
  Divide,
  /// =
  Equals,
  /// %
  Modulo,
  /// ^
  Exponent,

// double math
  /// ++
  UnaryPlus,
  /// +=
  PlusEquals,

  /// --
  UnaryMinus,
  /// -=
  MinusEquals,

  /// **
  UnaryMultiply,
  /// *=
  MultiplyEquals,

  /// //
  UnaryDivide,
  /// /=
  DivideEquals,

  /// %%
  UnaryModulo,
  /// %=
  ModuloEquals,

  /// ^^
  UnaryExponent,
  /// ^=
  ExponentEquals,

// single logical
  /// >
  GreaterThan,
  /// <
  LessThan,
  /// ?
  Question,
  /// !
  Not,

// double logical
  /// >=
  GreaterThanOrEquals,
  /// <=
  LessThanOrEquals,
  /// !=
  NotEquals,

// brackets
  /// (
  CircleBracketBegin,
  /// )
  CircleBracketEnd,
  /// [
  SquareBracketBegin,
  /// ]
  SquareBracketEnd,
  /// {
  FigureBracketBegin,
  /// }
  FigureBracketEnd,

// other
  /// :
  Colon,
  /// ->
  Pointer,

  // ~
  Tilde,
  /// ~~
  DoubleTilde,

  /// Ссылка на структуру
  Link,
  
  /// Что-то нативное
  Native,

// words
  /// Integer
  Int,
  /// Unsigned integer
  UInt,
  /// Float
  Float,
  /// Unsigned float
  UFloat,
  /// Rational
  Rational,
  /// Complex
  Complex,

  /// Bool
  Bool,
  /// & (and) Joint
  Joint,
  /// ^
  Disjoint,
  /// | (or)
  Inclusion,
  /// ! (not)
  Exclusion,
  // todo здесь должна быть троичная логика
}

impl ToString for TokenType
{ // todo convert -> fmt::Display ?
  fn to_string(&self) -> String 
  {
    match self 
    {
      // basic
      TokenType::None    => String::from("None"),
      TokenType::Any    => String::from("Any"),
      TokenType::Word    => String::from("Word"),
      TokenType::Endline => String::from("\\n"),
      TokenType::Comma   => String::from(","),
      TokenType::Dot     => String::from("."),

      TokenType::Comment => String::from("Comment"),
      
      // quotes
      TokenType::RawString          => String::from("RawString"),
      TokenType::String             => String::from("String"),
      TokenType::Char               => String::from("Char"),
      TokenType::FormattedRawString => String::from("FormattedRawString"),
      TokenType::FormattedString    => String::from("FormattedString"),
      TokenType::FormattedChar      => String::from("FormattedChar"),
     
      // single math
      TokenType::Plus     => String::from("+"),
      TokenType::Minus    => String::from("-"),
      TokenType::Multiply => String::from("*"),
      TokenType::Divide   => String::from("/"),
      TokenType::Equals   => String::from("="),
      TokenType::Modulo   => String::from("%"),
      TokenType::Exponent => String::from("^"),
      
      // double math
      TokenType::UnaryPlus      => String::from("++"),
      TokenType::PlusEquals     => String::from("+="),

      TokenType::UnaryMinus     => String::from("--"),
      TokenType::MinusEquals    => String::from("-="),

      TokenType::UnaryMultiply  => String::from("**"),
      TokenType::MultiplyEquals => String::from("*="),

      TokenType::UnaryDivide    => String::from("//"),
      TokenType::DivideEquals   => String::from("/="),

      TokenType::UnaryModulo    => String::from("%%"),
      TokenType::ModuloEquals   => String::from("%="),

      TokenType::UnaryExponent  => String::from("^^"),
      TokenType::ExponentEquals => String::from("^="),

      // single logical
      TokenType::GreaterThan => String::from(">"),
      TokenType::LessThan    => String::from("<"),
      TokenType::Question    => String::from("?"),
      TokenType::Not         => String::from("!"),
      
      // double logical
      TokenType::GreaterThanOrEquals => String::from(">="),
      TokenType::LessThanOrEquals    => String::from("<="),
      TokenType::NotEquals           => String::from("!="),
      
      // brackets
      TokenType::CircleBracketBegin => String::from("("),
      TokenType::CircleBracketEnd   => String::from(")"),
      TokenType::SquareBracketBegin => String::from("["),
      TokenType::SquareBracketEnd   => String::from("]"),
      TokenType::FigureBracketBegin => String::from("{"),
      TokenType::FigureBracketEnd   => String::from("}"),
      
      // other
      TokenType::Colon   => String::from(":"),
      TokenType::Pointer => String::from("->"),

      TokenType::Tilde       => String::from("~"),
      TokenType::DoubleTilde => String::from("~~"),

      TokenType::Link => String::from("Link"),

      TokenType::Native => String::from("Native"),
      
      // words
      TokenType::Int      => String::from("Int"),
      TokenType::UInt     => String::from("UInt"),
      TokenType::Float    => String::from("Float"),
      TokenType::UFloat   => String::from("UFloat"),
      TokenType::Rational => String::from("Rational"),
      TokenType::Complex  => String::from("Complex"),

      TokenType::Bool      => String::from("Bool"),
      TokenType::Joint     => String::from("Joint"),
      TokenType::Disjoint  => String::from("Disjoint"),
      TokenType::Inclusion => String::from("Inclusion"),
      TokenType::Exclusion => String::from("Exclusion")
    }
  }
}
impl Default for TokenType
{
  fn default() -> Self 
  {
    TokenType::None
  }
}

pub trait ToStructureType
{
  /// Преобразует TokenType в StructureType
  fn toStructureType(&self) -> StructureType;
}
impl ToStructureType for TokenType
{
  fn toStructureType(&self) -> StructureType
  {
    match self
    {
      TokenType::UInt => StructureType::UInt,
      TokenType::Int => StructureType::Int,
      TokenType::UFloat => StructureType::UFloat,
      TokenType::Float => StructureType::Float,
      TokenType::String => StructureType::String,
      TokenType::Char => StructureType::Char,
      // TokenType::Rational => StructureType::Rational,
      // TokenType::Complex => StructureType::Complex,
      _ =>
      { // todo: возможно нестабильно
        StructureType::Custom(self.to_string())
      }
    }
  }
}
// Token ===========================================================================================
/// Элементарная единица хранения информации
#[derive(Clone)]
pub struct Token 
{
  /// Данные единицы хранения
  data:       Bytes,
  /// Тип данных единицы хранения
  dataType:   TokenType,
  /// Набор вложенных единиц хранения
  pub lines: Option< Vec<Line> >,
}
impl Token 
{
  /// Обычное создание
  pub fn new<T: Into<Bytes>>(
    dataType: TokenType,
    data:     T,
  ) -> Self {
    Token {
      data: data.into(),
      dataType,
      lines: None,
    }
  }
  
  /// Пустой, но имеет тип данных
  pub fn newEmpty(
    dataType: TokenType
  ) -> Self 
  {
    Token 
    {
      data: Bytes::empty(),
      dataType,
      lines: None,
    }
  }
  /// Пустой, но выполняет роль держателя вложения
  pub fn newNesting(
    lines: Vec<Line>
  ) -> Self
  {
    Token
    {
      data: Bytes::empty(),
      dataType: TokenType::None,
      lines: Some(lines),
    }
  }

  // convert data
  // todo: фиг его знает что это за ерунда,
  // но смысл такой, что если тип был Int или Float, 
  // а ожидается UInt или UFloat, то понятно,
  // что результат будет 0
  fn convertData(&mut self) -> ()
  {
    match self.data.toString()
    {
      None => {}
      Some(data) => 
      {
        match data.chars().nth(0)  
        {
          Some('-') => 
          {
            match self.dataType.clone()
            {
              TokenType::UInt   => { self.data = Bytes::from(String::from("0")); }
              TokenType::UFloat => { self.data = Bytes::from(String::from("0.0")); } // todo: use . (0.0)
              _ => { }
            }
          }
          _ => {}
        }
      }
    }
  }

  /// Получает тип данных
  pub fn getDataType(&self) -> &TokenType
  {
    &self.dataType
  }
  /// Устанавливает тип данных
  pub fn setDataType(&mut self, newDataType: TokenType) -> ()
  {
    self.dataType = newDataType;
    self.convertData();
  }

  /// Проверяет примитивный это токен или нет
  pub fn isPrimitive(&self) -> bool
  {
    matches!(
      self.dataType,
      TokenType::None |
      TokenType::Any |
      TokenType::Link |
      TokenType::UInt |
      TokenType::Int |
      TokenType::UFloat |
      TokenType::Float |
      TokenType::Char |
      TokenType::String |
      TokenType::RawString |
      TokenType::FormattedChar |
      TokenType::FormattedString |
      TokenType::FormattedRawString
    )
  }

  /// Получает данные
  pub fn getData(&self) -> Bytes
  {
    self.data.clone()
  }
  /// Устанавливает данные
  pub fn setData<T: Into<Bytes>>(&mut self, newData: T) 
  {
    self.data = newData.into();
    self.convertData();
  }
}

impl fmt::Display for Token
{ // todo: debug only ?
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    match self.data.getAll()
    {
      Some(data) =>
      { // Есть данные - печатаем как символы
        write!(f, "{}", std::str::from_utf8(&data).unwrap_or_default())
      }
      None =>
      { // Данных нет - печатаем тип
        write!(f, "{}", self.getDataType().to_string())
      }
    }
  }
}

impl fmt::Debug for Token
{ // todo: debug only ?
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
  {
    match self.data.getAll()
    {
      Some(data) =>
      { // Есть данные - печатаем как символы
        write!(f, "{}", std::str::from_utf8(&data).unwrap_or_default())
      }
      None =>
      { // Данных нет - печатаем тип
        write!(f, "{}", self.getDataType().to_string())
      }
    }
  }
}
