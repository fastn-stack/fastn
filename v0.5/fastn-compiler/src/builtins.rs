// TODO: can we avoid the lock here?
pub static BUILTINS: std::sync::LazyLock<
    std::collections::HashMap<string_interner::DefaultSymbol, fastn_type::Definition>,
> = std::sync::LazyLock::new(builtins);

fn builtins() -> std::collections::HashMap<string_interner::DefaultSymbol, fastn_type::Definition> {
    todo!()
}
