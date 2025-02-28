/* /parser/value
  Interaction of primitives through operators
*/

use std::fmt;
use crate::parser::uf64::*;

// Value ===========================================================================================
#[derive(Clone, PartialEq, PartialOrd)]
pub enum Value 
{
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
      Value::Int(v)    => *v!=0,
      Value::UInt(v)   => *v!=0,
      Value::Float(v)  => *v!=0.0,
      Value::UFloat(v) => *v!=uf64::from(0.0),
      Value::Char(c)   => *c!='\0',
      Value::String(s) => !s.is_empty(),
    }
  }
}
impl fmt::Display for Value 
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
  {
    match *self 
    {
      Value::Int   (val)     => write!(f, "{}", val),
      Value::UInt  (val)     => write!(f, "{}", val),
      Value::Float (val)     => write!(f, "{}", val),
      Value::UFloat(val)     => write!(f, "{}", val),
      Value::Char  (val)     => write!(f, "{}", val),
      Value::String(ref val) => write!(f, "{}", val),
    }
  }
}

// + ===============================================================================================
impl std::ops::Add for Value 
{
  type Output = Self;
  fn add(self, other: Self) -> Self 
  {
    match (self.clone(), other) 
    {
      // Int
      (Value::Int(x), Value::Int(y))    => Value::Int   (x+y),
      (Value::Int(x), Value::UInt(y))   => Value::Int   (x+ y as i64),
      (Value::Int(x), Value::Float(y))  => Value::Float (x as f64 +y),
      (Value::Int(x), Value::UFloat(y)) => Value::Float (x as f64 +f64::from(y)),
      (Value::Int(x), Value::Char(y))   => Value::Int   (x+ y as i64),
      (Value::Int(x), Value::String(y)) => Value::String(x.to_string() +&y),
      // UInt
      (Value::UInt(x), Value::UInt(y))   => Value::UInt  (x+y),
      (Value::UInt(x), Value::Int(y))    => Value::Int   (x as i64 +y),
      (Value::UInt(x), Value::Float(y))  => Value::Float (x as f64 +y),
      (Value::UInt(x), Value::UFloat(y)) => Value::UFloat(uf64::from(x) +y),
      (Value::UInt(x), Value::Char(y))   => Value::UInt  (x+ y as u64),
      (Value::UInt(x), Value::String(y)) => Value::String(x.to_string() +&y),
      // Float
      (Value::Float(x), Value::Float(y))  => Value::Float (x+y),
      (Value::Float(x), Value::Int(y))    => Value::Float (x+ y as f64),
      (Value::Float(x), Value::UInt(y))   => Value::Float (x+ y as f64),
      (Value::Float(x), Value::UFloat(y)) => Value::Float (x+ f64::from(y)),
      (Value::Float(x), Value::String(y)) => Value::String(x.to_string() +&y),
      // UFloat
      (Value::UFloat(x), Value::UFloat(y)) => Value::UFloat(x+y),
      (Value::UFloat(x), Value::Int(y))    => Value::Float (f64::from(x)+ y as f64),
      (Value::UFloat(x), Value::UInt(y))   => Value::UFloat(x+ uf64::from(y)),
      (Value::UFloat(x), Value::Float(y))  => Value::Float (f64::from(x) +y),
      (Value::UFloat(x), Value::String(y)) => Value::String(x.to_string() +&y),
      // Char
      (Value::Char(x), Value::Char(y)) => 
      {
        Value::Char(
          match std::char::from_u32(x as u32 + y as u32)
          {
            Some(resultChar) => {resultChar} 
            None             => {'\0'}
          }
        )
      },
      (Value::Char(x), Value::Int(y))  => 
      {
        Value::Char(
          match std::char::from_u32((x as i64 +y) as u32)
          {
            Some(resultChar) => {resultChar} 
            None             => {'\0'}
          }
        )
      },
      (Value::Char(x), Value::UInt(y)) => 
      {
        Value::Char(
          match std::char::from_u32((x as u64 +y) as u32) 
          {
            Some(resultChar) => {resultChar} 
            None             => {'\0'}
          }
        )
      },
      (Value::Char(x), Value::String(y)) => Value::String(x.to_string()+ &y),
      // String
      (Value::String(x), Value::String(y)) => Value::String(x+ &y),
      (Value::String(x), Value::Int(y))    => Value::String(x+ &y.to_string()),
      (Value::String(x), Value::UInt(y))   => Value::String(x+ &y.to_string()),
      (Value::String(x), Value::Float(y))  => Value::String(x+ &y.to_string()),
      (Value::String(x), Value::UFloat(y)) => Value::String(x+ &y.to_string()),
      (Value::String(x), Value::Char(y))   => Value::String(x+ &y.to_string()),
      //
      _ => self
    }
  }
}
// - ===============================================================================================
impl std::ops::Sub for Value 
{
  type Output = Self;
  fn sub(self, other: Self) -> Self 
  {
    match (self.clone(), other) 
    {
      // Int
      (Value::Int(x), Value::Int(y))    => Value::Int  (x-y),
      (Value::Int(x), Value::UInt(y))   => Value::Int  (x- y as i64),
      (Value::Int(x), Value::Float(y))  => Value::Float(x as f64 -y),
      (Value::Int(x), Value::UFloat(y)) => Value::Float(x as f64 -f64::from(y)),
      (Value::Int(x), Value::Char(y))   => Value::Int  (x- y as i64),
      // UInt
      (Value::UInt(x), Value::UInt(y))   => 
      {
        match y > x 
        {
          true  => { Value::UInt(0) }  
          false => { Value::UInt(x-y) }
        }
      },
      (Value::UInt(x), Value::Int(y))    => Value::Int   (x as i64 -y),
      (Value::UInt(x), Value::Float(y))  => Value::Float (x as f64 -y),
      (Value::UInt(x), Value::UFloat(y)) => Value::UFloat(uf64::from(x) -y),
      (Value::UInt(x), Value::Char(y))   => Value::UInt  (x- y as u64),
      // Float
      (Value::Float(x), Value::Float(y))  => Value::Float(x-y),
      (Value::Float(x), Value::Int(y))    => Value::Float(x- y as f64),
      (Value::Float(x), Value::UInt(y))   => Value::Float(x- y as f64),
      (Value::Float(x), Value::UFloat(y)) => Value::Float(x- f64::from(y)),
      // UFloat
      (Value::UFloat(x), Value::UFloat(y)) => Value::UFloat(x-y),
      (Value::UFloat(x), Value::Int(y))    => Value::Float (f64::from(x)- y as f64),
      (Value::UFloat(x), Value::UInt(y))   => Value::UFloat(x- uf64::from(y)),
      (Value::UFloat(x), Value::Float(y))  => Value::Float (f64::from(x) -y),
      // Char
      (Value::Char(x), Value::Char(y)) => 
      {
        Value::Char(
          match std::char::from_u32(x as u32 - y as u32)
          {
            Some(resultChar) => {resultChar} 
            None => {'\0'}
          }
        )
      },
      (Value::Char(x), Value::Int(y))  => 
      {
        Value::Char(
          match std::char::from_u32((x as i64 -y) as u32)
          {
            Some(resultChar) => {resultChar} 
            None => {'\0'}
          }
        )
      },
      (Value::Char(x), Value::UInt(y)) => 
      {
        Value::Char(
          match std::char::from_u32((x as u64 -y) as u32)
          {
            Some(resultChar) => {resultChar} 
            None => {'\0'}
          }
        )
      },
      //
      _ => self
    }
  }
}
// * ===============================================================================================
impl std::ops::Mul for Value 
{
  type Output = Self;
  fn mul(self, other: Self) -> Self 
  {
    match (self.clone(), other) 
    {
      // Int
      (Value::Int(x), Value::Int(y))    => Value::Int  (x*y),
      (Value::Int(x), Value::UInt(y))   => Value::Int  (x* y as i64),
      (Value::Int(x), Value::Float(y))  => Value::Float(x as f64 *y),
      (Value::Int(x), Value::UFloat(y)) => Value::Float(x as f64 /f64::from(y)),
      // UInt
      (Value::UInt(x), Value::UInt(y))   => Value::UInt  (x*y),
      (Value::UInt(x), Value::Int(y))    => Value::Int   (x as i64 *y),
      (Value::UInt(x), Value::Float(y))  => Value::Float (x as f64 *y),
      (Value::UInt(x), Value::UFloat(y)) => Value::UFloat(uf64::from(x) *y),
      // Float
      (Value::Float(x), Value::Float(y))  => Value::Float(x*y),
      (Value::Float(x), Value::Int(y))    => Value::Float(x* y as f64),
      (Value::Float(x), Value::UInt(y))   => Value::Float(x* y as f64),
      (Value::Float(x), Value::UFloat(y)) => Value::Float(x* f64::from(y)),
      // UFloat
      (Value::UFloat(x), Value::UFloat(y)) => Value::UFloat(x*y),
      (Value::UFloat(x), Value::Int(y))    => Value::Float (f64::from(x)* y as f64),
      (Value::UFloat(x), Value::UInt(y))   => Value::UFloat(x* uf64::from(y)),
      (Value::UFloat(x), Value::Float(y))  => Value::Float (f64::from(x) *y),
      //
      _ => self
    }
  }
}
// / ===============================================================================================
impl std::ops::Div for Value 
{
  type Output = Self;
  fn div(self, other: Self) -> Self 
  {
    match (self.clone(), other) 
    {
      // Int
      (Value::Int(x), Value::Int(y))    => Value::Int  (x/y),
      (Value::Int(x), Value::UInt(y))   => Value::Int  (x/ y as i64),
      (Value::Int(x), Value::Float(y))  => Value::Float(x as f64 /y),
      (Value::Int(x), Value::UFloat(y)) => Value::Float(x as f64 /f64::from(y)),
      // UInt
      (Value::UInt(x), Value::UInt(y))   => Value::UInt  (x/y),
      (Value::UInt(x), Value::Int(y))    => Value::Int   (x as i64 /y),
      (Value::UInt(x), Value::Float(y))  => Value::Float (x as f64 /y),
      (Value::UInt(x), Value::UFloat(y)) => Value::UFloat(uf64::from(x) /y),
      // Float
      (Value::Float(x), Value::Float(y))  => Value::Float(x/y),
      (Value::Float(x), Value::Int(y))    => Value::Float(x/ y as f64),
      (Value::Float(x), Value::UInt(y))   => Value::Float(x/ y as f64),
      (Value::Float(x), Value::UFloat(y)) => Value::Float(x/ f64::from(y)),
      // UFloat
      (Value::UFloat(x), Value::UFloat(y)) => Value::UFloat(x/y),
      (Value::UFloat(x), Value::Int(y))    => Value::Float (f64::from(x)/ y as f64),
      (Value::UFloat(x), Value::UInt(y))   => Value::UFloat(x/ uf64::from(y)),
      (Value::UFloat(x), Value::Float(y))  => Value::Float (f64::from(x) /y),
      //
      _ => self
    }
  }
}