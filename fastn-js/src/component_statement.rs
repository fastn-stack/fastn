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

impl ComponentStatement {
    pub fn get_variable_name(&self) -> Option<String> {
        match self {
            ComponentStatement::StaticVariable(static_variable) => {
                Some(static_variable.name.clone())
            }
            ComponentStatement::MutableVariable(mutable_variable) => {
                Some(mutable_variable.name.clone())
            }
            ComponentStatement::RecordInstance(record_instance) => {
                Some(record_instance.name.clone())
            }
            ComponentStatement::MutableList(mutable_list) => Some(mutable_list.name.clone()),
            _ => None,
        }
    }
}

// pub struct ExprNode {
//     operator: Operator,
//     children: Vec<ExprNode>,
// }
//
// pub enum Operator {}
