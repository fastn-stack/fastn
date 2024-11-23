impl fastn_unresolved::ComponentInvocation {
    pub fn resolve(
        &mut self,
        input: fastn_unresolved::resolver::Input<'_>,
        output: &mut fastn_unresolved::resolver::Output,
    ) {
        fastn_unresolved::resolver::name::resolve(&mut self.name, input, output);
    }
}
