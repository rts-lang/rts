/* /tokenizer/token
  Token is the smallest unit of data, represents strings, numbers, operators...
*/

use std::fmt;

// TokenType =======================================================================================
/// Тип элементарной единицы хранения информации
#[derive(PartialEq)]
#[derive(Clone)]
pub enum TokenType
{
// basic
  /// Пустота
  None,
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

// Token ===========================================================================================
/// Элементарная единица хранения информации
#[derive(Clone)]
pub struct Token 
{
  /// Данные единицы хранения
  data:       Option< String >,
  /// Тип данных единицы хранения
  dataType:   Option< TokenType >,
  /// Набор вложенных единиц хранения
  pub tokens: Option< Vec<Token> >,
}
impl Token 
{
  /// Обычное создание
  pub fn new(
    dataType: Option< TokenType >,
    data:     Option< String >
  ) -> Self
  {
    Token
    {
      data,
      dataType,
      tokens: None,
    }
  }
  /// Пустой, но имеет тип данных
  pub fn newEmpty(
    dataType: Option< TokenType >
  ) -> Self 
  {
    Token 
    {
      data: None,
      dataType,
      tokens: None,
    }
  }
  /// Пустой, но выполняет роль держателя вложения
  pub fn newNesting(
    tokens: Option< Vec<Token> >
  ) -> Self 
  {
    Token 
    {
      data: None,
      dataType: None,
      tokens,
    }
  }

  // convert data
  fn convertData(&mut self) -> ()
  { // todo: фиг его знает что это за ерунда,
    // но смысл такой, что если тип был Int или Float, 
    // а ожидается UInt или UFloat, то понятно,
    // что результат будет 0
    match self.data 
    {
      Some(ref mut data) => 
      {
        match data.chars().nth(0)  
        {
          Some('-') => 
          {
            match self.dataType.clone().unwrap_or_default()
            {
              TokenType::UInt   => { *data = String::from("0"); }
              TokenType::UFloat => { *data = String::from("0.0"); } // todo: use . (0.0)
              _ => {}
            }
          }
          _ => {}
        }
      }
      None => {}
    }
  }

  /// Получает тип данных
  pub fn getDataType(&self) -> Option< TokenType >
  {
    self.dataType.clone()
  }
  /// Устанавливает тип данных
  pub fn setDataType(&mut self, newDataType: Option< TokenType >) -> ()
  {
    self.dataType = newDataType;
    self.convertData();
  }

  /// Получает данные
  pub fn getData(&self) -> Option< String >
  {
    self.data.clone()
  }
  /// Устанавливает данные
  pub fn setData(&mut self, newData: Option< String >) -> ()
  {
    self.data = newData;
    self.convertData();
  }
}
impl fmt::Display for Token 
{ // todo: debug only ?
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
  {
    match &self.data 
    {
      Some(data) =>
      {
        write!(f, "{}", data)
      }
      None =>
      {
        write!(f, "{}", self.getDataType().unwrap_or_default().to_string())
      }
    }
  }
}
impl fmt::Debug for Token 
{ // todo: debug only ?
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
  {
    match &self.data 
    {
      Some(data) =>
      {
        write!(f, "{}", data)
      }
      None =>
      {
        write!(f, "{}", self.getDataType().unwrap_or_default().to_string())
      }
    }
  }
}