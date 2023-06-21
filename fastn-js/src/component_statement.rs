pub enum ComponentStatement {
    StaticVariable(fastn_js::StaticVariable),
    MutableVariable(fastn_js::MutableVariable),
    CreateKernel(fastn_js::Kernel),
    Done { component_name: String },
    // JSExpression(ExprNode),
    // MutableList(MutableList),
    // RecordInstance(RecordInstance),
    // Formula(Formula),
    // SetProperty(SetProperty),
    // InstantiateComponent(InstantiateComponent),
    // ForLoop(ForLoop),
    // ConditionalComponent(ConditionalComponent),
}

// pub struct ExprNode {
//     operator: Operator,
//     children: Vec<ExprNode>,
// }
//
// pub enum Operator {}

impl ComponentStatement {
    pub fn from_component(
        component: &ftd::interpreter::Component,
        parent: &str,
        index: usize,
    ) -> Vec<ComponentStatement> {
        let mut component_statements = vec![];
        if fastn_js::utils::is_kernel(component.name.as_str()) {
            let kernel = fastn_js::Kernel::from_component(component.name.as_str(), parent, index);
            component_statements.push(ComponentStatement::CreateKernel(kernel.clone()));
            component_statements.push(ComponentStatement::Done {
                component_name: kernel.name.clone(),
            });
        } else {
            todo!()
        }
        component_statements
    }
}
