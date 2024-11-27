impl fastn_unresolved::ComponentInvocation {
    pub fn resolve(
        &mut self,
        input: &fastn_unresolved::resolver::Input<'_>,
        output: &mut fastn_unresolved::resolver::Output,
    ) {
        for c in self.children.iter_mut() {
            if let fastn_unresolved::UR::UnResolved(ref mut c) = c {
                c.resolve(input, output);
            }
        }

        fastn_unresolved::resolver::symbol::resolve(
            &self.module,
            &mut self.name,
            input,
            output,
            vec![], // TODO
        );

        let component = match self.name {
            fastn_unresolved::UR::Resolved(ref name) => input.get_component(name).unwrap(),
            // in case of error or not found, nothing left to do
            _ => return,
        };

        dbg!(component);
    }
}
