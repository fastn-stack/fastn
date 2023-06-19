pub enum Instruction {
    StaticVariable(fastn_js::StaticVariable),
    MutableVariable(fastn_js::MutableVariable),
    // MutableList(MutableList),
    // RecordInstance(RecordInstance),
    // Formula(Formula),
    // CreateKernel(CreateKernel),
    // SetProperty(SetProperty),
    // InstantiateComponent(InstantiateComponent),
    // ForLoop(ForLoop),
    // ConditionalComponent(ConditionalComponent),
}

impl Instruction {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        match self {
            Instruction::StaticVariable(f) => f.to_js(),
            Instruction::MutableVariable(f) => f.to_js(),
        }
    }
}
