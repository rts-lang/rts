// =================================================================================================

/// Тип элементарной единицы хранения информации
/// 
/// todo Можно создать глобальную общую структуру:
///   - structure type
///   - token type
///   - string
#[derive(PartialEq)]
#[derive(Copy, Clone)]
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

  /// Bool
  Bool, // todo issue #65
  // True, False, // todo issue #65
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
      
      //
      TokenType::Int      => String::from("Int"),
      TokenType::UInt     => String::from("UInt"),
      TokenType::Float    => String::from("Float"),
      TokenType::UFloat   => String::from("UFloat"),

      //
      TokenType::Bool      => String::from("Bool"), // todo issue #65
      
      TokenType::Joint     => String::from("Joint"),
      TokenType::Disjoint  => String::from("Disjoint"),
      TokenType::Inclusion => String::from("Inclusion"),
      TokenType::Exclusion => String::from("Exclusion")
    }
    //
  }
}

impl Default for TokenType
{
  fn default() -> Self 
  {
    TokenType::None
  }
}

// =================================================================================================