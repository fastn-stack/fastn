pub enum ComponentStatement {
    StaticVariable(fastn_js::StaticVariable),
    MutableVariable(fastn_js::MutableVariable),
    CreateKernel(fastn_js::Kernel),
    SetProperty(fastn_js::SetProperty),
    InstantiateComponent(fastn_js::InstantiateComponent),
    AddEventHandler(fastn_js::EventHandler),
    Done { component_name: String },
    Return { component_name: String },
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
