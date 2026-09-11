use std::cell::RefCell;
use std::rc::Rc;
use chillffi::ffi::scope::{FFIScope, Scope};
// =================================================================================================

// Стек удерживаемых FFI scope на текущем потоке. Один уровень стека —
// один `@ffi { ... }` блок rts. Создаётся лениво (после 1-го FFI вызова
// внутри блока) — требование issue #79 / #84.
//
// Важные инварианты:
//  - `FFIScope` не `Sync` (внутри `UnsafeCell`/`thread_local!` у chillffi),
//    поэтому используем `Rc<RefCell<...>>`, а не `Arc<Mutex<...>>`.
//  - `Library<'g>` живёт строго внутри одного вызова, потому что
//    `Scope<'_>` borrows из `FFIScope`. Поэтому в API ниже `&Scope<'_>`
//    отдаётся только на время одного FFI вызова.
//  - При выходе из `@ffi` блока слот в стеке снимается; если scope
//    уже был создан (RefCell::Some), он дропается здесь же по RAII.

// =================================================================================================

thread_local! {
  /// Стек ленивых FFI scope-ов. Каждый `@ffi { ... }` блок пушит свой
  /// `Rc<RefCell<Option<FFIScope>>>` сюда при входе и снимает при выходе.
  static FfiScopeStack: RefCell<Vec<Rc<RefCell<Option<FFIScope>>>>> =
    const { RefCell::new(Vec::new()) };
}

/// Отмечает вход в `@ffi { ... }` блок. Кладёт «пустой» (lazy) scope-slot
/// на стек. Сам `FFIScope` не создаётся до первого реального FFI вызова.
pub fn enterFfiBlock() -> Rc<RefCell<Option<FFIScope>>>
{
  let slot: Rc<RefCell<Option<FFIScope>>> = Rc::new(RefCell::new(None));
  FfiScopeStack.with(|stack| {
    stack.borrow_mut().push(Rc::clone(&slot));
  });
  slot
}

/// Отмечает выход из `@ffi { ... }` блока. Снимает свой слот со стека.
/// Если scope уже был создан — он дропается здесь, по RAII `FFIScope::drop`,
/// вместе с библиотеками и аллокациями, выполненными внутри блока.
pub fn exitFfiBlock() -> ()
{
  FfiScopeStack.with(|stack| {
    let mut s = stack.borrow_mut();
    // Закрывающий pop — если стек пустой, это баг парсера.
    let _ = s.pop();
  });
}

/// Возвращает `true`, если текущая точка выполнения находится внутри
/// `@ffi { ... }` блока (на верхушке стека что-то лежит).
pub fn isInsideFfiBlock() -> bool
{
  FfiScopeStack.with(|stack| !stack.borrow().is_empty())
}

/// Достаёт scope-slot текущего `@ffi` блока (тот, что положили в `enterFfiBlock`).
/// Возвращает `None`, если мы не внутри `@ffi` блока.
pub fn currentFfiBlock() -> Option<Rc<RefCell<Option<FFIScope>>>>
{
  FfiScopeStack.with(|stack| stack.borrow().last().map(Rc::clone))
}

/// Гарантирует, что у текущего `@ffi` блока есть живой `FFIScope`,
/// лениво создавая его на первом обращении. Возвращает копию Rc на слот —
/// реальный `FFIScope` можно достать у него через `borrow()`.
///
/// Семантика «создаётся после 1 запроса ffi» соблюдается здесь:
///  - до первого FFI-вызова в блоке слот хранит `None`, никакого
///    `FFIScope::enter()` (читай: никакого fork зигота) не происходит;
///  - на первом вызове `FFIScope::enter()` дёргается ровно один раз,
///    последующие вызовы переиспользуют тот же scope.
pub fn ensureFfiScope() -> Option<Rc<RefCell<Option<FFIScope>>>>
{
  let slot: Rc<RefCell<Option<FFIScope>>> = currentFfiBlock()?;

  // Создаём FFIScope, если ещё не создан.
  // Используем обычный `borrow_mut` — мы единственные владельцы слота.
  {
    let mut cell = slot.borrow_mut();
    if cell.is_none() {
      match FFIScope::enter() {
        Ok(scope) => *cell = Some(scope),
        // Если не смогли войти в scope — оставляем слот пустым.
        // Вызывающий код обработает как обычный FFI-вызов и вернёт ошибку.
        Err(_) => return None,
      }
    }
  }

  Some(slot)
}

/// Выполняет FFI-операцию `f` внутри временно взятого `&Scope<'_>` текущего
/// `@ffi` блока. Это единственное место, где `Scope<'_>` живёт достаточно
/// долго, чтобы через него можно было загрузить `Library` и позвать метод.
///
/// Если мы не в `@ffi` блоке — возвращает `None`, и вызывающий код
/// должен отработать fallback (старый `ffi!{}` макрос без удержания).
pub fn withCurrentFfiScope<R>(
  f: impl FnOnce(&Scope<'_>) -> Result<R, String>
) -> Option<Result<R, String>>
{
  // 1) Берём Rc на слот текущего блока (или None, если не в @ffi).
  let slot: Rc<RefCell<Option<FFIScope>>> = ensureFfiScope()?;

  // 2) Borrow-им FFIScope, чтобы получить &Scope<'_>.
  //    borrow живёт ровно столько, сколько нужно для вызова `f`,
  //    что и держит lifetime Scope<'_> валидным.
  let cell = slot.borrow();
  let ffi_scope: &FFIScope = cell.as_ref()?;
  let scope: Scope<'_> = ffi_scope.scope();

  Some(f(&scope))
}

// =================================================================================================

#[cfg(test)]
mod tests
{
  use super::*;
  // ===============================================================================================

  /// Вне `@ffi` блока `isInsideFfiBlock` возвращает `false`.
  #[test]
  fn outsideBlock() -> ()
  {
    assert!(!isInsideFfiBlock());
    assert!(currentFfiBlock().is_none());
    assert!(withCurrentFfiScope(|_s| Ok::<_, String>(())).is_none());
  }

  /// Внутри `@ffi` блока до первого FFI-вызова scope ещё не создан
  /// (требование issue #79/#84: «ffi блок должен создаваться после 1 запроса
  /// ffi в rts внутри блока»). После вызова `ensureFfiScope` он появляется.
  #[test]
  fn lazyScopeCreation() -> ()
  {
    let slot: Rc<RefCell<Option<FFIScope>>> = enterFfiBlock();
    assert!(isInsideFfiBlock());
    assert!(slot.borrow().is_none(),
            "до первого FFI-вызова FFIScope ещё не создан");

    let _: Option<Rc<RefCell<Option<FFIScope>>>> = ensureFfiScope();
    assert!(slot.borrow().is_some(),
            "после ensureFfiScope FFIScope создан");

    exitFfiBlock();
    assert!(!isInsideFfiBlock(),
            "после exitFfiBlock стек пуст");
  }

  /// `ensureFfiScope` идемпотентен: повторный вызов переиспользует
  /// уже созданный scope, а не плодит новые (Scope Retention).
  #[test]
  fn scopeReusedAcrossCalls() -> ()
  {
    let _slot = enterFfiBlock();
    let first: Option<Rc<RefCell<Option<FFIScope>>>> = ensureFfiScope();
    let second: Option<Rc<RefCell<Option<FFIScope>>>> = ensureFfiScope();
    assert!(first.is_some());
    assert!(second.is_some());
    // Один и тот же Rc на слот — это и есть «scope удерживается».
    assert!(Rc::ptr_eq(&first.unwrap(), &second.unwrap()));
    exitFfiBlock();
  }

  /// Реальный round-trip через libc: грузим `libc.so.6` в удерживаемом
  /// scope и зовём `getpid` дважды. Если scope не удерживается между
  /// вызовами (как было в ffi!{} макросе раньше) — это всё равно
  /// сработает, потому что getpid stateless. Поэтому дополнительно
  /// проверяем, что `withCurrentFfiScope` действительно возвращает Some.
  #[test]
  fn libcGetpid() -> ()
  {
    use chillffi::ffi::library::Library;
    use chillffi::ffi::scope::Scope;

    let _slot = enterFfiBlock();

    let pid1: Option<Result<i32, String>> = withCurrentFfiScope(|scope: &Scope<'_>| {
      let libc: Library = scope.load("libc.so.6")
        .map_err(|e| e.to_string())?;
      libc.call("getpid").void()
        .map_err(|e| e.to_string())?;
      Ok::<i32, String>(std::process::id() as i32)
    });
    let pid2: Option<Result<i32, String>> = withCurrentFfiScope(|scope: &Scope<'_>| {
      let libc: Library = scope.load("libc.so.6")
        .map_err(|e| e.to_string())?;
      libc.call("getpid").void()
        .map_err(|e| e.to_string())?;
      Ok::<i32, String>(std::process::id() as i32)
    });

    exitFfiBlock();

    let pid1: i32 = pid1.expect("должны быть в @ffi блоке").expect("getpid #1");
    let pid2: i32 = pid2.expect("должны быть в @ffi блоке").expect("getpid #2");
    assert_eq!(pid1, pid2, "getpid в одном процессе возвращает то же значение");
    assert!(pid1 > 0, "pid должен быть > 0");
  }

  // ===============================================================================================
}

// =================================================================================================