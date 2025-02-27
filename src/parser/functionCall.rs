use std::io;
use std::io::Write;
use std::process::{Command, ExitStatus, Output};
use std::str::SplitWhitespace;
use std::sync::RwLockReadGuard;
use rand::Rng;
use crate::parser::structure::parameters::Parameters;
use crate::parser::structure::Structure;
use crate::tokenizer::token::{Token, TokenType};

impl Structure
{
  /// Запускает функцию;
  ///
  /// Функция - это такая структура, которая возвращает значение.
  ///
  /// Но кроме того, запускает не стандартные методы;
  /// В нестандартных методах могут быть процедуры, которые не вернут результат.

  /// todo: вынести все стандартные варианты в отдельный модуль
  ///
  /// todo: когда будет вынесено, то должна ожидать тип данных, который должен в Tokenizer::getWord() тоже
  pub fn functionCall(&self, value: &mut Vec<Token>, valueLength: &mut usize, i: usize) -> ()
  {
    let parameters: Parameters = self.getCallParameters(value, i, valueLength);
    match value[i].getData()
    {
      None =>
      { // Вариант в котором тип токена может быть типом данных => это cast в другой тип;
        match value[i].getDataType().unwrap_or_default()
        {
          TokenType::UInt =>
          { // Получаем значение выражения в типе
            // todo: Float, UFloat
            match parameters.getExpression(self,0)
            {
              None => {}
              Some(p0) =>
              {
                value[i].setDataType( Some(TokenType::UInt ) );
                value[i].setData( Some(p0.getData().unwrap_or_default()) );
              }
            }
          }
          TokenType::Int =>
          { // Получаем значение выражения в типе
            match parameters.getExpression(self,0)
            {
              None => {}
              Some(p0) =>
              {
                value[i].setDataType( Some(TokenType::Int ) );
                value[i].setData( Some(p0.getData().unwrap_or_default()) );
                //
              }
            }
            //
          }
          TokenType::String =>
          { // Получаем значение выражение в типе String
            // todo: подумать над formatted типами
            match parameters.getExpression(self,0)
            {
              None => {}
              Some(p0) =>
              {
                value[i].setDataType( Some(TokenType::String ) );
                value[i].setData( Some(p0.getData().unwrap_or_default()) );
                //
              }
            }
            //
          }
          TokenType::Char =>
          { // Получаем значение выражения в типе Char
            // todo: проверить работу
            match parameters.getExpression(self,0)
            {
              None => {}
              Some(p0) =>
              {
                value[i].setDataType( Some(TokenType::Char) );
                value[i].setData(
                  Some(
                    (p0.getData().unwrap_or_default()
                      .parse::<u8>().unwrap() as char
                    ).to_string()
                  )
                );
                //
              }
            }
            //
          }
          _ => {} // todo: Возможно custom варианты преобразований из custom
        }
      }
      Some(structureName) =>
      { // Вариант в котором это обращение к стандартной или custrom функции;
        // todo: проверка на нижний регистр

        // Далее идут базовые методы;
        // Эти методы ожидают аргументов
        'basicMethods:
        { // Это позволит выйти, если мы ожидаем не стандартные варианты
          match structureName.as_str()
          { // Проверяем на сходство стандартных функций

            // todo: создать resultType() ?
            //       для возвращения результата ожидаемого структурой

            "type" =>
            { // Возвращает тип данных переданной структуры
              match parameters.getExpression(self,0)
              {
                None => {}
                Some(p0) =>
                {
                  value[i].setDataType( Some(TokenType::String) );
                  value[i].setData( Some(p0.getDataType().unwrap_or_default().to_string()) );
                }
              }
            }
            "mut" =>
            { // Возвращает уровень модификации переданной структуры
              match parameters.get(0)
              {
                None => {}
                Some(p0) =>
                {
                  value[i].setDataType( Some(TokenType::String) );
                  let result: String =
                    match p0.getData()
                    {
                      None => String::from(""),
                      Some(structureName) =>
                      { // Получили название структуры
                        match self.getStructureByName(&structureName)
                        {
                          None => String::from(""),
                          Some(structureLink) =>
                          { // Получили ссылку на структуру
                            let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
                            structure.mutable.to_string()
                          }
                        }
                      }
                    };
                  value[i].setData( Some(result) );
                }
              }
            }
            "randUInt" if !parameters.isNone() =>
            { // Возвращаем случайное число типа UInt от min до max
              let min: usize =
                match parameters.getExpression(self,0)
                {
                  None => 0,
                  Some(p0) =>
                  {
                    match p0.getData()
                    {
                      Some(expressionData) => expressionData.parse::<usize>().unwrap_or_default(),
                      None => 0
                    }
                  }
                };
              let max: usize =
                match parameters.getExpression(self,1)
                {
                  None => 0,
                  Some(p1) =>
                  {
                    match p1.getData()
                    {
                      Some(expressionData) => expressionData.parse::<usize>().unwrap_or_default(),
                      None => 0
                    }
                  }
                };
              let randomNumber: usize =
                match min < max
                {
                  true  => rand::thread_rng().gen_range(min..=max),
                  false => 0
                };
              value[i].setDataType( Some(TokenType::UInt) );
              value[i].setData    ( Some(randomNumber.to_string()) );
            }
            "len" =>
            { // Получаем размер структуры;
              match parameters.getExpression(self,0)
              {
                None => {}
                Some(p0) =>
                {
                  match p0.getDataType().unwrap_or_default()
                  {
                    TokenType::None =>
                    { // Результат 0
                      value[i] = Token::new( Some(TokenType::UInt),Some(String::from("0")) );
                    }
                    TokenType::Char =>
                    { // Получаем размер символа
                      value[i] = Token::new( Some(TokenType::UInt),Some(String::from("1")) );
                    }
                    TokenType::String | TokenType::RawString =>
                    { // Получаем размер строки
                      value[i] = Token::new(
                        Some(TokenType::UInt),
                        Some(
                          p0.getData().unwrap_or_default()
                            .chars().count().to_string()
                        )
                      );
                    }
                    _ =>
                    { // Получаем размер вложений в структуре
                      // Результат только в UInt
                      value[i].setDataType( Some(TokenType::UInt) );
                      // Получаем значение
                      match self.getStructureByName(&p0.getData().unwrap_or_default())
                      {
                        Some(structureLink) =>
                        {
                          value[i].setData(
                            Some(
                              structureLink.read().unwrap()
                                .lines.len().to_string()
                            )
                          );
                        }
                        None =>
                        { // Результат 0 т.к. не нашли такой структуры
                          value[i].setData( Some(String::from("0")) );
                        }
                      }
                    }
                  }
                }
              }
            }
            "input" =>
            { // Получаем результат ввода

              // Результат может быть только String
              value[i].setDataType( Some(TokenType::String) );

              match parameters.getExpression(self,0)
              {
                None => {}
                Some(p0) =>
                {
                  match p0.getData()
                  {
                    None => {}
                    Some(data) =>
                    { // Это может быть выведено перед вводом;
                      // todo: возможно потом это лучше убрать,
                      //       т.к. программист сам может вызвать
                      //       такое через иные методы
                      print!("{}",data);
                      io::stdout().flush().unwrap(); // forced withdrawal of old
                    }
                  }
                }
              }

              let mut valueBuffer: String = String::new(); // временный буффер ввода
              match io::stdin().read_line(&mut valueBuffer)
              { // Читаем ввод
                Ok(_) =>
                { // Успешно ввели и записали
                  value[i].setData(
                    Some( valueBuffer.trim_end().to_string() )
                  );
                }
                Err(_) =>
                { // Не удалось ввести, пустая строка
                  value[i].setData( Some(String::new()) );
                }
              }
            }
            "exec" =>
            { // Запускает что-то и возвращает строковый output работы
              match parameters.getExpression(self,0)
              {
                None => {}
                Some(p0) =>
                {
                  let data: String = p0.getData().unwrap_or_default();
                  let mut parts: SplitWhitespace<'_> = data.split_whitespace();

                  let command: &str      = parts.next().expect("No command found in parameters"); // todo: no errors
                  let    args: Vec<&str> = parts.collect();

                  let output: Output =
                    Command::new(command)
                      .args(&args)
                      .output()
                      .expect("Failed to execute process"); // todo: no errors
                  let outputString: String = String::from_utf8_lossy(&output.stdout).to_string();
                  match !outputString.is_empty()
                  {
                    false => {}
                    true =>
                      { // result
                        value[i].setData    ( Some(outputString.trim_end().to_string()) );
                        value[i].setDataType( Some(TokenType::String) );
                      }
                  }
                  //
                }
              }
              //
            }
            "execs" =>
            { // Запускает что-то и возвращает кодовый результат работы
              // todo: Возможно изменение: Следует ли оставлять вывод stdout & stderr ?
              //       -> Возможно следует сделать отдельные методы для подобных операций.
              match parameters.getExpression(self,0)
              {
                None => {}
                Some(p0) =>
                {
                  let data: String = p0.getData().unwrap_or_default();
                  let mut parts: SplitWhitespace<'_> = data.split_whitespace();

                  let command: &str      = parts.next().expect("No command found in expression"); // todo: no errors
                  let    args: Vec<&str> = parts.collect();

                  let status: ExitStatus =
                    Command::new(command)
                      .args(&args)
                      .stdout(std::process::Stdio::null())
                      .stderr(std::process::Stdio::null())
                      .status()
                      .expect("Failed to execute process"); // todo: no errors
                  value[i].setData    ( Some(status.code().unwrap_or(-1).to_string()) );
                  value[i].setDataType( Some(TokenType::String) );
                }
              }
            }
            _ => { break 'basicMethods; } // Выходим, т.к. ожидается нестандартный метод
          }
          return;
        }
        // Если код не завершился ранее, то далее идут custom методы;
        { // Передаём параметры, они также могут быть None
          self.procedureCall(&structureName, parameters);
          // После чего решаем какой результат оставить
          match self.getStructureByName(&structureName)
          {
            None => {}
            Some(structureLink) =>
            { // По результату структуры, определяем пустой он или нет
              match
              &structureLink.read().unwrap()
                .result
              {
                Some(result) =>
                { // Результат не пустой, значит оставляем его
                  value[i].setData    ( result.getData() );
                  value[i].setDataType( result.getDataType().clone() );
                }
                None =>
                { // Если результата структуры не было,
                  // значит это была действительно процедура
                  value[i].setData    ( None );
                  value[i].setDataType( None );
                }
              }
            }
          }
        }
        //
      }
    }
    //
  }
}