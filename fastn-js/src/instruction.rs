pub enum Statement {
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

pub struct ExprNode {
    operator: Operator,
    children: Vec<ExprNode>,
}

pub enum Operator {}

impl Statement {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        match self {
            Statement::StaticVariable(f) => f.to_js(),
            Statement::MutableVariable(f) => f.to_js(),
        }
    }
}
