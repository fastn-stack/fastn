#[derive(Debug)]
pub struct ConditionalComponent {
    pub deps: Vec<String>,
    pub condition: fastn_resolved::evalexpr::ExprNode,
    pub statements: Vec<fastn_js::ComponentStatement>,
    pub parent: String,
    pub should_return: bool,
}
