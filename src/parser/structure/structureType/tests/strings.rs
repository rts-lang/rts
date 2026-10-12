use crate::parser::structure::structureType::StructureType;
use crate::parser::testing::checkStructure;
// =================================================================================================

// Это тесты на три вида строк: String, CString и RawString.
//
// Аннотация типа сама решает каст.

// =================================================================================================

/// Пустая строка в каждом из трёх типов строк.
///
/// todo type() здесь None, а не String: с явным типом пустая строка теряет тип токена.
/// Хотя структура получает правильный тип. Без явного типа (`a := ""`) токен остаётся String.
#[test]
fn types() -> ()
{
  checkStructure!(
    r#"a: String = """#,
    "a",
    stype StructureType::String,
    value ""
  );
  // CString = Pointer (алиас).
  checkStructure!(
    r#"b: CString = """#,
    "b",
    stype StructureType::Pointer,
    value ""
  );
  checkStructure!(
    r#"c: RawString = """#,
    "c",
    stype StructureType::RawString,
    value ""
  );
}

// =================================================================================================
