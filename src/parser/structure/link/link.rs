use std::sync::{Arc, RwLock, RwLockReadGuard};
use crate::parser::bytes::Bytes;
use crate::parser::structure::structure::Structure;
use crate::parser::structure::structureType::StructureType;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

/// Чтение ссылок: `a.b.c`, `a[0]`, `a.method()`.
///
/// Модуль вынесен из `structure.rs`, чтобы логика ссылок жила в одном месте.
///
/// Ссылка разбирается в два шага:
///
/// 1. Голова ссылки - это либо имя структуры (`a.b.c`), либо номер линии (`a[0]`).
///    Дальше путь идёт только вниз, по уже найденной структуре.
///
/// 2. В конце ссылки найденная структура либо читается (значение),
///    либо вызывается (были скобки вызова - `parameters`).
///
/// Два разных вида ссылки дают разные ветки, но общий итог у них один:
/// либо значение, либо ссылка, либо вызов, либо пусто.
impl Structure
{
  // ===============================================================================================
  
  /// Получает значение из ссылки на структуру;
  ///
  /// Ссылка на структуру может состоять как из struct name, так и просто из цифр.
  pub fn linkExpression(
    &self, // Текущая структура - текущее пространство.
    currentStructureLink: Option< Arc<RwLock<Self>> >, // Структура предыдущего уровня ссылки.
    link: &mut Vec<String>, // Осталось читать.
    parameters: Option< Vec<Token> >
  ) -> Token
  { // Обработка динамического выражение.
    if link[0].starts_with('[')
    { // Получаем динамическое выражение между [].
      link[0] = format!("{{{}}}", &link[0][1..link[0].len()-1]);
      // Получаем новую строку значения из обработки выражения.
      link[0] = self.formatQuote(link[0].clone());
    }
    // Обработка пути.
    match link[0].parse::<usize>()
    { // Проверяем тип.
      Ok(lineNumber) =>
      { // Цифра в начале ссылки - это номер линии в структуре.
        self.linkByLineNumber(currentStructureLink, link, parameters, lineNumber)
      }
      Err(_) =>
      { // Не цифра - значит это имя структуры.
        self.linkByName(currentStructureLink, link, parameters)
      }
    }
    //
  }

  // ===============================================================================================

  /// Ссылка по номеру линии: `a[0]` - это `lines[0]` структуры `a`.
  ///
  /// Номер относится только к `currentStructureLink`: без него ссылка пустая.
  ///
  /// Дальше по ссылке может идти продолжение (`a[0].b`) - тогда первый токен
  /// линии становится именем структуры, в которой ищем продолжение.
  fn linkByLineNumber(
    &self,
    currentStructureLink: Option< Arc<RwLock<Self>> >, // Структура, в которой ищем линию.
    link: &mut Vec<String>, // Осталось читать.
    parameters: Option< Vec<Token> >, // Аргументы вызова, если это вызов.
    lineNumber: usize // Номер линии.
  ) -> Token
  {
    link.remove(0);

    let currentStructureLock: Arc<RwLock<Self>> =
      match currentStructureLink
      { // Номер линии без структуры - читать нечего.
        Some(currentStructureLock) => currentStructureLock,
        None => return Token::newEmpty(TokenType::None)
      };

    let currentStructure: RwLockReadGuard<Self> = currentStructureLock.read().unwrap();

    let lineTokens: Vec<Token> =
    {
      let lines: &Vec< Arc<RwLock<Line>> > =
        if let Some(lines) = &currentStructure.lines {
          lines
        } else {
          return Token::newEmpty(TokenType::None)
        };
      let line: &Arc<RwLock<Line>> =
        if let Some(line) = lines.get(lineNumber) {
          line
        } else {
          return Token::newEmpty(TokenType::None)
        };
      line.read().unwrap().tokens.clone().unwrap_or_default() // todo плохо
    };

    if lineTokens.is_empty()
    { // В линии нет токенов, нам нечего вычислять.
      return Token::newEmpty(TokenType::None);
    }

    if !link.is_empty()
    { // Продолжение ссылки: `a[0].b` - первый токен линии становится именем.
      let name: String = lineTokens[0].getData().toString().unwrap_or_default();
      link.insert(0, name.clone());

      // Есть ли такая структура во внутреннем пространстве текущей?
      let inside: bool = currentStructure.getStructureByLink(&name).is_some();
      drop(currentStructure);

      return if inside
      { // Идём вниз от найденной структуры.
        currentStructureLock.read().unwrap().linkExpression(None, link, parameters)
      } else
      { // А если такой ссылки там не было, то значит она в self.
        self.linkExpression(Some(currentStructureLock), link, parameters)
      };
    }

    if parameters.is_some()
    { // Вызов метода: `a[0]()`.
      // Внимание: аргументы вызова здесь не прокидываются - как и раньше.
      let mut expressionTokens: Vec<Token> = Self::callTokens(lineTokens[0].getData(), None);
      drop(currentStructure);
      return currentStructureLock.read().unwrap().expression(&mut expressionTokens);
    }

    // Чтение значения из линии.
    if *lineTokens[0].getDataType() == TokenType::Word
    {
      // Если это слово, то это либо ссылка т.к. там много значений в ней;
      // Либо это структура с одиночным вложением и мы можем его забрать сейчас.
      let name: String = lineTokens[0].getData().toString().unwrap_or_default();
      let childTokens: Option<Vec<Token>> =
      {
        if let Some(childStructureLink) = currentStructure.getStructureByLink(&name)
        { // Пробуем проверить что там 1 линия вложена в структуре;
          // После чего сможем посчитать её значение.
          
          let childStructure: RwLockReadGuard<Self> = childStructureLink.read().unwrap();
          // Вложение забираем, только если в текущей структуре ровно одна линия.
          if currentStructure.lines.as_ref().map(|lines| lines.len()) == Some(1) {
            Self::firstLineTokens(childStructure.lines.as_deref())
          } else {
            None
          }
        } else {
          None
        }
      };

      if let Some(mut childTokens) = childTokens
      { // Вложение есть - считаем его значение.
        drop(currentStructure);
        self.expression(&mut childTokens)
      } else
      { // Если ничего не получилось, значит оставляем ссылку.
        drop(currentStructure);
        Token::new( TokenType::Link, lineTokens[0].getData() )
      }
    } else
    { // Если это не слово, то смотрим на результат expression.
      let mut lineTokens: Vec<Token> = lineTokens;
      drop(currentStructure);
      self.expression(&mut lineTokens)
    }
    //
  }

  // ===============================================================================================

  /// Ссылка по имени: `a.b.c` - путь по структурам.
  ///
  /// Голова ссылки ищется вверх по области видимости, остаток пути - строго вниз.
  fn linkByName(
    &self,
    currentStructureLink: Option< Arc<RwLock<Self>> >, // Структура предыдущего уровня ссылки.
    link: &mut Vec<String>, // Осталось читать.
    parameters: Option< Vec<Token> > // Аргументы вызова, если это вызов.
  ) -> Token
  {
    // Голова ссылки (`a` в `a.b`) или голое имя: ищется ВВЕРХ по области видимости.
    let structureLink: Option< Arc<RwLock<Self>> > =
      if let Some(currentStructureLink) = currentStructureLink
      {
        let currentStructure: RwLockReadGuard<Self> = currentStructureLink.read().unwrap();

        // Если в текущем пространстве есть такая структура и у неё есть линии -
        // берём именно её; иначе поднимаемся выше (из блока `? { }` видна и структура выше блока).
        let hasLines: bool =
          if let Some(childStructureLink) = currentStructure.getStructureByScope(&link[0]) {
            childStructureLink.read().unwrap().lines.as_ref()
              .map(|lines| !lines.is_empty()).unwrap_or(false)
          } else {
            false
          };

        if hasLines {
          currentStructure.getStructureByScope(&link[0])
        } else {
          self.getStructureByScope(&link[0])
        }
      } else {
        self.getStructureByScope(&link[0])
      };

    link.remove(0);

    let structureLink: Arc<RwLock<Self>> =
      if let Some(structureLink) = structureLink
      { // Не нашли структуру - пусто.
        structureLink
      } else {
        return Token::newEmpty(TokenType::None)
      };

    if !link.is_empty()
    {
      // Остался ещё путь: если это динамическая библиотека - собираем токен для FFI.
      if let Some(libraryToken) = Self::nativeCallToken(&structureLink, &link[0]) {
        return libraryToken;
      }
      // Иначе - продолжаем чтение ссылки в найденной структуре.
      return self.linkExpression(Some(structureLink), link, parameters);
    }

    // Ссылка закончилась - работаем с найденной структурой.
    let structure: RwLockReadGuard<Self> = structureLink.read().unwrap();
    let lines: &Vec< Arc<RwLock<Line>> > =
      if let Some(lines) = &structure.lines
      { // У структуры нет линий - читать нечего.
        lines
      } else {
        return Token::newEmpty(TokenType::None)
      };

    if let Some(parameters) = parameters
    { // Это вызов метода (скобки были) - запускаем его, для тела любой длины.
      // Метод вызывается в той области, где он объявлен, - это родитель структуры.
      let structureParent: Option< Arc<RwLock<Self>> > = structure.parent.clone();
      let mut expressionTokens: Vec<Token> = Self::callTokens(
        Bytes::from(structure.name.clone().unwrap_or_default()),
        Some(parameters)
      );
      drop(structure);

      return match structureParent
      {
        Some(structureParent) => structureParent.read().unwrap().expression(&mut expressionTokens),
        None => Token::newEmpty(TokenType::None)
      };
    }

    if lines.len() == 1
    { // Чтение одиночного значения (скобок вызова не было): просто выдаём его.
      // По сути это просто 0 линия через expression.
      //
      // todo Требуется уточнение: сюда попадают две разные структуры,
      //  у обеих lines.len() == 1, а ветка рассчитана только на первую.
      //
      //  a.rt:
      //    c = 10                                        # 1. значение
      //    f(name: String) { println(f"Hi, {name}") }    # 2. функция из одной строки
      //
      //  main.rt:
      //    a.c          — 10. Ради этого ветка и написана:
      //                   у конца ссылки берём значение.
      //
      //  Вызовы (`a.f("World")`) сюда не попадают: у них есть `parameters`.
      let mut lineTokens: Vec<Token> =
        if let Some(lineTokens) = Self::firstLineTokens(Some(lines)) {
          lineTokens
        } else {
          return Token::newEmpty(TokenType::None)
        };
      drop(structure);
      return self.expression(&mut lineTokens);
    }

    // Если это просто ссылка, то оставляем её.
    Token::new( TokenType::Link, structure.name.clone().unwrap_or_default() ) // todo плохо.
    //
  }

  // ===============================================================================================

  /// Токены вызова метода: `имя( аргументы )`.
  ///
  /// `arguments` = `None` - вызов без аргументов (пустое вложение).
  fn callTokens(name: Bytes, arguments: Option< Vec<Token> >) -> Vec<Token>
  {
    let mut parametersToken: Token =
      if let Some(arguments) = arguments
      {
        Token::newNesting(vec![
          Arc::new(RwLock::new(Line
          {
            tokens: Some(arguments),
            lines: None
          }))
        ])
      } else {
        Token::newNesting(Vec::new()) // todo: add parameters.
      };
    parametersToken.setDataType( TokenType::CircleBracketBegin );

    vec![
      Token::new( TokenType::Word, name ), // todo плохо.
      parametersToken
    ]
  }

  // ===============================================================================================

  /// Токены первой линии структуры.
  fn firstLineTokens(lines: Option<&[Arc<RwLock<Line>>]>) -> Option<Vec<Token>>
  {
    let line: &Arc<RwLock<Line>> = lines?.first()?;
    Some(line.read().unwrap().tokens.clone().unwrap_or_default()) // todo плохо
  }

  // ===============================================================================================

  /// Токен вызова динамической библиотеки: `lib.method` - путь к библиотеке и имя метода.
  ///
  /// `None` - структура не библиотека, идём дальше по обычному пути.
  /// `Some` - токен готов (пустой, если библиотека не читается).
  fn nativeCallToken(structureLink: &Arc<RwLock<Self>>, methodName: &str) -> Option<Token>
  {
    let structureGuard: RwLockReadGuard<Self> = structureLink.read().unwrap();

    // Если структура имеет тип Pointer - это динамическая библиотека.
    if structureGuard.dataType != StructureType::Pointer {
      return None;
    }

    // Далее извлекаем указатель на библиотеку, сохранённый в lines[0].tokens[0].
    // Библиотека там лежит как токен типа String с путём к файлу библиотеки.
    //
    // Из токена извлекаем сырые байты (путь к библиотеке).
    let bytes: Bytes =
    {
      // Получаем вектор линий структуры (в нашем случае lines[0] хранит токен).
      let linesVec: &Vec< Arc<RwLock<Line>> > = match &structureGuard.lines {
        Some(v) => v,
        None => return Some(Token::newEmpty(TokenType::None))
      };
      // Берём первую линию (индекс 0).
      let lineLock: &Arc<RwLock<Line>> = match linesVec.first() {
        Some(l) => l,
        None => return Some(Token::newEmpty(TokenType::None))
      };
      // Читаем линию, чтобы получить её токены.
      let line: RwLockReadGuard<Line> = lineLock.read().unwrap();
      // Токены линии — здесь должен быть один токен типа String.
      let tokensVec: &[Token] = match &line.tokens {
        Some(t) => t,
        None => return Some(Token::newEmpty(TokenType::None))
      };
      // Берём первый (и единственный) токен.
      let nativeToken: &Token = match tokensVec.first() {
        Some(t) => t,
        None => return Some(Token::newEmpty(TokenType::None))
      };
      // Убеждаемся, что токен действительно типа String.
      if *nativeToken.getDataType() != TokenType::String {
        return Some(Token::newEmpty(TokenType::None));
      }

      nativeToken.getData()
    };
    drop(structureGuard);

    let raw: &[u8] = match bytes.getAll() {
      Some(r) => r,
      None => return Some(Token::newEmpty(TokenType::None))
    };

    // Преобразуем байты в строку (путь).
    let libraryPath: &str = match std::str::from_utf8(raw) {
      Ok(s) => s,
      Err(_) => return Some(Token::newEmpty(TokenType::None))
    };

    // Формируем токен Nesting: одна линия с двумя токенами-строками.
    Some(Token::newNesting(vec![
      Arc::new(RwLock::new(Line {
        tokens: Some(vec![
          // Путь к lib, например: "./libprint.so".
          Token::new(TokenType::String, libraryPath),
          // Имя метода, который вызывают; например: "method" в lib.method(...).
          Token::new(TokenType::String, methodName.to_string()) // todo Но кстати оно больше не надо будет? зачем тогда .clone.
        ]),
        lines: None
      }))
    ]))
    //
  }

  // ===============================================================================================
}

// =================================================================================================
