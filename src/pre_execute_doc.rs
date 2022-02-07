#[derive(Debug, PartialEq)]
pub(crate) struct PreExecuteDoc {
    pub name: String,
    pub bag: std::collections::BTreeMap<String, ftd::p2::Thing>,
    pub aliases: std::collections::BTreeMap<String, String>,
    pub instructions: Vec<ftd::Instruction>,
}

impl PreExecuteDoc {
    pub(crate) fn pre_execute(&mut self, instructions: &[ftd::Instruction]) {
        for instruction in instructions {
            match instruction {
                ftd::Instruction::Component { .. } => {}
                ftd::Instruction::ChildComponent { .. } => {}
                ftd::Instruction::ChangeContainer { .. } => {}
                ftd::Instruction::RecursiveChildComponent { .. } => {}
            }
        }
    }
}
