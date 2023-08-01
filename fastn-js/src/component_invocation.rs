#[derive(Clone, Debug)]
pub struct Kernel {
    pub element_kind: ElementKind,
    pub name: String,
    pub parent: String,
}

impl Kernel {
    pub fn from_component(
        element_kind: fastn_js::ElementKind,
        parent: &str,
        index: usize,
    ) -> Kernel {
        Kernel {
            element_kind,
            name: component_declaration_variable_name(parent, index),
            parent: parent.to_string(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ElementKind {
    Row,
    Column,
    ContainerElement,
    Integer,
    Decimal,
    Boolean,
    Text,
    Image,
    IFrame,
    Device,
    CheckBox,
    TextInput,
    Rive,
    Document,
    Code,
}

#[derive(Debug)]
pub struct InstantiateComponent {
    pub component_name: String,
    pub arguments: Vec<(String, fastn_js::SetPropertyValue)>,
    pub parent: String,
    pub inherited: String,
    pub should_return: bool,
    pub var_name: String,
}

impl InstantiateComponent {
    pub fn new(
        component_name: &str,
        arguments: Vec<(String, fastn_js::SetPropertyValue)>,
        parent: &str,
        inherited: &str,
        should_return: bool,
        index: usize,
    ) -> InstantiateComponent {
        InstantiateComponent {
            component_name: component_name.to_string(),
            arguments,
            parent: parent.to_string(),
            inherited: inherited.to_string(),
            should_return,
            var_name: component_declaration_variable_name(parent, index),
        }
    }
}

fn component_declaration_variable_name(parent: &str, index: usize) -> String {
    format!("{parent}i{index}")
}
