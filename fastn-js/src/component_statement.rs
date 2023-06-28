pub enum ComponentStatement {
    StaticVariable(fastn_js::StaticVariable),
    MutableVariable(fastn_js::MutableVariable),
    CreateKernel(fastn_js::Kernel),
    Done { component_name: String },
    SetProperty(fastn_js::SetProperty),
    InstantiateComponent(fastn_js::InstantiateComponent),
    // JSExpression(ExprNode),
    // MutableList(MutableList),
    // RecordInstance(RecordInstance),
    // Formula(Formula),
    // ForLoop(ForLoop),
    // ConditionalComponent(ConditionalComponent),
}

// pub struct ExprNode {
//     operator: Operator,
//     children: Vec<ExprNode>,
// }
//
// pub enum Operator {}
