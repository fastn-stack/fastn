pub enum ComponentStatement {
    StaticVariable(fastn_js::StaticVariable),
    MutableVariable(fastn_js::MutableVariable),
    CreateKernel(fastn_js::Kernel),
    SetProperty(fastn_js::SetProperty),
    InstantiateComponent(fastn_js::InstantiateComponent),
    AddEventHandler(fastn_js::EventHandler),
    Return { component_name: String },
    ConditionalComponent(fastn_js::ConditionalComponent),
    MutableList(fastn_js::MutableList),
    ForLoop(fastn_js::ForLoop),
    RecordInstance(fastn_js::RecordInstance),
    // JSExpression(ExprNode),
    // RecordInstance(RecordInstance),
    // Formula(Formula),
}

// pub struct ExprNode {
//     operator: Operator,
//     children: Vec<ExprNode>,
// }
//
// pub enum Operator {}
