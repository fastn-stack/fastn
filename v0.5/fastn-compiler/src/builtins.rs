// TODO: can we avoid the lock here?
static CELL: std::sync::OnceLock<
    std::collections::HashMap<string_interner::DefaultSymbol, fastn_type::Definition>,
> = std::sync::OnceLock::new();

pub fn builtins(
) -> &'static std::collections::HashMap<string_interner::DefaultSymbol, fastn_type::Definition> {
    CELL.get_or_init(create_built_ins)
}

fn create_built_ins(
) -> std::collections::HashMap<string_interner::DefaultSymbol, fastn_type::Definition> {
    todo!()
}
