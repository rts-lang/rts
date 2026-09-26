use std::fmt;
use crate::parser::structure::tokenValue::uf64::uf64;
// =================================================================================================

// Чтобы понять, что выполняет Value, нужно понять следующее:
// TokenType - это просто абстрактные данные для токенайзера и парсера;
// Token - хранит data = Vec<u8>, что есть абстрактные данные;
// А Value - это математика абстрактных данных, т.е. самих токенов.
// Это все потому, что Token не может знать что есть какой тип - это дело структур;
// Поэтому тут не должно быть ABI типов - это разные вещи.

// =================================================================================================

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Value 
{
  None(),
  
  Int(i64),
  UInt(u64),
  Float(f64),
  UFloat(uf64),
  
  Char(char),
  String(String),
}

impl Value 
{
  // to bool
  pub fn toBool(&self) -> bool 
  {
    match self 
    {
      Self::None() => false, // todo непонятно что нужно возвращать здесь
      Self::Int(v) => *v!=0,
      Self::UInt(v) => *v!=0,
      Self::Float(v) => *v!=0.0,
      Self::UFloat(v) => *v!=uf64::from(0.0),
      Self::Char(c) => *c!='\0',
      Self::String(s) => !s.is_empty(),
    }
  }
}

impl fmt::Display for Value 
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
  {
    match *self 
    {
      Self::None() => write!(f, "None"), // todo непонятно что нужно возвращать здесь
      Self::Int(val) => write!(f, "{}", val),
      Self::UInt(val) => write!(f, "{}", val),
      Self::Float(val) => write!(f, "{}", val),
      Self::UFloat(val) => write!(f, "{}", val),
      Self::Char(val) => write!(f, "{}", val),
      Self::String(ref val) => write!(f, "{}", val),
    }
  }
}

// =================================================================================================

// plus
impl std::ops::Add for Value 
{
  type Output = Self;
  fn add(self, other: Self) -> Self 
  {
    match (self.clone(), other) 
    {
      // None
      // None + None обрабатывается в _
      (Self::None(), Self::Int(y))    => Self::Int(y),
      (Self::None(), Self::UInt(y))   => Self::UInt(y),
      (Self::None(), Self::Float(y))  => Self::Float(y),
      (Self::None(), Self::UFloat(y)) => Self::UFloat(y),
      (Self::None(), Self::Char(y))   => Self::Char(y),
      (Self::None(), Self::String(y)) => Self::String(y),
      // Int
      (Self::Int(x), Self::Int(y))    => Self::Int   (x+y),
      (Self::Int(x), Self::UInt(y))   => Self::Int   (x+ y as i64),
      (Self::Int(x), Self::Float(y))  => Self::Float (x as f64 +y),
      (Self::Int(x), Self::UFloat(y)) => Self::Float (x as f64 +f64::from(y)),
      (Self::Int(x), Self::Char(y))   => Self::Int   (x+ y as i64),
      (Self::Int(x), Self::String(y)) => Self::String(x.to_string() +&y),
      // UInt
      (Self::UInt(x), Self::UInt(y))   => Self::UInt  (x+y),
      (Self::UInt(x), Self::Int(y))    => Self::Int   (x as i64 +y),
      (Self::UInt(x), Self::Float(y))  => Self::Float (x as f64 +y),
      (Self::UInt(x), Self::UFloat(y)) => Self::UFloat(uf64::from(x) +y),
      (Self::UInt(x), Self::Char(y))   => Self::UInt  (x+ y as u64),
      (Self::UInt(x), Self::String(y)) => Self::String(x.to_string() +&y),
      // Float
      (Self::Float(x), Self::Float(y))  => Self::Float (x+y),
      (Self::Float(x), Self::Int(y))    => Self::Float (x+ y as f64),
      (Self::Float(x), Self::UInt(y))   => Self::Float (x+ y as f64),
      (Self::Float(x), Self::UFloat(y)) => Self::Float (x+ f64::from(y)),
      (Self::Float(x), Self::String(y)) => Self::String(x.to_string() +&y),
      // UFloat
      (Self::UFloat(x), Self::UFloat(y)) => Self::UFloat(x+y),
      (Self::UFloat(x), Self::Int(y))    => Self::Float (f64::from(x)+ y as f64),
      (Self::UFloat(x), Self::UInt(y))   => Self::UFloat(x+ uf64::from(y)),
      (Self::UFloat(x), Self::Float(y))  => Self::Float (f64::from(x) +y),
      (Self::UFloat(x), Self::String(y)) => Self::String(x.to_string() +&y),
      // Char
      (Self::Char(x), Self::Char(y)) => 
      {
        Self::Char(
          char::from_u32(x as u32 + y as u32).unwrap_or('\0')
        )
      },
      (Self::Char(x), Self::Int(y)) => 
      {
        Self::Char(
          char::from_u32((x as i64 +y) as u32).unwrap_or('\0')
        )
      },
      (Self::Char(x), Self::UInt(y)) => 
      {
        Self::Char(
          char::from_u32((x as u64 +y) as u32).unwrap_or('\0')
        )
      },
      (Self::Char(x), Self::String(y)) => Self::String(x.to_string()+ &y),
      // String
      (Self::String(x), Self::String(y)) => Self::String(x+ &y),
      (Self::String(x), Self::Int(y))    => Self::String(x+ &y.to_string()),
      (Self::String(x), Self::UInt(y))   => Self::String(x+ &y.to_string()),
      (Self::String(x), Self::Float(y))  => Self::String(x+ &y.to_string()),
      (Self::String(x), Self::UFloat(y)) => Self::String(x+ &y.to_string()),
      (Self::String(x), Self::Char(y))   => Self::String(x+ &y.to_string()),
      //
      _ => self
    }
  }
}

// =================================================================================================

// minus
impl std::ops::Sub for Value 
{
  type Output = Self;
  fn sub(self, other: Self) -> Self 
  {
    match (self.clone(), other) 
    {
      // None
      // None + None обрабатывается в _
      (Self::None(), Self::Int(y))    => Self::Int(y),
      (Self::None(), Self::UInt(y))   => Self::UInt(y),
      (Self::None(), Self::Float(y))  => Self::Float(y),
      (Self::None(), Self::UFloat(y)) => Self::UFloat(y),
      (Self::None(), Self::Char(y))   => Self::Char(y),
      (Self::None(), Self::String(y)) => Self::String(y),
      // Int
      (Self::Int(x), Self::Int(y))    => Self::Int  (x-y),
      (Self::Int(x), Self::UInt(y))   => Self::Int  (x- y as i64),
      (Self::Int(x), Self::Float(y))  => Self::Float(x as f64 -y),
      (Self::Int(x), Self::UFloat(y)) => Self::Float(x as f64 -f64::from(y)),
      (Self::Int(x), Self::Char(y))   => Self::Int  (x- y as i64),
      // UInt
      (Self::UInt(x), Self::UInt(y)) => 
      {
        match y > x 
        {
          true  => { Self::UInt(0) }  
          false => { Self::UInt(x-y) }
        }
      },
      (Self::UInt(x), Self::Int(y))    => Self::Int   (x as i64 -y),
      (Self::UInt(x), Self::Float(y))  => Self::Float (x as f64 -y),
      (Self::UInt(x), Self::UFloat(y)) => Self::UFloat(uf64::from(x) -y),
      (Self::UInt(x), Self::Char(y))   => Self::UInt  (x- y as u64),
      // Float
      (Self::Float(x), Self::Float(y))  => Self::Float(x-y),
      (Self::Float(x), Self::Int(y))    => Self::Float(x- y as f64),
      (Self::Float(x), Self::UInt(y))   => Self::Float(x- y as f64),
      (Self::Float(x), Self::UFloat(y)) => Self::Float(x- f64::from(y)),
      // UFloat
      (Self::UFloat(x), Self::UFloat(y)) => Self::UFloat(x-y),
      (Self::UFloat(x), Self::Int(y))    => Self::Float (f64::from(x)- y as f64),
      (Self::UFloat(x), Self::UInt(y))   => Self::UFloat(x- uf64::from(y)),
      (Self::UFloat(x), Self::Float(y))  => Self::Float (f64::from(x) -y),
      // Char
      (Self::Char(x), Self::Char(y)) => 
      {
        Self::Char(
          char::from_u32(x as u32 - y as u32).unwrap_or('\0')
        )
      },
      (Self::Char(x), Self::Int(y)) => 
      {
        Self::Char(
          char::from_u32((x as i64 -y) as u32).unwrap_or('\0')
        )
      },
      (Self::Char(x), Self::UInt(y)) => 
      {
        Self::Char(
          char::from_u32((x as u64 - y) as u32).unwrap_or('\0')
        )
      },
      //
      _ => self
    }
  }
}

// =================================================================================================

// multiple
impl std::ops::Mul for Value 
{
  type Output = Self;
  fn mul(self, other: Self) -> Self 
  {
    match (self.clone(), other) 
    {
      // None
      // None + None обрабатывается в _
      (Self::None(), Self::Int(y))    => Self::Int(y),
      (Self::None(), Self::UInt(y))   => Self::UInt(y),
      (Self::None(), Self::Float(y))  => Self::Float(y),
      (Self::None(), Self::UFloat(y)) => Self::UFloat(y),
      (Self::None(), Self::Char(y))   => Self::Char(y),
      (Self::None(), Self::String(y)) => Self::String(y),
      // Int
      (Self::Int(x), Self::Int(y))    => Self::Int  (x*y),
      (Self::Int(x), Self::UInt(y))   => Self::Int  (x* y as i64),
      (Self::Int(x), Self::Float(y))  => Self::Float(x as f64 *y),
      (Self::Int(x), Self::UFloat(y)) => Self::Float(x as f64 /f64::from(y)),
      // UInt
      (Self::UInt(x), Self::UInt(y))   => Self::UInt  (x*y),
      (Self::UInt(x), Self::Int(y))    => Self::Int   (x as i64 *y),
      (Self::UInt(x), Self::Float(y))  => Self::Float (x as f64 *y),
      (Self::UInt(x), Self::UFloat(y)) => Self::UFloat(uf64::from(x) *y),
      // Float
      (Self::Float(x), Self::Float(y))  => Self::Float(x*y),
      (Self::Float(x), Self::Int(y))    => Self::Float(x* y as f64),
      (Self::Float(x), Self::UInt(y))   => Self::Float(x* y as f64),
      (Self::Float(x), Self::UFloat(y)) => Self::Float(x* f64::from(y)),
      // UFloat
      (Self::UFloat(x), Self::UFloat(y)) => Self::UFloat(x*y),
      (Self::UFloat(x), Self::Int(y))    => Self::Float (f64::from(x)* y as f64),
      (Self::UFloat(x), Self::UInt(y))   => Self::UFloat(x* uf64::from(y)),
      (Self::UFloat(x), Self::Float(y))  => Self::Float (f64::from(x) *y),
      //
      _ => self
    }
  }
}

// =================================================================================================

// divide
impl std::ops::Div for Value 
{
  type Output = Self;
  fn div(self, other: Self) -> Self 
  {
    match (self.clone(), other) 
    {
      // None
      // None + None обрабатывается в _
      (Self::None(), Self::Int(y))    => Self::Int(y),
      (Self::None(), Self::UInt(y))   => Self::UInt(y),
      (Self::None(), Self::Float(y))  => Self::Float(y),
      (Self::None(), Self::UFloat(y)) => Self::UFloat(y),
      (Self::None(), Self::Char(y))   => Self::Char(y),
      (Self::None(), Self::String(y)) => Self::String(y),
      // Int
      (Self::Int(x), Self::Int(y))    => Self::Int  (x/y),
      (Self::Int(x), Self::UInt(y))   => Self::Int  (x/ y as i64),
      (Self::Int(x), Self::Float(y))  => Self::Float(x as f64 /y),
      (Self::Int(x), Self::UFloat(y)) => Self::Float(x as f64 /f64::from(y)),
      // UInt
      (Self::UInt(x), Self::UInt(y))   => Self::UInt  (x/y),
      (Self::UInt(x), Self::Int(y))    => Self::Int   (x as i64 /y),
      (Self::UInt(x), Self::Float(y))  => Self::Float (x as f64 /y),
      (Self::UInt(x), Self::UFloat(y)) => Self::UFloat(uf64::from(x) /y),
      // Float
      (Self::Float(x), Self::Float(y))  => Self::Float(x/y),
      (Self::Float(x), Self::Int(y))    => Self::Float(x/ y as f64),
      (Self::Float(x), Self::UInt(y))   => Self::Float(x/ y as f64),
      (Self::Float(x), Self::UFloat(y)) => Self::Float(x/ f64::from(y)),
      // UFloat
      (Self::UFloat(x), Self::UFloat(y)) => Self::UFloat(x/y),
      (Self::UFloat(x), Self::Int(y))    => Self::Float (f64::from(x)/ y as f64),
      (Self::UFloat(x), Self::UInt(y))   => Self::UFloat(x/ uf64::from(y)),
      (Self::UFloat(x), Self::Float(y))  => Self::Float (f64::from(x) /y),
      //
      _ => self
    }
  }
}

// =================================================================================================