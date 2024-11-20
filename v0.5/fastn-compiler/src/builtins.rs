/// We are using a LazyLock, which[1]:
///
/// > This method will block the calling thread if another initialization routine is currently
/// > running.
///
/// So the lock only blocks threads when the first time initialization is happening, so it is fine
/// to use it in the global scope.
///
/// [1]: https://doc.rust-lang.org/beta/std/sync/struct.LazyLock.html#method.deref
pub static BUILTINS: std::sync::LazyLock<
    // TODO: consider https://crates.io/crates/phf
    std::collections::HashMap<string_interner::DefaultSymbol, fastn_type::Definition>,
> = std::sync::LazyLock::new(builtins);

fn builtins() -> std::collections::HashMap<string_interner::DefaultSymbol, fastn_type::Definition> {
    todo!()
}
