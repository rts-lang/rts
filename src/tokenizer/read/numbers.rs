use crate::tokenizer::types::token::{Token, TokenType};
// =================================================================================================

/// Проверяет что байт является числом
pub fn isDigit(c: &u8) -> bool
{
  *c >= b'0' && *c <= b'9'
}

// =================================================================================================

/// Проверяет buffer по index и так находит возможные примитивные числовые типы данных;
/// `UInt, Int, UFloat, Float, Rational, Complex`
///
/// todo: Ввести Complex числа;
///
/// todo: Ввести работу float с .1 или . как 0.0
pub fn getNumber(buffer: &[u8], index: &mut usize, bufferLength: &usize) -> Token
{
  let mut savedIndex: usize = *index; // index buffer
  let mut result: String = String::from(buffer[savedIndex] as char);
  savedIndex += 1;

  let mut      dot: bool = false; // dot check
  let mut negative: bool = false; // negative check
  let rational: bool = false; // rational check

  let mut byte1: u8; // Текущий символ
  let mut byte2: u8; // Следующий символ
  while savedIndex < *bufferLength
  {
    byte1 = buffer[savedIndex]; // Значение текущего символа
    byte2 =                     // Значение следующего символа
      match savedIndex+1 < *bufferLength
      {
        true  => { buffer[savedIndex+1] }
        false => { b'\0' }
      };

    // todo: use match case
    if !negative && buffer[*index] == b'-'
    { // Int/Float
      result.push(byte1 as char);
      negative = true;
      savedIndex += 1;
    } else
    if isDigit(&byte1)
    { // UInt
      result.push(byte1 as char);
      savedIndex += 1;
    } else
    if byte1 == b'.' && !dot && isDigit(&byte2) //&&
      //savedIndex > 1 && buffer[*index-1] != b'.' // fixed for a.0.1 // todo Я убрал это, но мб зря
    { // UFloat
      match rational
      { false => {}
        true => { break; }
      }
      dot = true;
      result.push(byte1 as char);
      savedIndex += 1;
    } /*else // todo Rational пока что нет как типа
    if byte1 == b'/' && byte2 == b'/' && !dot &&
      (savedIndex+2 < *bufferLength && isDigit(&buffer[savedIndex+2]))
    { // Rational
      rational = true;
      result.push_str("//");
      savedIndex += 2;
    }*/ else { break; }
  }

  *index = savedIndex;

  // next return
  match (rational, dot, negative)
  { // rational, dot, negative
    // (true, _, _)     => Token::new( TokenType::Rational, result ), // todo Rational пока что нет как типа
    (_, true, true)  => Token::new( TokenType::Float,    result ),
    (_, true, false) => Token::new( TokenType::UFloat,   result ),
    (_, false, true) => Token::new( TokenType::Int,      result ),
    _                => Token::new( TokenType::UInt,     result ),
  }
  //
}

// =================================================================================================

#[cfg(test)]
mod tests
{
  use crate::tokenizer::read::tests::{checkSplit, checkThroughOthers, checkValues, getTokensFromBuffer};
  use crate::tokenizer::types::token::{Token, TokenType};

  // ===============================================================================================

  /// Проверяем тип и значение
  #[test]
  fn values()
  {
    checkValues([
      // UInt
      ("0", TokenType::UInt),
      ("1", TokenType::UInt),
      ("1234567890", TokenType::UInt),
      
      // Int
      ("-0", TokenType::Int),
      ("-1", TokenType::Int),
      ("-987654321", TokenType::Int),
      
      // UFloat
      ("3.14", TokenType::UFloat),
      //("3.", TokenType::UFloat), // todo Сейчас нет реализации 3. = 3.0
      //(".14", TokenType::UFloat), // todo Сейчас нет реализации .14 = 0.14
      //(".", TokenType::UInt), // todo Сейчас нет реализации . = 0.0
      
      // UInt
      ("-14.3", TokenType::Float),
      ("-2.5", TokenType::Float),
      ("-100.1000", TokenType::Float),
      
      // Rational
      //("1//2", TokenType::Rational), // todo Rational пока что нет как типа
    ]);
  }

  /// Проверяет разделение пробелами на несколько токенов
  #[test]
  fn split()
  {
    checkSplit(&[
      // UInt
      ("1 2 3", &[
        TokenType::UInt,
        TokenType::UInt,
        TokenType::UInt,
      ]),

      // Int
      ("-1 -2 -3", &[
        TokenType::Int,
        TokenType::Int,
        TokenType::Int,
      ]),

      // UFloat
      ("1.1 2.2 3.3", &[
        TokenType::UFloat,
        TokenType::UFloat,
        TokenType::UFloat,
      ]),

      // Float
      ("-1.1 -2.2 -3.3", &[
        TokenType::Float,
        TokenType::Float,
        TokenType::Float,
      ]),

      // Rational
      // todo Rational пока что нет как типа
      // ("1//2 3//4 5//6", &[
      //   TokenType::Rational,
      //   TokenType::Rational,
      //   TokenType::Rational,
      // ]),

      // Все числовые типы
      ("1 -1 1.1 -1.1", &[ // todo Rational пока что нет как типа
        TokenType::UInt,
        TokenType::Int,
        TokenType::UFloat,
        TokenType::Float,
      ]),
    ]);
  }

  /// Проверяет максимальный размер
  #[test]
  fn maxSize()
  {
    // todo В целом это логика не tokenizer, а value;
    //  Но пока что не известно будут ли лимиты на типы данных, 
    //  следует ли их проверять заранее - поэтому пока что оно останется тут.
    //  Хотя лимитов не должно быть у примитивов.
  }
  
  /// Проверяет через несколько токенов
  #[test]
  fn throughOthers() -> ()
  {
    checkThroughOthers([
      ("a=10", "a", "=", TokenType::UInt),
      ("b=-20", "b", "=", TokenType::Int),
      ("c=3.14", "c", "=", TokenType::UFloat),
      ("d=-2.5", "d", "=", TokenType::Float),
      // ("e=1//2", "e", "=", TokenType::Rational) // todo Rational пока что нет как типа
    ]);
  }

  // ===============================================================================================
}

// =================================================================================================