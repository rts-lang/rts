use crate::parser::parser::readLines;
use crate::parser::structure::ffi::scopeStack;
use crate::parser::structure::methods::parameters::Parameters;
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::tokenizer::tokenizer::readTokensSimple;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use crate::{_filePath, _sourcePath};
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

/// Это набор базовых функций.
struct Function;

impl Function
{
  // ===============================================================================================
  
  /// Возвращает тип данных выражения.
  fn _type(
    structure: &Structure, 
    parameters: &Parameters, 
    value: &mut [Token], 
    i: usize
  ) -> ()
  {
    if parameters.isNone()
    {
      value[i].setDataType(TokenType::None);
      value[i].setData(None);
    } else
    {
      if let Some(parameter0) = parameters.getExpression(structure, 0)
      {
        value[i].setDataType( TokenType::String );
        value[i].setData( parameter0.getDataType().to_string() );
      } else 
      {
        value[i].setDataType(TokenType::None);
        value[i].setData(None);
      }
    }
    //
  }
  
  /// Возвращает тип данных структуры.
  fn stype(
    structure: &Structure, 
    parameters: &Parameters, 
    value: &mut [Token], 
    i: usize
  ) -> ()
  {
    if parameters.isNone()
    {
      value[i].setDataType(TokenType::None);
      value[i].setData(None);
    } else
    {
      if let Some(parameter0Link) = parameters.get(0)
      { // Получаем 0 параметр.
        let parameter0: RwLockReadGuard<Line> = parameter0Link.read().unwrap();
        if let Some(tokens) = &parameter0.tokens
        { // Получаем список токенов.
      
          let token: &Token = tokens.first().unwrap(); // Получаем 0 токен.
          
          let structureName: String = token.getData().toString().unwrap_or_default();
          if !structureName.is_empty()
          { // Ищем структуру.
            if let Some(structureLink) = structure.getStructureByScope(&structureName)
            { // Это custom structure.
              let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
              value[i].setDataType(TokenType::String);
              value[i].setData(structure.dataType.to_string());
            } else
            {
              // Структуры с таким именем нет - значит это значение.
              // Отвечаем его типом: так же, как тип структуры, в которую оно попало бы:
              // `5` -> U8, `"text"` -> String.
              let mut tokenCopy: Token = token.clone();
              let structureType: StructureType = StructureType::naturalType(&mut tokenCopy);

              match structureType
              {
                StructureType::Primitive(PrimitiveType::None) =>
                { // Типа данных нет - такого значения не бывает.
                  value[i].setDataType(TokenType::None);
                  value[i].setData(None);
                }
                _ =>
                {
                  value[i].setDataType(TokenType::String);
                  value[i].setData(structureType.to_string());
                }
              }
              //
            }
          }
          //
        }
      }
      //
    }
  }
  
  /// Возвращает полный union-тип структуры (`a: U8 | 10` → `U8 | 10`).
  ///
  /// Если структура объявлена без объединения — совпадает со `stype`.
  /// `unionTypes` хранит исходный набор вариантов и не сужается при присваивании.
  /// 
  /// todo Здесь система отличается немного от других, а еще нет комментов по частям кода.
  ///   Я думаю тут просто у прощена в плане реализации, что стоит применить stype, type ...
  fn utype(
    structure: &Structure,
    parameters: &Parameters,
    value: &mut [Token],
    i: usize
  ) -> ()
  {
    if parameters.isNone()
    {
      value[i].setDataType(TokenType::None);
      value[i].setData(None);
      return;
    }

    if let Some(parameter0Link) = parameters.get(0)
    {
      let parameter0: RwLockReadGuard<Line> = parameter0Link.read().unwrap();
      if let Some(tokens) = &parameter0.tokens
      {
        let token: &Token = tokens.first().unwrap();
        let structureName: String = token.getData().toString().unwrap_or_default();
        if !structureName.is_empty()
        {
          if let Some(structureLink) = structure.getStructureByScope(&structureName)
          {
            let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
            value[i].setDataType(TokenType::String);
            value[i].setData(structure.utype().to_string());
            return;
          }
        }
      }
    }

    value[i].setDataType(TokenType::None);
    value[i].setData(None);
  }

  // ===============================================================================================
  
  /// Возвращает уровень модификации переданной структуры.
  /// 
  /// todo Может проверять несколько параметров и возвращать список
  fn _mut(
    structure: &Structure, 
    parameters: &Parameters, 
    value: &mut [Token], 
    i: usize
  ) -> ()
  {
    if let Some(parameter0Link) = parameters.get(0)
    { // Получаем 0 параметр.
      
      let parameter0: RwLockReadGuard<Line> = parameter0Link.read().unwrap();
      if let Some(tokens) = &parameter0.tokens
      { // Получаем список токенов.

        let token: &Token = tokens.first().unwrap(); // Получаем 0 токен.
        
        value[i].setDataType( TokenType::String );
        let result: String = if let Some(structureName) = token.getData().toString()
        {
          // Получили название структуры.
          if let Some(structureLink) = structure.getStructureByScope(&structureName)
          {
            // Получили ссылку на структуру.
            let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
            structure.mutable.to_string()
          } else {
            String::from("")
          }
        } else {
          String::from("")
        };
        value[i].setData(result);
        //
      }
      //
    }
    //
  }
  
  // ===============================================================================================
  
  /// Получаем размер структуры.
  fn len(
    structure: &Structure, 
    parameters: &Parameters, 
    value: &mut [Token], 
    i: usize
  ) -> ()
  {
    if let Some(parameter0) = parameters.getExpression(structure,0)
    {
      match *parameter0.getDataType()
      {
        TokenType::None =>
        { // Результат 0.
          value[i] = Token::new( TokenType::UInt, String::from("0") );
        }
        TokenType::Char =>
        { // Получаем размер символа.
          value[i] = Token::new( TokenType::UInt, String::from("1") );
        }
        TokenType::String | TokenType::RawString =>
        { // Получаем размер строки.
          value[i] = Token::new(
            TokenType::UInt,
            parameter0.getData().toString().unwrap_or_default()
              .chars().count().to_string()
          );
        }
        _ =>
        { // Получаем размер вложений в структуре
          // Результат только в UInt.
          value[i].setDataType( TokenType::UInt );
          // Получаем значение
          if let Some(structureLink) = 
            structure.getStructureByScope( &parameter0.getData().toString().unwrap_or_default() )
          {
            value[i].setData(
              // Получаем количество линий структуры.
              if let Some(lines) = &structureLink.read().unwrap().lines {
                lines.len().to_string()
              } else { String::from("0") }
              //
            );
          } else
          { // Результат 0 т.к. не нашли такой структуры.
            value[i].setData( String::from("0") );
          }
          //
        }
      }
      //
    }
  }
  
  // ===============================================================================================
  
  /// importNative(path) — загружает native-библиотеку в текущий FFI-scope.
  ///
  /// - Вызов сам по себе является FFI-операцией: если мы внутри `[ffi] { ... }`
  ///   блока, он лениво создаёт/удерживает FFIScope (ensureFfiScope) и грузит
  ///   библиотеку в него. Native живёт на весь блок и дропается при выходе.
  /// 
  /// - Возвращает путь как String-токен; при присваивании `lib: Pointer = ...`
  ///   структура получает тип Pointer и дальше используется как native handle.
  /// 
  /// - Вне `[ffi]` блока путь всё равно возвращается (можно хранить/передавать),
  ///   но реальная загрузка и вызовы методов работают только внутри блока
  ///   (либо через временный scope на стороне callExternal — legacy fallback).
  /// 
  /// - Если загрузка внутри блока падает — importNative выдаст `None`.
  pub fn importNative(
    structure: &Structure, 
    parameters: &Parameters, 
    value: &mut [Token], 
    i: usize
  ) -> ()
  {
    if let Some(parameter0) = parameters.getExpression(structure, 0)
    {
      let libraryPath: String = parameter0.getData().toString().unwrap_or_default();
      if libraryPath.is_empty() 
      {
        value[i].setDataType(TokenType::None);
        return;
      }

      // Путь должен быть относительно запущенного файла.
      let libraryPath: String = 
        if libraryPath.contains('/') && !std::path::Path::new(&libraryPath).is_absolute()
        {
          std::path::Path::new(_filePath.get().unwrap())
            .parent()
            .map(|dir| dir.join(&libraryPath).to_string_lossy().into_owned())
            .unwrap_or(libraryPath)
        } else { libraryPath };

      #[cfg(not(target_family = "wasm"))]
      {
        // importNative — FFI-вызов. Если мы внутри [ffi]-блока, ensureFfiScope 
        // создаёт/удерживает scope, а load проверяет, что библиотека реально открывается.
        // 
        // Ошибка загрузки → None (import «падает» вместе с блоком).
        // 
        // Вне блока просто возвращаем путь — handle можно хранить, 
        // но вызовы без живого scope не сработают
        // (или пойдут через legacy temp-scope в callExternal).
        let loadOk: bool = match scopeStack::withCurrentFfiScope(|scope| {
          scope.load(&libraryPath)
            .map(|_| ())
            .map_err(|e| e.to_string())
        })
        {
          // Мы внутри [ffi]-блока: load прошёл или нет.
          Some(Ok(())) => true,
          Some(Err(_)) => false,
          // Вне [ffi]-блока: scope нет — путь всё равно отдаём (можно
          // сохранить в структуру и использовать позже внутри блока).
          None => true
        };

        if loadOk
        {
          value[i].setDataType(TokenType::String);
          value[i].setData(libraryPath);
        } else
        {
          value[i].setDataType(TokenType::None);
          value[i].setData(None);
        }
      }

      // Если мы компилируем под WebAssembly, динамическая загрузка .so невозможна.
      //
      // todo Это нужно будет решить
      #[cfg(target_family = "wasm")]
      {
        value[i].setDataType(TokenType::None);
      }
      //
    } else {
      value[i].setDataType(TokenType::None);
    }
    //
  }

  // ===============================================================================================

  /// Динамический импорт `.rt` файла как структуры с подструктурами.
  /// 
  ///
  /// В отличие от статического import (Разрешается до запуска, в фиксированной точке), 
  /// `import(...)` — обычная функция: путь может быть вычислен в рантайме, а сам вызов 
  /// может стоять внутри условия. цикла. функции и выполнится ровно в момент, 
  /// когда до него дойдёт исполнение.
  ///
  /// Файл читается, токенизируется и выполняется как отдельная изолированная структура;
  /// Он клонирует поведение main структуры - но не создает новую.
  /// Это также полезно, чтобы не дублировать `структуры-примитивы`.
  /// 
  /// 
  /// Объявления верхнего уровня: структуры становятся подструктурами 
  /// результата и доступны через `.` точно так же, как обычные вложенные структуры.
  /// 
  ///
  /// Относительный путь считается от файла, в котором написан сам import(),
  /// а не от запущенного файла. Модуль указывает соседа, глядя на собственную папку, 
  /// и ему не нужно знать, где лежит точка входа.
  /// 
  /// Папку с модулем можно перенести целиком — в подпапку, в другой проект,
  /// в общее хранилище пакетов — и её внутренние import() продолжат работать.
  /// 
  /// Если считать от запущенного файла, то путь внутри модуля зависит от того,
  /// кто его подключил, и один и тот же модуль ломается при подключении из
  /// разных мест. Абсолютные пути остаются как есть.
  ///
  /// Файл, код которого исполняется сейчас, хранится в `_sourcePath`:
  /// его ставит import() на время загрузки модуля, и вызов функции из модуля
  /// (см. procedureCall). Структуры запоминают его при создании.
  pub fn import(
    structure: &Structure, 
    parameters: &Parameters, 
    value: &mut [Token], 
    i: usize
  ) -> ()
  {
    if let Some(parameter0) = parameters.getExpression(structure, 0)
    {
      let importPath: String = parameter0.getData().toString().unwrap_or_default();
      if importPath.is_empty() 
      {
        value[i].setDataType(TokenType::None);
        return;
      }

      // Путь — относительно файла, в котором написан этот import().
      let importPath: String =
        if !std::path::Path::new(&importPath).is_absolute()
        {
          let currentFile: String = _sourcePath.with(|sourcePath|
          {
            if let Some(sourcePath) = sourcePath.borrow().as_ref()
            {
              sourcePath.as_str().to_string()
            } else
            { // Код запущенного файла.
              _filePath.get().unwrap().clone()
            }
          });
          std::path::Path::new(&currentFile)
            .parent()
            .map(|dir| dir.join(&importPath).components().collect::<std::path::PathBuf>())
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| importPath.clone())
        } else { importPath };

      let mut buffer: Vec<u8> = match std::fs::read(&importPath)
      {
        Ok(data) => data,
        Err(_) =>
        {
          value[i].setDataType(TokenType::None);
          return;
        }
      };

      // Токенизируем содержимое файла как отдельную независимую программу.
      let moduleLines: Vec<Arc<RwLock<Line>>> = readTokensSimple(&mut buffer);

      // Уникальный id для временной подструктуры в текущей области видимости -
      // нужно, чтобы найти её обратно.
      let moduleId: usize =
      {
        let structuresLink: RwLockReadGuard<Option<
          Vec< Arc<RwLock<Structure>> >
        >> = structure.structures.read().unwrap();
        structuresLink.as_ref().map(|v| v.len()).unwrap_or(0)
      };

      // Модуль — изолированная структура по аналогии main структуры,
      // чтобы импортированный файл не видел структуры вызывающей стороны.
      //
      // Имя не нужно: lookup идёт по `StructureType::Module(id)`;
      // Это правильнее т.к. поиск по имени структуры может выйти за `self`,
      // а `id` не позволяет это сделать.
      //
      // Пока модуль загружается, исполняется код его файла:
      // структуры модуля запомнят его путь при создании.
      let previousSourcePath: Option<Arc<String>> =
        _sourcePath.with(|sourcePath| sourcePath.borrow().clone());
      _sourcePath.with(|sourcePath|
        *sourcePath.borrow_mut() = Some(Arc::new(importPath))
      );

      let moduleLink: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
        None,
        StructureMut::Constant,
        StructureType::Module(moduleId),
        Some(moduleLines),
        None
      )));

      // Выполняем верхний уровень файла прямо сейчас — это и есть
      // динамический импорт: объявления файла (структуры)
      // пушатся в `moduleLink.structures` по мере исполнения его строк,
      // так же, как это происходит при обычном запуске кода.
      readLines(moduleLink.clone());
      _sourcePath.with(|sourcePath|
        *sourcePath.borrow_mut() = previousSourcePath
      );

      structure.pushStructure(moduleLink);

      // Link несёт id модуля — его найдут через `getModuleById`.
      value[i].setDataType(TokenType::Link);
      value[i].setData(moduleId.to_string());
    } else {
      value[i].setDataType(TokenType::None);
    }
    //
  }

  // ===============================================================================================

  /// Приводит значение к границам [`StructureType`]: `U8(a)`/`I16(a)`/`F32(a)`/`Usize(a)`/...
  ///
  /// 
  /// Это метод-помощник: берёт токен и возвращает новый токен в границах [`StructureType`];

  /// Границы те же, что при объявлении `a: U8 = ...`: считает их [`Structure::normalizeToken`].
  /// 
  ///
  /// Результат остаётся токеном и работает в выражении: `c := U8(a) + b`;
  /// 
  /// Тип данных дальше не несёт - его хранит структура: `x: U8 = a`.
  /// 
  ///
  /// Тип приведения - любой примитив из [`PrimitiveType`]: числовые зажимаются
  /// по своим границам, строки становятся строками, `None` - это отсутствие
  /// значения;
  /// 
  /// todo `Any`/`Pointer` и некоторые другие ничего не меняют - приведения для них пока нет.
  fn cast(
    structure: &Structure,
    parameters: &Parameters,
    value: &mut [Token],
    i: usize,
    structureType: StructureType
  ) -> ()
  {
    match parameters.getExpression(structure, 0)
    {
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      None =>
      { // Нет аргумента - нет значения.
        value[i].setDataType(TokenType::None);
        value[i].setData(None);
      }

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      
      Some(mut token) =>
      {
        if structureType == StructureType::Primitive(PrimitiveType::None)
        { // `None` - это отсутствие значения, а не значение.
          value[i].setDataType(TokenType::None);
          value[i].setData(None);
        } else
        if structureType == StructureType::Primitive(PrimitiveType::String) ||
           structureType == StructureType::Primitive(PrimitiveType::RawString)
        { // Значение становится строкой: `String(5)` - это "5".
          value[i].setDataType(
            if structureType == StructureType::Primitive(PrimitiveType::String)
            { TokenType::String } else { TokenType::RawString }
          );
          value[i].setData( token.getData().toString().unwrap_or_default() );
        } else
        { // Остальные примитивы приводятся по своим границам.
          Structure::normalizeToken(&mut token, structureType);
          value[i].setDataType(*token.getDataType());
          value[i].setData(token.getData());
        }
      }

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    }
  }

  // ===============================================================================================
}

// =================================================================================================

impl Structure
{
  // ===============================================================================================
  
  /// Запускает функцию;
  ///
  /// Функция - это такая структура, которая возвращает значение.
  ///
  /// Но кроме того, запускает не стандартные методы;
  /// В нестандартных методах могут быть процедуры, которые не вернут результат.
  /// 
  /// 
  /// todo: вынести все стандартные варианты в отдельный модуль
  ///
  /// todo: когда будет вынесено, то должна ожидать тип данных, который должен в Tokenizer::getWord() тоже
  pub fn functionCall(&self, value: &mut Vec<Token>, valueLength: &mut usize, i: usize) -> ()
  {
    let parameters: Parameters = self.getCallParameters(value, i, valueLength);

    // ---------------------------------------------------------------------------------------------
    if let Some(structureName) = value[i].getData().toString()
    { // Вариант в котором это обращение к стандартной или custom функции.
      //
      // todo: проверка на нижний регистр

      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      // Далее идут базовые методы;
      // Эти методы ожидают аргументов.
      'basicMethods:
      { // Это позволит выйти, если мы ожидаем не стандартные варианты.
        // Методы приведения к StructureType: `U8(a)`, `I16(a)`, `F32(a)`, `Usize(a)`.
        //
        // Типы берутся из `PrimitiveType` - реестра примитивов языка,
        // поэтому привести значение можно к любому примитивному типу.
        if let Some(structureType) = PrimitiveType::fromName(structureName.as_str())
        {
          Function::cast(self, &parameters, value, i, StructureType::Primitive(structureType.clone()));
          return;
        }
        //
        match structureName.as_str()
        { // Проверяем на сходство стандартных функций.

          // todo: создать resultType() ?
          //       для возвращения результата ожидаемого структурой

          "type" => Function::_type(self, &parameters, value, i),
          "stype" => Function::stype(self, &parameters, value, i),
          "utype" => Function::utype(self, &parameters, value, i),
          "mut" => Function::_mut(self, &parameters, value, i),
          "len" => Function::len(self, &parameters, value, i),
          "importNative" => Function::importNative(self, &parameters, value, i),
          "import" => Function::import(self, &parameters, value, i),
          _ => break 'basicMethods // Выходим, ожидается нестандартный метод.
        }
        return;
      }
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
      // Если код не завершился ранее, то далее идут custom методы.

      // Передаём параметры, они также могут быть None
//        println!("? {} - parameters: {:?}",structureName,parameters.get(0).unwrap().tokens);
//        println!("  > A1 {:?}",parameters.getAllExpressions(self).unwrap_or_default());
      self.procedureCall(&structureName, parameters);
      // После чего решаем какой результат оставить.
      if let Some(structureLink) = self.getStructureByScope(&structureName)
      { // По результату структуры, определяем пустой он или нет.
        if let Some(result) = &structureLink.read().unwrap().result
        { // Результат не пустой, значит оставляем его.
          value[i].setData(result.getData());
          value[i].setDataType(*result.getDataType());
        } else
        { // Если результата структуры не было,
          // значит это была действительно процедура.
          value[i].setData(None);
          value[i].setDataType(TokenType::None);
        }
      }
      // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    } else
    // ---------------------------------------------------------------------------------------------
    { // Тип токена известен, а его текст - нет.
      //
      // Имя примитива тут не узнать - но оно и не нужно: `None` - это не имя,
      // а само значение, и примитив по нему один.
      if *value[i].getDataType() == TokenType::None
      {
        Function::cast(self, &parameters, value, i, StructureType::Primitive(PrimitiveType::None));
        return;
      }
      //
    }
    // ---------------------------------------------------------------------------------------------
  }
  
  // ===============================================================================================
}

// =================================================================================================
