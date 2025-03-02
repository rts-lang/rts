use std::io;
use std::io::Write;
use std::process::{Command, ExitStatus, Output};
use std::str::SplitWhitespace;
use std::sync::RwLockReadGuard;
use rand::Rng;
use crate::parser::structure::parameters::Parameters;
use crate::parser::structure::Structure;
use crate::tokenizer::token::{Token, TokenType};

// Procedure =======================================================================================
/// Это набор базовых функций
struct Function {}
impl Function
{
  // ===============================================================================================
  /// Возвращает тип данных переданной структуры
  fn _type(structure: &Structure, parameters: &Parameters, value: &mut Vec<Token>, i: usize)
  {
    match parameters.getExpression(structure,0)
    { None => {} Some(p0) =>
    {
      value[i].setDataType( TokenType::String );
      value[i].setData( Some(p0.getDataType().to_string()) );
    }}
  }
  // ===============================================================================================
  /// Возвращает уровень модификации переданной структуры
  fn _mut(structure: &Structure, parameters: &Parameters, value: &mut Vec<Token>, i: usize)
  {
    match parameters.get(0)
    { None => {} Some(p0) =>
    {
      value[i].setDataType( TokenType::String );
      let result: String =
        match p0.getData()
        {
          None => String::from(""),
          Some(structureName) =>
          { // Получили название структуры
            match structure.getStructureByName(&structureName)
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
    }}
  }
  // ===============================================================================================
  /// Возвращаем случайное число типа UInt от min до max
  fn randUInt(structure: &Structure, parameters: &Parameters, value: &mut Vec<Token>, i: usize)
  {
    if !parameters.isNone() // todo оставить либо это, либо снизу нули
    {
      let min: usize =
        match parameters.getExpression(structure,0)
        { None => 0, Some(p0) =>
        {
          match p0.getData()
          {
            Some(expressionData) => expressionData.parse::<usize>().unwrap_or_default(),
            None => 0
          }
        }};
      let max: usize =
        match parameters.getExpression(structure,1)
        { None => 0, Some(p1) =>
        {
          match p1.getData()
          {
            Some(expressionData) => expressionData.parse::<usize>().unwrap_or_default(),
            None => 0
          }
        }};
      let randomNumber: usize =
        match min < max
        {
          true  => rand::thread_rng().gen_range(min..=max),
          false => 0
        };
      value[i].setDataType( TokenType::UInt );
      value[i].setData    ( Some(randomNumber.to_string()) );
    }
  }
  // ===============================================================================================
  /// Получаем размер структуры
  fn len(structure: &Structure, parameters: &Parameters, value: &mut Vec<Token>, i: usize)
  {
    match parameters.getExpression(structure,0)
    { None => {} Some(p0) =>
    {
      match *p0.getDataType()
      {
        TokenType::None =>
        { // Результат 0
          value[i] = Token::new( TokenType::UInt, Some(String::from("0")) );
        }
        TokenType::Char =>
        { // Получаем размер символа
          value[i] = Token::new( TokenType::UInt, Some(String::from("1")) );
        }
        TokenType::String | TokenType::RawString =>
        { // Получаем размер строки
          value[i] = Token::new(
            TokenType::UInt,
            Some(
              p0.getData().unwrap_or_default()
                .chars().count().to_string()
            )
          );
        }
        _ =>
        { // Получаем размер вложений в структуре
          // Результат только в UInt
          value[i].setDataType( TokenType::UInt );
          // Получаем значение
          match structure.getStructureByName(&p0.getData().unwrap_or_default())
          {
            Some(structureLink) =>
            {
              value[i].setData(
                Some(
                  // Получаем количество линий структуры
                  match &structureLink.read().unwrap().lines
                  {
                    None => String::from("0"),
                    Some(lines) =>
                    {
                      lines.len().to_string()
                    }
                    //
                  }
                )
                //
              );
            }
            None =>
            { // Результат 0 т.к. не нашли такой структуры
              value[i].setData( Some(String::from("0")) );
            }
          }
          //
        }
      }
      //
    }}
  }
  // ===============================================================================================
  /// Получаем результат ввода
  fn input(structure: &Structure, parameters: &Parameters, value: &mut Vec<Token>, i: usize)
  {
    // Результат может быть только String
    value[i].setDataType( TokenType::String );

    match parameters.getExpression(structure,0)
    { None => {} Some(p0) =>
    {
      match p0.getData()
      { None => {} Some(data) =>
      { // Это может быть выведено перед вводом;
        // todo: возможно потом это лучше убрать,
        //       т.к. программист сам может вызвать
        //       такое через иные методы
        print!("{}",data);
        io::stdout().flush().unwrap(); // forced withdrawal of old
      }}
    }}

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
  // ===============================================================================================
  /// Запускает что-то и возвращает строковый output работы
  fn exec(structure: &Structure, parameters: &Parameters, value: &mut Vec<Token>, i: usize)
  {
    match parameters.getExpression(structure,0)
    { None => {} Some(p0) =>
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
      { false => {} true =>
      { // result
        value[i].setData    ( Some(outputString.trim_end().to_string()) );
        value[i].setDataType( TokenType::String );
      }}
      //
    }}
    //
  }
  // ===============================================================================================
  /// Запускает что-то и возвращает кодовый результат работы
  /// todo: Возможно изменение: Следует ли оставлять вывод stdout & stderr ?
  ///       -> Возможно следует сделать отдельные методы для подобных операций.
  fn execs(structure: &Structure, parameters: &Parameters, value: &mut Vec<Token>, i: usize)
  {
    match parameters.getExpression(structure,0)
    { None => {} Some(p0) =>
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
      value[i].setDataType( TokenType::String );
    }}
  }
}

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
      // ===========================================================================================
      None =>
      { // Вариант в котором тип токена может быть типом данных => это cast в другой тип;
        match *value[i].getDataType()
        {
          TokenType::UInt =>
          { // Получаем значение выражения в типе
            // todo: Float, UFloat
            match parameters.getExpression(self,0)
            { None => {} Some(p0) =>
            {
              value[i].setDataType( TokenType::UInt );
              value[i].setData( Some(p0.getData().unwrap_or_default()) );
            }}
          }
          TokenType::Int =>
          { // Получаем значение выражения в типе
            match parameters.getExpression(self,0)
            { None => {} Some(p0) =>
            {
              value[i].setDataType( TokenType::Int );
              value[i].setData( Some(p0.getData().unwrap_or_default()) );
              //
            }}
            //
          }
          TokenType::String =>
          { // Получаем значение выражение в типе String
            // todo: подумать над formatted типами
            match parameters.getExpression(self,0)
            { None => {} Some(p0) =>
            {
              value[i].setDataType( TokenType::String  );
              value[i].setData( Some(p0.getData().unwrap_or_default()) );
              //
            }}
            //
          }
          TokenType::Char =>
          { // Получаем значение выражения в типе Char
            // todo: проверить работу
            match parameters.getExpression(self,0)
            { None => {} Some(p0) =>
            {
              value[i].setDataType( TokenType::Char );
              value[i].setData(
                Some(
                  (p0.getData().unwrap_or_default()
                    .parse::<u8>().unwrap() as char
                  ).to_string()
                )
              );
              //
            }}
            //
          }
          _ => {} // todo: Возможно custom варианты преобразований из custom ?
        }
      }
      // ===========================================================================================
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

            "type" => Function::_type(self, &parameters, value, i),
            "mut" => Function::_mut(self, &parameters, value, i),
            "randUInt" => Function::randUInt(self, &parameters, value, i),
            "len" => Function::len(self, &parameters, value, i),
            "input" => Function::input(self, &parameters, value, i),
            "exec" => Function::exec(self, &parameters, value, i),
            "execs" => Function::execs(self, &parameters, value, i),
            _ => { break 'basicMethods; } // Выходим, ожидается нестандартный метод
          }
          return;
        }
        // =========================================================================================
        // Если код не завершился ранее, то далее идут custom методы;
        { // Передаём параметры, они также могут быть None
          self.procedureCall(&structureName, parameters);
          // После чего решаем какой результат оставить
          match self.getStructureByName(&structureName)
          { None => {} Some(structureLink) =>
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
                value[i].setDataType( TokenType::None );
              }
            }
          }}
        }
        //
      }
    }
    //
  }
}