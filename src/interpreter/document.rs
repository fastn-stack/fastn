use itertools::Itertools;

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub data: ftd::Map<ftd::interpreter::Thing>,
    pub name: String,
    pub instructions: Vec<ftd::interpreter::Instruction>,
    pub main: ftd::Column,
    pub aliases: ftd::Map<String>,
}
