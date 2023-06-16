pub enum Instruction {
    StaticVariable(StaticVariable),
    // MutableVariable(MutableVariable),
    // MutableList(MutableList),
    // RecordInstance(RecordInstance),
    // Formula(Formula),
    // CreateKernel(CreateKernel),
    // SetProperty(SetProperty),
    // InstantiateComponent(InstantiateComponent),
    // ForLoop(ForLoop),
    // ConditionalComponent(ConditionalComponent),
}

pub struct StaticVariable {
    pub name: String,
    pub value: String,
}
