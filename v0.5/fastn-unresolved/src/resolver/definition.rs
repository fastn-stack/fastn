impl fastn_unresolved::Definition {
    pub fn resolve(
        &mut self,
        _definitions: &std::collections::HashMap<String, fastn_unresolved::URD>,
        _modules: &std::collections::HashMap<fastn_unresolved::Module, bool>,
        _arena: &mut fastn_unresolved::Arena,
        _output: &mut fastn_unresolved::resolver::Output,
        _main_package: &fastn_package::MainPackage,
    ) {
        todo!()
    }
}
