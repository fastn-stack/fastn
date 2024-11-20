pub trait TDoc {
    fn get_opt_function(&self, name: &str, line_number: usize) -> Option<fastn_resolved::Function>;

    fn get_opt_record(&self, name: &str, line_number: usize) -> Option<fastn_resolved::Record>;

    fn name(&self) -> &str;

    fn get_opt_component(
        &self,
        name: &str,
        line_number: usize,
    ) -> Option<fastn_resolved::ComponentDefinition>;
    fn get_opt_web_component(
        &self,
        name: &str,
        line_number: usize,
    ) -> Option<fastn_resolved::WebComponentDefinition>;
}
