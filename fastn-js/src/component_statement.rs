pub enum ComponentStatement {
    StaticVariable(fastn_js::StaticVariable),
    MutableVariable(fastn_js::MutableVariable),
    // JSExpression(ExprNode),
    // MutableList(MutableList),
    // RecordInstance(RecordInstance),
    // Formula(Formula),
    // CreateKernel(CreateKernel),
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
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        match self {
            ComponentStatement::StaticVariable(f) => f.to_js(),
            ComponentStatement::MutableVariable(f) => f.to_js(),
        }
    }
}
