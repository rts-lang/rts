/* /logger
  Has both debug functions and for normal work
*/

use crate::{
  _debugMode,
};

use termion::color::{Bg, Fg, Rgb, Reset};
use termion::style;

// hex str -> termion::color::Rgb
fn hexToTermionColor(hex: &str) -> Option<Rgb> {
  match hex.len() != 6 
  { 
    true  => { return None; }
    false => {
      Some(Rgb(
        u8::from_str_radix(&hex[0..2], 16).ok()?, 
        u8::from_str_radix(&hex[2..4], 16).ok()?, 
        u8::from_str_radix(&hex[4..6], 16).ok()?
      ))
    } 
  }
}
// devide white space, begin from the left
fn divideWhitespace(input: &str) -> (&str, &str) 
{
  let firstNonSpaceIndex: usize = input
    .find(|c: char| !c.is_whitespace())
    .unwrap_or(input.len());
  (&input[..firstNonSpaceIndex], &input[firstNonSpaceIndex..])
}
// style log
pub fn formatPrint(string: &str) -> ()
{
  print!("{}",&formatString(string));
}

static mut _result: String = String::new();

static mut _i:            usize = 0;
static mut _stringLength: usize = 0;

static mut _stringChars:   Vec<char>   = Vec::new();
static mut _string:        String      = String::new();

/*
  Formats a string, you can use flags:

  \c    clear all
  
  \b    bold
  \fg   foreground
  \bg   background

  \cb   clear bold
  \cfg  clear foreground
  \cbg  clear background
*/
// todo: if -> match
pub fn formatString(string: &str) -> String 
{
  unsafe
  {
    _result = String::new();

    _i = 0;
    _stringChars  = string.chars().collect();
    _stringLength = _stringChars.len();

    while _i < _stringLength 
    { // special 
      if _stringChars[_i] == '\\' && _i+1 < _stringLength &&
         ((_i == 0) || (_i > 0 && _stringChars[_i-1] != '\\')) // Проверяем на экранировние
      {
        match _stringChars[_i+1] 
        {
          // todo: Добавить \t и другие варианты
          'n' => 
          {
            _i += 2;
            _result.push_str("\n");
            continue;
          }
          'b' => 
          {
            match _i+2 < _stringLength && _stringChars[_i+2] == 'g' 
            {
              true => 
              { // bg
                _i += 5;
                _string = String::from_iter(
                  _stringChars[_i.._stringLength]
                    .iter()
                    .take_while(|&&c| c != ')')
                );
                _result.push_str(&format!(
                  "{}",
                  Bg(hexToTermionColor(_string.as_str()).unwrap_or_else(|| Rgb(0, 0, 0)))
                ));
                _i += _string.len()+1;
                continue;
              }  
              false => 
              { // bold
                _result.push_str( &format!("{}",style::Bold) );
                _i += 2;
                continue;
              }
            }
          }
          'f' => 
          {
            match _i+2 < _stringLength && _stringChars[_i+2] == 'g' 
            {
              true => 
              { // fg
                _i += 5;
                _string = String::from_iter(
                  _stringChars[_i.._stringLength]
                    .iter()
                    .take_while(|&&c| c != ')')
                );
                _result.push_str(&format!(
                  "{}",
                  Fg(hexToTermionColor(&_string).unwrap_or_else(|| Rgb(0, 0, 0)))
                ));
                _i += _string.len()+1;
                continue;
              }
              false => {}
            }
          }
          'c' => 
          { // clear
            if _i+2 < _stringLength && _stringChars[_i+2] == 'b' 
            {
              match _i+3 < _stringLength && _stringChars[_i+3] == 'g' 
              {
                true =>
                { // cbg
                  _i += 4;
                  _result.push_str(&format!(
                    "{}",
                    Bg(Reset)
                  ));
                  continue;
                } 
                false =>
                { // cb
                  _i += 3;
                  _result.push_str(&format!(
                    "{}",
                    style::NoBold
                  ));
                  continue;
                }
              }
            } else
            if _i+2 < _stringLength && _stringChars[_i+2] == 'f' 
            {
              match _i+3 < _stringLength && _stringChars[_i+3] == 'g' 
              {
                true => 
                { // cfg
                  _i += 4;
                  _result.push_str(&format!(
                    "{}",
                    Fg(Reset)
                  ));
                  continue;
                }
                false => {}
              }
            } else 
            { // clear all
              _i += 2;
              _result.push_str(&format!(
                "{}",
                style::Reset
              ));
              continue;
            }
          }
          _ => 
          {
            _result.push_str("\\");
            _i += 1;
            continue;
          }
        }
      // basic
      } else 
      {
        _result.push( _stringChars[_i] );
      }
      _i += 1;
    }
    return _result.clone();
  }
}
// separator log
pub fn logSeparator(text: &str) -> ()
{
  formatPrint(&format!(
    " \\fg(#55af96)\\bx \\fg(#0095B6){}\\c\n",
    text
  ));
}
// Завершает программу и при необходимости в debug режиме
// возвращает описание выхода;
pub fn logExit(code: i32) -> !
{
  match code == 0 
  {
    true => 
    { // В данном случае завершение успешно;
      match unsafe{_debugMode}
      {
        true  => { formatPrint("   \\b┗\\fg(#1ae96b) Exit 0\\c \\fg(#f0f8ff)\\b:)\\c\n"); }
        false => {}
      }
      std::process::exit(0);
    }
    false => 
    { // В данном случае завершение не успешное;
      match unsafe{_debugMode}
      {
        true => 
        { 
          formatPrint(
            &format!(
              "   \\b┗\\fg(#e91a34) Exit {}\\c \\fg(#f0f8ff)\\b:(\\c\n", 
              code
            )
          );
        }
        false => {}
      }
      std::process::exit(code);
    }
  }
}
// basic style log
static mut _parts:       Vec<String> = Vec::new();
static mut _outputParts: Vec<String> = Vec::new();
pub fn log(textType: &str, text: &str) -> ()
{
  match textType 
  {
    "syntax" => 
    { //
      formatPrint("\\fg(#e91a34)\\bSyntax \\c");
    } 
    "parserBegin" => 
    { // AST open +
      let (divide1, divide2): (&str, &str) = divideWhitespace(text);
      formatPrint(&format!(
        "{}\\bg(#29352f)\\fg(#b5df90)\\b{}\\c\n",
        divide1,
        divide2
      ));
    } 
    "parserInfo" => 
    { // AST info
      let (divide1, divide2): (&str, &str) = divideWhitespace(text);
      formatPrint(&format!(
        "{}\\bg(#29352f)\\fg(#d9d9d9)\\b{}\\c\n",
        divide1,
        divide2
      ));
    } 
    "parserToken" => 
    { // AST token
    unsafe{
      _parts = text.split("|").map(|s| s.to_string()).collect();
      _outputParts = Vec::new();
      // first word no format
      match _parts.first() 
      {
        Some(firstPart) => 
        {
          _outputParts.push( formatString(firstPart) );
        }
        None => {}
      }
      // last word
      for part in _parts.iter().skip(1) 
      {
        _outputParts.push(
          formatString(&format!(
            "\\b\\fg(#d9d9d9){}\\c",
            part
          ))
        );
      }
      println!("{}", _outputParts.join(""));
    }} 
    "ok" => 
    { // ok
      let (content, prefix): (&str, &str) = 
        if text.starts_with('+') 
        {
          (&text[1..], "O\\cfg \\fg(#f0f8ff)┳")
        } else
        if text.starts_with('x') 
        {
          (&text[1..], "X\\cfg \\fg(#f0f8ff)┻")
        } else 
        {
          (text, "+")
        };
      formatPrint(&format!(
        "   \\fg(#1ae96b)\\b{}\\cb\\cfg \\fg(#f0f8ff)\\b{}\\c\n",
        prefix,
        content
      ));
    } 
    "err" => 
    { // error
      formatPrint(&format!(
        "   \\fg(#e91a34)\\b-\\cb\\cfg \\fg(#f0f8ff)\\b{}\\c\n",
        text
      ));
    } 
    "warn" => 
    { // warning
      formatPrint(&format!(
        "   \\fg(#e98e1a)\\b?\\cb\\cfg \\fg(#f0f8ff)\\b{}\\c\n",
        text
      ));
    } 
    "warn-input" => 
    { // warn input
      formatPrint(&format!(
        "   \\fg(#e98e1a)\\b?\\cb\\cfg \\fg(#f0f8ff)\\b{}\\c",
        text
      ));
    } 
    "note" => 
    { // note
      formatPrint(&format!(
        "  \\fg(#f0f8ff)\\bNote:\\c \\fg(#f0f8ff){}\\c\n",
        text
      ));
    } 
    "path" => 
    { // path
    unsafe{
      _parts = text.split("->").map(|s| s.to_string()).collect();
      _string = 
        _parts.join(
          &formatString("\\fg(#f0f8ff)\\b->\\c")
        );
      formatPrint(&format!(
        "\\fg(#f0f8ff)\\b->\\c \\fg(#f0f8ff){}\\c\n",
        _string
      ));
    }} 
    "line" => 
    { // line
    unsafe{
      _parts = text.split("|").map(|s| s.to_string()).collect();
      _outputParts = Vec::new();
      // left
      match _parts.first() 
      {
        Some(firstPart) => 
        {
          _outputParts.push(
            formatString(&format!(
              "  \\fg(#f0f8ff)\\b{} | \\c",
              firstPart.to_string()
            ))
          );
        }
        None => {}
      }
      // right
      for part in _parts.iter().skip(1) 
      {
        _outputParts.push(part.to_string());
      }
      println!("{}",_outputParts.join(""));
    }}  
    _ => 
    { // basic
      formatPrint(&format!(
        "\\fg(#f0f8ff){}\\c\n",
        text
      ));
    }
  }
}