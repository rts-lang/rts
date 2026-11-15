use serde::{Deserialize, Serialize};
use crate::parser::structure::structureType::primitiveTypeFrame::PrimitiveTypeFrame;
use crate::tokenizer::types::token::Token;
use crate::tokenizer::types::tokenType::TokenType;
// =================================================================================================

macro_rules! defineEnum
{
  (
    $($(#[$varAttr:meta])* $variant:ident $(($field:ty))?),* $(,)?
  ) =>
  {
    /// Список структур-примитивов.
    /// 
    /// Он также включает в себя `Custom` - хвост для пользовательских типов;
    /// Хотя фактически они не примитивы - но они должны быть логически здесь;
    /// Поэтому из списков имен `Custom` исключен на этапе раскрытия макроса.
    #[repr(u8)]
    #[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub enum PrimitiveType
    {
      $(
        $(#[$varAttr])*
        $variant $(($field))?,
      )*
    }

    impl PrimitiveType
    {
      /// Имена примитивов в порядке объявления (без `Custom`).
      const Names: &'static [&'static str] =
        defineEnum!(@names []; $($variant $(($field))?,)*);

      /// Пары `имя -> значение` для примитивов (без `Custom`).
      const Entries: &'static [(&'static str, PrimitiveType)] =
        defineEnum!(@entries []; $($variant $(($field))?,)*);

      /// Примитив по имени; `Custom` и неизвестные имена -> `None`.
      pub fn fromName(name: &str) -> Option<Self>
      {
        Self::Entries.iter().find(|(n, _)| *n == name).map(|(_, v)| v.clone())
      }

      /// Является ли строка именем примитива (не `Custom`).
      pub fn isPrimitive(name: &str) -> bool
      {
        Self::Names.contains(&name)
      }

      /// Перебор примитивов: `(имя, значение)`.
      pub fn iter() -> impl Iterator<Item = (&'static str, Self)>
      {
        Self::Entries.iter().cloned()
      }
    }
  };

  (@names [$($acc:tt)*];) =>
  {
    &[$($acc)*]
  };
  (@names [$($acc:tt)*]; $v:ident, $($rest:tt)*) =>
  {
    defineEnum!(@names [$($acc)* stringify!($v),]; $($rest)*)
  };
  (@names [$($acc:tt)*]; $v:ident ($t:ty), $($rest:tt)*) =>
  {
    defineEnum!(@names [$($acc)*]; $($rest)*)
  };

  (@entries [$($acc:tt)*];) =>
  {
    &[$($acc)*]
  };
  (@entries [$($acc:tt)*]; $v:ident, $($rest:tt)*) =>
  {
    defineEnum!(@entries
      [$($acc)* (stringify!($v), PrimitiveType::$v),];
      $($rest)*
    )
  };
  (@entries [$($acc:tt)*]; $v:ident ($t:ty), $($rest:tt)*) =>
  {
    defineEnum!(@entries [$($acc)*]; $($rest)*)
  };
}

// =================================================================================================

defineEnum!(
  None,
  Any, // todo У неё нет использований, их надо потом будет добавить.
  
  True,
  False,

  U8, U16, U32, U64, Usize,
  I8, I16, I32, I64, Isize,
  F32, F64,

  /// Указатель (raw).
  Pointer,
  
  /// ABI-композит: 2 поля `.pointer` и `.length`.
  String,
  /// Алиас от `Pointer`.
  CString,
  /// Байты без длины/NUL в самом StructureType — длина берётся из токена при FFI-вызове.
  // todo rewrite desc.
  RawString,

  // todo Требует удаление для FFI-ABI?
  //  Это кстати может называться ещё `Block`, но не знаю точно пока что.
  //  Но это не мешает по идее FFI, а штука в целом нужная для RTS.
  //  Сама она не участвует в FFI.
  Method,
  // todo Требует удаление для FFI-ABI?
  //  Идея была сделать List<Type> но это под вопросом.
  List,
  
  // todo `Method` и `List` не понятно должны ли вообще сюда входить или быть в `StructureType`?
  //  Потому что это явно выше и не C ABI типы, а что-то более высокое внутри самого языка.
  //  Может быть если `Method` = `Module`? Может быть их можно объединить? Ведь это блоки кода.
  //  Но тоже как бы `None`/`Any` - тоже штуки языка, а не C ABI. Тут вопрос не в том, чтобы 
  //  сделать C ABI список - это есть в `bridge` для FFI, а в правильности нахождения абстракций.

  /// Позволяет создавать пользовательские типы.
  Custom(String)
);

// =================================================================================================

impl PrimitiveType
{
  // ===============================================================================================
  
  /// Ставит стандартное значение токена для требуемого [`StructureType`].
  ///
  /// Очень удобно для случаев, если в структуру приравнивают что-то не то,
  /// и нам надо `default` значение.
  ///
  /// Например, если `a: U8 = "test"`, то будет `0` из-за константного поведения.
  pub(super) fn setTokenDefault(&self, token: &mut Token) -> ()
  {
    // Создаём базовый токен.
    let default: (TokenType, &str) = match self
    {
      primitive => primitive.getFrame().default
    };

    token.setDataType(default.0);
    token.setData(default.1);
  }

  // ===============================================================================================

  /// Выдаёт `TokenType`, соответствующий текущему `StructureType`:
  ///
  /// - `U8`/`U16`/`U32`/`U64`/`Usize` ⟶ `UInt`;
  ///
  /// - `I8`/`I16`/`I32`/`I64`/`Isize` ⟶ `Int`;
  ///
  /// - `F32`/`F64` ⟶ `UFloat` для неотрицательного значения, иначе `Float`;
  ///
  /// - Прочие примитивы ⟶ `None`, тип токена не пересчитывается.
  fn getTokenType(&self, tokenValue: &str) -> Option<TokenType>
  {
    match self
    {
      PrimitiveType::U8 |
      PrimitiveType::U16 |
      PrimitiveType::U32 |
      PrimitiveType::U64 |
      PrimitiveType::Usize => Some(TokenType::UInt),
      //
      PrimitiveType::I8 |
      PrimitiveType::I16 |
      PrimitiveType::I32 |
      PrimitiveType::I64 |
      PrimitiveType::Isize => Some(TokenType::Int),
      //
      PrimitiveType::F32 |
      PrimitiveType::F64 =>
        Some(if tokenValue.starts_with('-') { TokenType::Float } else { TokenType::UFloat }),
      //
      _ => None
    }
  }

  pub(super) fn retypeToken(&self, token: &mut Token) -> ()
  {
    let tokenValue: String = token.getData().toString().unwrap_or_default();
    if let Some(targetTokenType) = self.getTokenType(&tokenValue) {
      token.setDataType(targetTokenType);
    }
  }

  // ===============================================================================================
  
  /// Приводит `Token` в рамки [`StructureType`], чтобы [`Structure`] смог его хранить.
  ///
  /// **Нужно для:**
  /// - Методы приведения `U8(a)`;
  /// - Типизация параметров `abc(a: U8)`;
  /// - Объявление `a: U8 = ...`.
  ///
  //  todo Я бы переписал всё что ниже по desc или удалил.
  /// **Важно**:
  ///
  /// - `Union` - вариант подбирается по значению, приведение к нему `normalizeUnion`;
  ///
  /// - `None` - это отсутствие значения, а не значение: токен очищается, как у `None(a)`;
  ///
  /// - `String`/`RawString` - значение становится строкой этого типа:
  ///   `String(5)` - это `"5"`;
  ///
  /// - Остальное приводится по границам `StructureType`: число зажимается в
  ///   границы типа, значение не того рода становится базовым значением типа
  ///   (`U8(-10)` -> `0`, как и `U8("test")` -> `0`).
  ///
  /// - Границы приведения живут в рамке ([`PrimitiveTypeFrame`]): они есть только у
  ///   числовых типов; у остальных приведения нет - значение проходит как есть.
  ///
  /// - `TokenType` и `StructureType` - разные вещи: `TokenType` - вид данных
  ///   в токене (по нему значение читается), `StructureType` - рамка, в которую
  ///   значение зажимается.
  //
  // todo Есть идея ввести на уровне `Structure` в будущем `bool`,
  //  чтобы можно было проверить и оптимизировать эту работу.
  pub fn castToken(&self, token: &mut Token) -> ()
  {
    // ---------------------------------------------------------------------------------------------

    // `None` - отсутствие значения: `abc(a: None)` и `None(a)` очищают токен.
    if self == &PrimitiveType::None
    {
      token.setDataType(TokenType::None);
      token.setData(None);
      return;
    }

    // ---------------------------------------------------------------------------------------------
    
    // Строки: значение становится строкой этого типа, как `String(5)` - `"5"`.
    if matches!(self, PrimitiveType::String |PrimitiveType::RawString)
    {
      let data: String = token.getData().toString().unwrap_or_default();
      token.setDataType(
        if *self == PrimitiveType::String {
          TokenType::String
        } else {
          TokenType::RawString
        }
      );
      token.setData(data);
      return;
    }

    // ---------------------------------------------------------------------------------------------
    
    let dataType: TokenType = *token.getDataType(); // todo Плохо, что тут `*`.

    // ---------------------------------------------------------------------------------------------

    // Обработка типов; Только числовые типы имеют `min`/`max`;
    // Это важно, чтобы сохранит `None` ветку.
    let frame: PrimitiveTypeFrame = self.getFrame();
    match frame.min
    {
      // Здесь на самом деле мог бы быть `StructureType` в `match`,
      // но легче проверять через `TokenType`.
      Some(_) => match dataType
      {
        TokenType::UInt | TokenType::Int | TokenType::UFloat | TokenType::Float =>
        { // Числовые типы нормализуются.
          self.normalizeToken(&frame, token, &dataType);
        }
        // todo Строковые типы и другие должны тут нормализоваться? Или только `min`/`max`?
        _ =>
        { // Здесь пытаются прировнять что-то не то.
          self.setTokenDefault(token);
        }
      }
      None => {
        // Другие типы — ничего не делаем.
        //
        // todo По сути здесь должна быть логика рамок для других типов;
        //  Это всё вытекает из `константного поведения`;
        //  Но пока что здесь ничего нет; Возможно и не надо будет.
      }
    }

    // ---------------------------------------------------------------------------------------------

    // Пересчёт `TokenType`.
    self.retypeToken(token);

    // ---------------------------------------------------------------------------------------------
  }
  
  // ===============================================================================================
}

// =================================================================================================
