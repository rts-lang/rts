/* /tokenizer/token
  Token is the smallest unit of data, represents strings, numbers, operators...
*/

use std::fmt;

#[derive(PartialEq)]
#[derive(Clone)]
pub enum TokenType 
{
// basic
  None,    // None
  Word,    // Word
  Endline, // Endline
  Comma,   // ,
  Dot,     // .

  Comment, // #
// quotes
  RawString,          // `
  String,             // "
  Char,               // '
  FormattedRawString, // f``
  FormattedString,    // f""
  FormattedChar,      // f''
// single math
  Plus,     // +
  Minus,    // -
  Multiply, // *
  Divide,   // /
  Equals,   // =
  Modulo,   // %
  Exponent, // ^
// double math
  UnaryPlus,      // ++
  PlusEquals,     // +=

  UnaryMinus,     // --
  MinusEquals,    // -=

  UnaryMultiply,  // **
  MultiplyEquals, // *=

  UnaryDivide,    // //
  DivideEquals,   // /=

  UnaryModulo,    // %%
  ModuloEquals,   // %=

  UnaryExponent,  // ^^
  ExponentEquals, // ^=
// single logical
  GreaterThan, // >
  LessThan,    // <
  Question,    // ?
  Not,         // !
// double logical
  GreaterThanOrEquals, // >=
  LessThanOrEquals,    // <=
  NotEquals,           // !=
// brackets
  CircleBracketBegin, // (
  CircleBracketEnd,   // )
  SquareBracketBegin, // [
  SquareBracketEnd,   // ]
  FigureBracketBegin, // {
  FigureBracketEnd,   // }
// other
  Colon,   // :
  Pointer, // ->

  Tilde,       // ~
  DoubleTilde, // ~~

  Link, // Link
// words
  Int,      // Integer
  UInt,     // Unsigned integer
  Float,    // Float
  UFloat,   // Unsigned float
  Rational, // Rational
  Complex,  // Complex

  Bool,      // Bool
  Joint,     // & (and) Joint
  Disjoint,  // ^
  Inclusion, // | (or)
  Exclusion, // ! (not)

  Custom(String),
}

impl ToString for TokenType 
{
  fn to_string(&self) -> String 
  {
    match self 
    {
      // basic
      TokenType::None    => String::from("None"),    // None
      TokenType::Word    => String::from("Word"),    // Word
      TokenType::Endline => String::from("\\n"),     // Endline
      TokenType::Comma   => String::from(","),       // ,
      TokenType::Dot     => String::from("."),       // .

      TokenType::Comment => String::from("Comment"), // #
      
      // quotes
      TokenType::RawString          => String::from("RawString"),          // `
      TokenType::String             => String::from("String"),             // "
      TokenType::Char               => String::from("Char"),               // '
      TokenType::FormattedRawString => String::from("FormattedRawString"), // f``
      TokenType::FormattedString    => String::from("FormattedString"),    // f""
      TokenType::FormattedChar      => String::from("FormattedChar"),      // f''
     
      // single math
      TokenType::Plus     => String::from("+"), // +
      TokenType::Minus    => String::from("-"), // -
      TokenType::Multiply => String::from("*"), // *
      TokenType::Divide   => String::from("/"), // /
      TokenType::Equals   => String::from("="), // =
      TokenType::Modulo   => String::from("%"), // %
      TokenType::Exponent => String::from("^"), // ^
      
      // double math
      TokenType::UnaryPlus      => String::from("++"), // ++
      TokenType::PlusEquals     => String::from("+="), // +=

      TokenType::UnaryMinus     => String::from("--"), // --
      TokenType::MinusEquals    => String::from("-="), // -=

      TokenType::UnaryMultiply  => String::from("**"), // **
      TokenType::MultiplyEquals => String::from("*="), // *=

      TokenType::UnaryDivide    => String::from("//"), // //
      TokenType::DivideEquals   => String::from("/="), // /=

      TokenType::UnaryModulo    => String::from("%%"), // %%
      TokenType::ModuloEquals   => String::from("%="), // %=

      TokenType::UnaryExponent  => String::from("^^"), // ^^
      TokenType::ExponentEquals => String::from("^="), // ^=

      // single logical
      TokenType::GreaterThan => String::from(">"), // >
      TokenType::LessThan    => String::from("<"), // <
      TokenType::Question    => String::from("?"), // ?
      TokenType::Not         => String::from("!"), // !
      
      // double logical
      TokenType::GreaterThanOrEquals => String::from(">="),  // >=
      TokenType::LessThanOrEquals    => String::from("<="),  // <=
      TokenType::NotEquals           => String::from("!="),  // !=
      
      // brackets
      TokenType::CircleBracketBegin => String::from("("), // (
      TokenType::CircleBracketEnd   => String::from(")"), // )
      TokenType::SquareBracketBegin => String::from("["), // [
      TokenType::SquareBracketEnd   => String::from("]"), // ]
      TokenType::FigureBracketBegin => String::from("{"), // {
      TokenType::FigureBracketEnd   => String::from("}"), // }
      
      // other
      TokenType::Colon   => String::from(":"),  // :
      TokenType::Pointer => String::from("->"), // ->

      TokenType::Tilde       => String::from("~"),   // ~
      TokenType::DoubleTilde => String::from("~~"),  // ~~

      TokenType::Link => String::from("Link"), // Link
      
      // words
      TokenType::Int      => String::from("Int"),      // Integer
      TokenType::UInt     => String::from("UInt"),     // Unsigned integer
      TokenType::Float    => String::from("Float"),    // Float
      TokenType::UFloat   => String::from("UFloat"),   // Unsigned float
      TokenType::Rational => String::from("Rational"), // Rational
      TokenType::Complex  => String::from("Complex"),  // Complex

      TokenType::Bool      => String::from("Bool"),      // Bool
      TokenType::Joint     => String::from("Joint"),     // & (and)
      TokenType::Disjoint  => String::from("Disjoint"),  // ^
      TokenType::Inclusion => String::from("Inclusion"), // | (or)
      TokenType::Exclusion => String::from("Exclusion"), // ! (not)

      TokenType::Custom(value) => value.clone(),
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

#[derive(Clone)]
pub struct Token 
{
        data: Option< String >,
    dataType: Option< TokenType >,
  pub tokens: Option< Vec<Token> >,
}
impl Token 
{
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

  // convert type
  // todo:
  /*
  fn convertType(&mut self) -> ()
  {
    if self.data.chars().nth(0) == Some('-') 
    {
      self.dataType = 
        match self.dataType 
        {
          TokenType::UInt => TokenType::Int,
          TokenType::UFloat => TokenType::Float,
          _ => self.dataType.clone(),
        }
    }
  }
  */

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

  //
  pub fn getDataType(&self) -> Option< TokenType >
  {
    self.dataType.clone()
  }
  //
  pub fn setDataType(&mut self, newDataType: Option< TokenType >) -> ()
  {
    self.dataType = newDataType;
    self.convertData();
  }

  //
  pub fn getData(&self) -> Option< String >
  {
    self.data.clone()
  }
  //
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