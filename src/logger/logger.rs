// =================================================================================================
use crate::_debugMode;
use termion::color::{Bg, Fg, Rgb, Reset};
use termion::style;
// =================================================================================================

// hex str -> termion::color::Rgb
fn hexToTermionColor(hex: &str) -> Option<Rgb>
{
  match hex.len() != 6 
  { 
    true => None,
    false => {
      Some(Rgb(
        u8::from_str_radix(&hex[0..2], 16).ok()?, 
        u8::from_str_radix(&hex[2..4], 16).ok()?, 
        u8::from_str_radix(&hex[4..6], 16).ok()?
      ))
    } 
  }
  //
}
// devide white space, begin from the left
fn divideWhitespace(input: &str) -> (&str, &str) 
{
  let firstNonSpaceIndex: usize = input
    .find(|c: char| !c.is_whitespace())
    .unwrap_or(input.len());
  (&input[..firstNonSpaceIndex], &input[firstNonSpaceIndex..])
}

// =================================================================================================

// style log
pub fn formatPrint(string: &str) -> ()
{
  print!("{}",formatString(string));
}

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
pub fn formatString(inputString: &str) -> String 
{
  let mut result: String = String::new();

  let mut i: usize = 0;
  let stringChars: Vec<char> = inputString.chars().collect();
  let stringLength: usize = stringChars.len();
  let mut string: String;

  while i < stringLength 
  { // special 
    if stringChars[i] == '\\' && i+1 < stringLength &&
       ((i == 0) || (i > 0 && stringChars[i-1] != '\\')) // Проверяем на экранировние
    {
      match stringChars[i+1] 
      {
        // todo: Добавить \t и другие варианты
        'n' => 
        {
          i += 2;
          result.push('\n');
          continue;
        }
        'b' => 
        {
          if i+2 < stringLength && stringChars[i+2] == 'g' 
          { // bg
            i += 5;
            string = String::from_iter(
              stringChars[i..stringLength]
                .iter()
                .take_while(|&&c| c != ')')
            );
            result.push_str(&format!(
              "{}",
              Bg(hexToTermionColor(string.as_str()).unwrap_or(Rgb(0, 0, 0)))
            ));
            i += string.len()+1;
            continue;
          }  
          else
          { // bold
            result.push_str( &format!("{}",style::Bold) );
            i += 2;
            continue;
          }
        }
        'f' => 
        {
          if i+2 < stringLength && stringChars[i+2] == 'g' 
          { // fg
            i += 5;
            string = String::from_iter(
              stringChars[i..stringLength]
                .iter()
                .take_while(|&&c| c != ')')
            );
            result.push_str(&format!(
              "{}",
              Fg(hexToTermionColor(&string).unwrap_or(Rgb(0, 0, 0)))
            ));
            i += string.len()+1;
            continue;
          }
        }
        'c' => 
        { // clear
          if i+2 < stringLength && stringChars[i+2] == 'b' 
          {
            if i+3 < stringLength && stringChars[i+3] == 'g' 
            { // cbg
              i += 4;
              result.push_str(&format!(
                "{}",
                Bg(Reset)
              ));
              continue;
            } else
            { // cb
              i += 3;
              result.push_str(&format!(
                "{}",
                style::NoBold
              ));
              continue;
            }
          } else
          if i+2 < stringLength && stringChars[i+2] == 'f' 
          {
            if i+3 < stringLength && stringChars[i+3] == 'g' 
            { // cfg
              i += 4;
              result.push_str(&format!(
                "{}",
                Fg(Reset)
              ));
              continue;
            }
          } else 
          { // clear all
            i += 2;
            result.push_str(&format!(
              "{}",
              style::Reset
            ));
            continue;
          }
        }
        _ => 
        {
          result.push_str("\\");
          i += 1;
          continue;
        }
      }
    // basic
    } else {
      result.push( stringChars[i] );
    }
    i += 1;
  }
  result.clone()
}

// =================================================================================================

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
      if unsafe{_debugMode} {
        formatPrint("   \\b┗\\fg(#1ae96b) Exit 0\\c \\fg(#f0f8ff)\\b:)\\c\n");
      }
      std::process::exit(0);
    }
    false => 
    { // В данном случае завершение не успешное;
      if unsafe{_debugMode}
      {
        formatPrint(
          &format!(
            "   \\b┗\\fg(#e91a34) Exit {}\\c \\fg(#f0f8ff)\\b:(\\c\n", 
            code
          )
        );
      }
      std::process::exit(code);
    }
  }
}

// =================================================================================================

// basic style log
pub fn log(textType: &str, text: &str) -> ()
{
  let mut parts: Vec<String>;
  let mut outputParts: Vec<String>;
  
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
    {
      parts = text.split("|").map(|s| s.to_string()).collect();
      outputParts = Vec::new();
      // first word no format
      match parts.first() 
      {
        None => {}
        Some(firstPart) => 
        {
          outputParts.push( formatString(firstPart) );
        }
      }
      // last word
      for part in parts.iter().skip(1) 
      {
        outputParts.push(
          formatString(&format!(
            "\\b\\fg(#d9d9d9){}\\c",
            part
          ))
        );
      }
      println!("{}", outputParts.join(""));
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
    {
      parts = text.split("->").map(|s| s.to_string()).collect();
      let string: String = 
        parts.join(
          &formatString("\\fg(#f0f8ff)\\b->\\c")
        );
      formatPrint(&format!(
        "\\fg(#f0f8ff)\\b->\\c \\fg(#f0f8ff){}\\c\n",
        string
      ));
    }} 
    "line" => 
    { // line
    {
      parts = text.split("|").map(|s| s.to_string()).collect();
      outputParts = Vec::new();
      // left
      match parts.first() 
      {
        Some(firstPart) => 
        {
          outputParts.push(
            formatString(&format!(
              "  \\fg(#f0f8ff)\\b{} | \\c",
              firstPart
            ))
          );
        }
        None => {}
      }
      // right
      for part in parts.iter().skip(1) 
      {
        outputParts.push(part.to_string());
      }
      println!("{}", outputParts.join(""));
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

// =================================================================================================