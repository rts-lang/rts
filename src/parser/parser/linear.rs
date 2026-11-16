use super::condition::replaceConditionExpression;
use super::parser::isMathOperator;
use crate::parser::structure::ffi::bridge;
use crate::parser::structure::primitives::isPrimitiveName;
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::tokenizer::tools::splitByType::splitByType;
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

/// Читает `линейное выражение`; Это запись в одну `Line`.
/// 
/// Например:
/// ```ignore
/// a = 10
/// a := 10+20
/// ```
/// 
/// Обычно состоит оно из левой и правой части выражения;
/// 
/// Хотя правая часть может отсутствовать в некоторых контекстах.
/// 
/// Например:
/// ```ignore
/// a
/// a: U8
/// a~
/// ```
/// 
fn linearStructure(
  lineTokens: &[Token],
  parentLink: Arc<RwLock<Structure>>,
  isBlockValue: bool
) -> bool
{
  // Получаем тип операции.
  let operatorType: TokenType = lineTokens.iter().find_map(|token| {
    match isMathOperator(*token.getDataType()) {
      true => Some(*token.getDataType()),
      false => None,
    }
  }).unwrap_or(TokenType::None);

  // Получаем левую и правую часть.
  let leftValue: Vec<Token>;
  let mut rightValue: Option< Vec<Token> > = None;
  {
    if operatorType == TokenType::None
    { // Операции не было.
      leftValue = std::mem::take(&mut lineTokens.to_owned()); // todo: Тут точно клонирование ?
    }
    else
    {// Операция есть.
      let mut parts: Vec<Line> = splitByType(lineTokens.to_owned(), &[operatorType]); // todo: Тут точно клонирование ?

      leftValue = std::mem::take(&mut parts[0].tokens).unwrap();
      rightValue = std::mem::take(&mut parts[1].tokens);
    }
  }

  let structureName: String;
  let structureMutability: StructureMut;
  let mut structureType: StructureType;
  { // Определяем тип данных у левой части выражения.
    let (structureNameTokens, structureTypeTokens): (Vec<Token>, Option< Vec<Token> >) =
    {
      let mut parts: Vec<Line> = splitByType(leftValue.clone(), &[TokenType::Colon]);
      if parts.len() == 2 {
        (std::mem::take(&mut parts[0].tokens).unwrap(), std::mem::take(&mut parts[1].tokens))
      } else {
        (std::mem::take(&mut parts[0].tokens).unwrap(), None)
      }
    };

    // Определяем тип изменяемости у левой части выражения.
    let structureMutabilityType: StructureMut = 
      if let Some(mutabilityType) = structureNameTokens.get(1)
      { // Если есть флаг изменяемости.
        match mutabilityType.getDataType()
        {
          TokenType::DoubleTilde => StructureMut::Dynamic,
          TokenType::Tilde => StructureMut::Variable,
          _ => return false // Это что-то другое, а не линейное выражение.
        }
      } else if rightValue.is_some() {
        StructureMut::Constant
      } else 
      { // Если нет флага изменяемости и правой части.
        StructureMut::Final
      };

    //
    structureName = structureNameTokens[0].getData().toString().unwrap(); // Имя точно есть.
    structureMutability = structureMutabilityType;
    structureType = if let Some(structureTypeTokens) = structureTypeTokens
    { // `a: U8 | String` — объединение типов (issue #59);
      // Обычный `a: U8` даёт здесь одиночный тип.
      StructureType::fromTypeTokens(&structureTypeTokens)
    } else {
      StructureType::Primitive(PrimitiveType::None)
    }
  };

  drop(leftValue);

  // Ищем структуру с таким именем. Здесь две разные области видимости:
  //
  // - `:=` => Создание в текущем области. Ищем только среди своих структур, наверх не идём.
  //
  // - `=` => Присваивание по области видимости: сначала свои структуры, потом до самого верха.
  //
  let isCreation: bool = operatorType == TokenType::Creation;

  // `:= None`— это явная очистка структуры из текущей области;
  // Это `примитив`, а не выражение.
  //
  // 
  // Не путать с `:= method()`, где `method` уже вернул `None`: 
  // в этом случае структура создаётся со значением `None` для хранения.
  //
  //
  // Различаем по токенам, а не по вычисленному значению:
  // - `a := None` => правая часть = `[Token(None)]` (один литерал) => Удалить.
  //
  // - `a := b` => правая часть = `[ident(b)]` (выражение) => Создать/перезаписать.
  //
  // - `a := method()` => правая часть = `[method, (, )]` => Создать/перезаписать.
  //
  // - `a = None` => оператор `=`, не `:=` => Записать `None` как значение.
  //
  //
  // Это разделение намерений между явным со стороны программиста и 
  // работой состояний внутри программы.
  //
  // Если правая часть пришла из блока-выражения `a := ? False { = 1 }`, то `None` — это
  // результат блока, а не литерал из кода: структуру `a` нужно создать, а не удалить.
  let isCreationNoneLiteral: bool =
    isCreation && !isBlockValue &&
    rightValue.as_ref()
      .map(|tokens| tokens.len() == 1 && *tokens[0].getDataType() == TokenType::None)
      .unwrap_or(false);
  let structureLink: Option< Arc<RwLock<Structure>> > =
  {
    let parent: RwLockReadGuard<Structure> = parentLink.read().unwrap();
    if isCreation
    { // Только свои, строго вниз.
      parent.getStructureByLink(&structureName)
    } else
    { // Свои, потом вверх до самого верха.
      parent.getStructureByScope(&structureName)
    }
  };

  // `:= None`: удаляем найденную локальную структуру.
  //
  //
  // Если структуры нет — ничего не делаем (нечего удалять).
  //
  // Если структура — примитив, то `removeStructure` внутри себя откажется её удалять, 
  // базовая часть языка останется на месте.
  //
  // Это safety-net: даже если проверка `isPrimitiveName` выше пропустит
  // имя, `removeStructure` всё равно проверит по списку примитивов.
  if isCreationNoneLiteral
  {
    if let Some(structureLink) = structureLink
    {
      parentLink.read().unwrap()
        .removeStructure(structureLink);
    }
    return true;
  }

  if let Some(structureLink) = structureLink
  { // Если мы нашли структуру, то значит работаем уже с существующей структурой.
    let parent: RwLockReadGuard<Structure> = parentLink.read().unwrap();

    let structureMut: StructureMut =
    {
      let structure: RwLockReadGuard<Structure> = structureLink.read().unwrap();
      structure.mutable.clone()
    };

    match structureMut
    {
      StructureMut::Constant => {} // Константные структуры изменить нельзя.
      StructureMut::Final | StructureMut::Variable | StructureMut::Dynamic =>
      { // Всё остальное изменить можно.
        
        // `:=` отличается от `=` только тем, где искать структуру.
        //
        // Само присваивание делается то же самое, поэтому `:=` сводим к `=`
        // уже после того, как нужная структура найдена.
        let operator: TokenType = if operatorType == TokenType::Creation {
          TokenType::Equals
        } else {
          operatorType
        };

        parent.structureOp(
          structureLink,
          operator,
          structureMut,
          rightValue.unwrap_or_default()
        );
        //
      }
    }
  } else 
  if isPrimitiveName(&structureName) {
    // Зарезервированное имя — новую структуру не создаём.
  } else 
  if !isCreation && structureType == StructureType::Primitive(PrimitiveType::None) &&
     rightValue.is_some()
  { // `=` без существующей структуры и без объявления типа (`:`) — скип.
    //
    // Это случай именно `a = X` (есть правая часть, но не `:=`): нет структуры =>
    // нечего менять => пропускаем. Создание — это `:=`.
    //
    // Но forward-declaration `a` (без правой части, без `:`, без `~`) — это
    // объявление структуры, которая ждёт первого `=`. Её мы создаём как Final,
    // поэтому условие `rightValue.is_some()` обязательно: иначе `a` без значения
    // пропустилось бы, и потом `a = X` не нашло бы её.
    //
    // `a: U8 = 300` — это объявление с типом, оно создаёт структуру через
    // `structureType != None` ниже. Сюда не попадает.
    //
    // Получается если нет структуры => нечего менять => пропускаем.
  } else
  { // `:=` без существующей структуры, либо `=` с явным типом (`:`),
    // либо forward-declaration `a` без значения — создаём новую.

    // Объединение типов: сам набор вариантов объявления;
    //
    // Хранится отдельно от structureType, потому что в структуре лежит
    // ровно один из вариантов, а объединение продолжает ограничивать
    // все следующие присваивания.
    let unionTypes: Option<Vec<StructureType>> = match structureType
    {
      StructureType::Union(ref variants) => Some(variants.clone()),
      _ => None
    };

    // Считаем правую часть только когда она реально есть;
    //
    // Правая часть бывает не у всех объявлений:
    //
    // - `a` => Final без значения, ждём первого `=` (станет Const);
    //
    // - `a: UInt` => То же с явным типом;
    //
    // - `a~` => Variable без значения, тип выведется при первом присваивании.
    //
    // todo `a~~` еще по идее? нужно проверить отдельно и вписать сюда все варианты.
    let hasValue: bool = rightValue.is_some();

    // Вычисляем правое выражение?
    if hasValue
    {
      // Что ждём от FFI-вызова справа:
      //
      // - `a: Usize = lib.f(x)` => Тип указан, он же тип возврата C-функции
      //   и в него кастуется результат (`castToken` ниже);
      //
      // - `a = lib.f(x)` / `a~~ = lib.f(x)` => Тип слева получаем от правой части;
      //
      // - `a: U8 | String = lib.f(x)` => У объединения нет единственного ABI-типа,
      //   поэтому вызов читается как Infer, а вариант подбирается уже по значению.
      let expect: bridge::FfiExpect =
        if structureType == StructureType::Primitive(PrimitiveType::None) ||
           unionTypes.is_some() ||
           structureMutability == StructureMut::Dynamic
        { bridge::FfiExpect::Infer } else
        { bridge::FfiExpect::Typed(structureType.clone()) };

      // Важно: правую часть считаем под блокировкой чтения, а не записи.
      //
      // Вычисление выражения может искать имя вверх по области видимости
      // (getStructureByScope) — а он берёт read() на каждого родителя, включая
      // текущий. Если бы read() приходился на структуру, которую мы уже держим
      // на записи, то std::sync::RwLock не перевзводим и процесс вставал бы
      // навсегда (0% CPU, sleeping). Старый код (tag 231206) считал ровно так же:
      // сначала expression, потом write на pushStructure.
      //
      // todo rewrite desc, должно быть по смыслу работы.
      let mut value: Token =
      {
        let parent: RwLockReadGuard<Structure> = parentLink.read().unwrap();
        parent.expressionWith(&mut rightValue.unwrap(), &expect)
      };

      // Динамический import(): правая часть выражения — не скаляр, а целая
      // под-структура (модуль со своими подструктурами).
      // 
      // Обычная ветка ниже (inference/castToken/Line{tokens: Some(vec![value])}) 
      // рассчитана на скаляр, поэтому переносим (dataType, lines, structures) модуля
      // напрямую в новую структуру и выходим раньше.
      //
      // todo rewrite desc (в целом, эти абзацы не информативные).
      if *value.getDataType() == TokenType::Link
      {
        if let Some(markerName) = value.getData().toString()
        {
          if let Ok(moduleId) = markerName.parse::<usize>()
          {
            let moduleLink: Option< Arc<RwLock<Structure>> > =
            {
              let parent: RwLockReadGuard<Structure> = parentLink.read().unwrap();
              parent.getModuleById(moduleId)
            };
            if let Some(moduleLink) = moduleLink
            { // Если это модуль.
              
              let moduleGuard: RwLockReadGuard<Structure> = moduleLink.read().unwrap();
              let newStructureLink: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
                Some(structureName),
                structureMutability,
                moduleGuard.dataType.clone(),
                moduleGuard.lines.clone(),
                None
              )));
              *newStructureLink.read().unwrap().structures.write().unwrap() =
                moduleGuard.structures.read().unwrap().clone();
              drop(moduleGuard);

              parentLink.write().unwrap()
                .pushStructure(newStructureLink);
              return true;
              
              //
            }
          }
          //
        }
      }

      if let Some(variants) = unionTypes.clone()
      { // Объединение типов (issue #59): под значение выбирается один вариант.
        if structureMutability == StructureMut::Dynamic
        { // При полной мутабельности `~~` тип всё равно свободно меняется,
          // поэтому объединение можно указать, но оно ничего не ограничивает (#22).
          structureType = value.getStructureType();
        } else
        { // todo сюда desc бы?
          structureType = StructureType::Union(variants).normalizeUnion(&mut value);
        }
      } else
      if structureType == StructureType::Primitive(PrimitiveType::None)
      { // Тип вычисляется если он не был изначально определён;
        // Вычисляется он по типу из результата правой части выражения.
        structureType = value.getStructureType();
      } else
      if structureMutability == StructureMut::Dynamic
      { // Тип вычисляется, если флаг изменяемости Dynamic;
        // Вычисляется он по типу из результата правой части выражения.
        structureType = value.getStructureType();
      } else 
      { // Требуется выполнить преобразование в указанный тип данных.
        structureType.castToken(&mut value);
      }

      //
      rightValue = Some(vec![value]);
    }

    // Создаём структуру.
    let newStructureLink: Arc<RwLock<Structure>> = Arc::new(RwLock::new(Structure::new(
      Some(structureName),
      structureMutability,
      structureType.clone(),
      Some(vec![
        Arc::new(RwLock::new(Line {
          tokens: rightValue.clone(),
          lines: None
        }))
      ]),
      None
    )));

    // Сохраняем объединение, чтобы оно ограничивало и следующие присваивания (#59).
    if let Some(variants) = unionTypes
    {
      newStructureLink.write().unwrap().unionTypes = Some(variants);
    }

    // ABI-композит String: .pointer/.length поверх исходного токена.
    // Primitive(Text) тоже String на уровне ABI (stype при этом показывает примитив).
    if structureType.abiType() == StructureType::Primitive(PrimitiveType::String)
    {
      if let Some(valueToken) = rightValue.as_ref().and_then(|tokens| tokens.first())
      {
        if let Some(fields) = bridge::stringFields(valueToken)
        {
          let newStructure: RwLockReadGuard<Structure> = newStructureLink.read().unwrap();
          for field in fields
          {
            newStructure.pushStructure(field);
          }
        }
        //
      }
    }

    // Блокировку на запись берём только здесь — уже после того, как правая
    // часть посчитана. Так ничего не держит родителя на записи, пока
    // вычисляется выражение, и поиск вверх по области видимости остаётся
    // безопасным.
    parentLink.write().unwrap()
      .pushStructure(newStructureLink);
  }

  //
  true
}

// =================================================================================================

/// Обёртка над чтением `линейного выражения`.
/// 
/// Блок-выражение в правой части `a := ? True { = 20 }`
/// предварительно заменяется своим результатом.
pub(super) fn linearLine(
  lineTokens: &[Token],
  lineLines: &Option< Vec< Arc<RwLock<Line>> > >,
  parentLink: Arc<RwLock<Structure>>
) -> bool
{
  match replaceConditionExpression(lineTokens, lineLines, &parentLink)
  {
    Some(tokens) => linearStructure(&tokens, parentLink, true),
    None => linearStructure(lineTokens, parentLink, false)
  }
}

// =================================================================================================
