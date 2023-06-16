pub enum Ast {
    Func(Func),
}

pub struct Func {
    pub name: String,
    // params: Vec<Param>,
    // body: Vec<Instruction>,
}

// enum Instruction {
//     StaticVariable(StaticVariable),
//     MutableVariable(MutableVariable),
//     MutableList(MutableList),
//     RecordInstance(RecordInstance),
//     Formula(Formula),
//     CreateKernel(CreateKernel),
//     SetProperty(SetProperty),
//     InstantiateComponent(InstantiateComponent),
//     ForLoop(ForLoop),
//     ConditionalComponent(ConditionalComponent),
// }
//
