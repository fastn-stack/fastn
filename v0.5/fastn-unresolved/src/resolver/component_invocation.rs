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

        let component =
            match fastn_unresolved::resolver::name::resolve(&mut self.name, input, output) {
                Some(c) => c,
                None => return,
            };

        dbg!(component);
    }
}
