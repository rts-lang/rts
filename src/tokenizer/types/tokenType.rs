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

impl TokenType
{
  /// Проверяет, подходит ли этот оператор для продолжения строки дальше; #85
  pub const fn isContinuationOperator(&self) -> bool
  {
    matches!(
      self,
      Self::Plus | Self::Minus | Self::Multiply | Self::Divide |
      Self::Equals | Self::Modulo | Self::Exponent |
      Self::GreaterThan | Self::LessThan |
      Self::GreaterThanOrEquals | Self::LessThanOrEquals | Self::NotEquals |
      Self::Colon | Self::Pointer | Self::Tilde | Self::DoubleTilde |
      Self::Dot | Self::Comma |
      Self::Joint | Self::Disjoint | Self::Inclusion | Self::Exclusion
    )
  }
}

impl ToString for TokenType
{ // todo convert -> fmt::Display ?
  fn to_string(&self) -> String 
  {
    match self 
    {
      // basic
      Self::None    => String::from("None"),
      Self::Any    => String::from("Any"),
      Self::Word    => String::from("Word"),
      Self::Endline => String::from("\\n"),
      Self::Comma   => String::from(","),
      Self::Dot     => String::from("."),

      Self::Comment => String::from("Comment"),
      
      // quotes
      Self::RawString          => String::from("RawString"),
      Self::String             => String::from("String"),
      Self::Char               => String::from("Char"),
      Self::FormattedRawString => String::from("FormattedRawString"),
      Self::FormattedString    => String::from("FormattedString"),
      Self::FormattedChar      => String::from("FormattedChar"),
     
      // single math
      Self::Plus     => String::from("+"),
      Self::Minus    => String::from("-"),
      Self::Multiply => String::from("*"),
      Self::Divide   => String::from("/"),
      Self::Equals   => String::from("="),
      Self::Modulo   => String::from("%"),
      Self::Exponent => String::from("^"),
      
      // double math
      Self::UnaryPlus      => String::from("++"),
      Self::PlusEquals     => String::from("+="),

      Self::UnaryMinus     => String::from("--"),
      Self::MinusEquals    => String::from("-="),

      Self::UnaryMultiply  => String::from("**"),
      Self::MultiplyEquals => String::from("*="),

      Self::UnaryDivide    => String::from("//"),
      Self::DivideEquals   => String::from("/="),

      Self::UnaryModulo    => String::from("%%"),
      Self::ModuloEquals   => String::from("%="),

      Self::UnaryExponent  => String::from("^^"),
      Self::ExponentEquals => String::from("^="),

      // single logical
      Self::GreaterThan => String::from(">"),
      Self::LessThan    => String::from("<"),
      Self::Question    => String::from("?"),
      Self::Not         => String::from("!"),
      
      // double logical
      Self::GreaterThanOrEquals => String::from(">="),
      Self::LessThanOrEquals    => String::from("<="),
      Self::NotEquals           => String::from("!="),
      
      // brackets
      Self::CircleBracketBegin => String::from("("),
      Self::CircleBracketEnd   => String::from(")"),
      Self::SquareBracketBegin => String::from("["),
      Self::SquareBracketEnd   => String::from("]"),
      Self::FigureBracketBegin => String::from("{"),
      Self::FigureBracketEnd   => String::from("}"),
      
      // other
      Self::Colon   => String::from(":"),
      Self::Pointer => String::from("->"),

      Self::Tilde       => String::from("~"),
      Self::DoubleTilde => String::from("~~"),

      Self::Link => String::from("Link"),
      
      //
      Self::Int      => String::from("Int"),
      Self::UInt     => String::from("UInt"),
      Self::Float    => String::from("Float"),
      Self::UFloat   => String::from("UFloat"),

      //
      Self::Bool      => String::from("Bool"), // todo issue #65

      Self::Joint     => String::from("Joint"),
      Self::Disjoint  => String::from("Disjoint"),
      Self::Inclusion => String::from("Inclusion"),
      Self::Exclusion => String::from("Exclusion")
    }
    //
  }
}

impl Default for TokenType
{
  fn default() -> Self 
  {
    Self::None
  }
}

// =================================================================================================