pub struct ForLoop {
    pub list_variable: fastn_js::SetPropertyValue,
    pub statements: Vec<fastn_js::ComponentStatement>,
    pub parent: String,
    pub should_return: bool,
}
