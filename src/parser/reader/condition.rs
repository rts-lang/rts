use super::reader::readStructureLines;
use crate::parser::structure::structure::{Structure, StructureMut};
use crate::parser::structure::structureType::{PrimitiveType, StructureType};
use crate::tokenizer::types::line::Line;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
use std::sync::{Arc, RwLock, RwLockReadGuard};
// =================================================================================================

/// Одна ветка условной цепочки: `? условие { тело }`.
///
/// `condition == None` — это `else` (`? { тело }`), его условие всегда `True`.
struct ConditionBranch
{
  /// Токены условия.
  condition: Option< Vec<Token> >,
  
  /// Линии блока (структуры).
  body: Option< Vec< Arc<RwLock<Line>> > >
}

// =================================================================================================

/// Читает ветки `elif`/`else` ниже текущей линии: линии, которые начинаются с `?`.
///
/// `else` (`? {}`) завершает цепочку: линия `?` после него начинает новую цепочку.
/// 
/// Сдвигает `lineIndex` на количество прочитанных линий, чтобы их не читали повторно.
fn collectConditionBranches(
  parentLink: &Arc<RwLock<Structure>>
) -> Vec<ConditionBranch>
{
  let mut branches: Vec<ConditionBranch> = Vec::new();
  {
    let parent: RwLockReadGuard<Structure> = parentLink.read().unwrap();
    let Some(lines) = &parent.lines else { return branches };
    let start: usize = parent.lineIndex.get();
    for lineLink in lines.iter().skip(start + 1)
    {
      let line: RwLockReadGuard<Line> = lineLink.read().unwrap();
      let Some(tokens) = &line.tokens else { break };
      if tokens.is_empty() || *tokens[0].getDataType() != TokenType::Question
      { // Выходим, если в начале линии нет TokenType::Question.
        break;
      }
      // Если в токенах просто TokenType::Question, значит это else.
      let isElse: bool = tokens.len() == 1;
      branches.push(ConditionBranch
      {
        condition: if isElse { None } else { Some(tokens[1..].to_vec()) },
        body: line.lines.clone()
      });
      if isElse {
        break
      }
      //
    }
  }
  {
    let parent: RwLockReadGuard<Structure> = parentLink.read().unwrap();
    parent.lineIndex.set(parent.lineIndex.get() + branches.len());
  }
  branches
}

// =================================================================================================

/// Выполняет тело ветки во временной структуре и возвращает её `result`;
/// 
/// Если `result` нет — пустой `None`-токен.
fn executeConditionBody(
  body: Option< Vec< Arc<RwLock<Line>> > >,
  parentLink: &Arc<RwLock<Structure>>
) -> Token
{
  let blockStructure: Arc<RwLock<Structure>> =
    Arc::new(RwLock::new(Structure::new(
      None,
      StructureMut::Constant,
      StructureType::Primitive(PrimitiveType::Method),
      body,
      Some(parentLink.clone())
    )));
  readStructureLines(blockStructure.clone());
  let result: Option<Token> = blockStructure.read().unwrap().result.clone();
  result.unwrap_or_else(|| Token::newEmpty(TokenType::None))
}

/// Единая логика `if`/`elif`/`else`: и для условного блока-statement, и для блока-выражения.
///
/// Ветки проверяются по порядку: блок запускается только по `True`;
/// `False` и `None` (пустота, #39) — идём к следующей ветке.
///
/// Возвращает `result` выполненной ветки (пустой `None`-токен, если `result` нет),
/// либо `None`, если ни одна ветка не подошла.
fn executeConditionChain(
  branches: Vec<ConditionBranch>,
  parentLink: &Arc<RwLock<Structure>>
) -> Option<Token>
{
  for mut branch in branches
  {
    if let Some(conditionTokens) = &mut branch.condition
    {
      let conditionTruth: bool =
      { // Блокировка на родителе живёт только в этом выражении.
        *parentLink.read().unwrap()
          .expression(conditionTokens)
          .getDataType() == TokenType::True
      };
      if !conditionTruth {
        continue
      }
    }
    return Some(executeConditionBody(branch.body, parentLink));
  }
  None
}

// =================================================================================================

/// Читает условный блок (`if`/`elif`/`else`) как statement.
///
/// Первая ветка — текущая линия, остальные ищутся ниже.
/// 
/// Сдвигает `lineIndex` на количество пройденных блоков.
pub fn executeConditionStatement(
  lineTokens: &[Token],
  lineLines: &Option< Vec< Arc<RwLock<Line>> > >,
  parentLink: &Arc<RwLock<Structure>>
) -> bool
{
  let mut branches: Vec<ConditionBranch> = vec![ConditionBranch
  {
    // Если в токенах просто TokenType::Question, значит это else.
    condition: if lineTokens.len() > 1 { Some(lineTokens[1..].to_vec()) } else { None },
    body: lineLines.clone()
  }];
  // Если первый блок уже else, то цепочка закончена: `?` ниже начинает новую.
  if branches[0].condition.is_some()
  {
    branches.extend(collectConditionBranches(parentLink));
  }
  executeConditionChain(branches, parentLink);
  true
}

/// Читает `?` в токенах как блок-выражение `a := ? True { = 20 }`.
///
/// Выполняет цепочку: первая ветка — `?` из этих токенов и `bodyLines`
/// (вложения текущей линии), остальные (`elif`/`else`) — линии `?` ниже.
/// Возвращает токены, в которых `?` + условие заменены результатом цепочки.
/// `?` без условия — это else: блок выполняется безусловно.
///
/// Возвращает `None`, если блока-выражения нет — тогда это обычные токены.
pub fn replaceConditionExpression(
  tokens: &[Token],
  bodyLines: &Option< Vec< Arc<RwLock<Line>> > >,
  parentLink: &Arc<RwLock<Structure>>
) -> Option< Vec<Token> >
{
  let questionIndex: usize = tokens.iter().position(|t| *t.getDataType() == TokenType::Question)?;
  // Если после `?` нет условия, то это else: блок выполняется безусловно `a := ? { = 1 }`.
  let hasCondition: bool = questionIndex + 1 < tokens.len();

  let mut branches: Vec<ConditionBranch> = vec![ConditionBranch {
    condition: if hasCondition { Some(tokens[questionIndex + 1..].to_vec()) } else { None },
    body: bodyLines.clone()
  }];
  // Если первый блок уже else, то цепочка закончена: `?` ниже начинает новую.
  if hasCondition {
    branches.extend(collectConditionBranches(parentLink));
  }

  let result: Token = executeConditionChain(branches, parentLink)
    .unwrap_or_else(|| Token::newEmpty(TokenType::None));
  let mut replaced: Vec<Token> = tokens[..questionIndex].to_vec();
  replaced.push(result);
  Some(replaced)
}

// =================================================================================================
