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
        let name = component_declaration_variable_name(parent, index);
        Kernel {
            element_kind,
            name,
            parent: parent.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ElementKind {
    Row,
    Column,
    ContainerElement,
    Integer,
    Decimal,
    Boolean,
    Text,
    Image,
    Video,
    IFrame,
    Device,
    CheckBox,
    TextInput,
    Rive,
    Document,
    Code,
    WebComponent(String),
}

#[derive(Debug)]
pub struct InstantiateComponent {
    pub component: InstantiateComponentData,
    pub arguments: Vec<(String, fastn_js::SetPropertyValue, bool)>,
    pub parent: String,
    pub inherited: String,
    pub should_return: bool,
    pub var_name: String,
    pub already_formatted: bool,
}

#[derive(Debug)]
pub enum InstantiateComponentData {
    Name(String),
    // Todo: add closure to `uis` to display 0th item
    // -- ftd.ui list uis:
    // -- ftd.text: Hello World
    // -- end: ftd.ui
    // -- uis.0:
    Definition(fastn_js::SetPropertyValue),
}

impl InstantiateComponent {
    pub fn new(
        component_name: &str,
        arguments: Vec<(String, fastn_js::SetPropertyValue, bool)>,
        parent: &str,
        inherited: &str,
        should_return: bool,
        index: usize,
        already_formatted: bool,
    ) -> InstantiateComponent {
        InstantiateComponent {
            component: fastn_js::InstantiateComponentData::Name(component_name.to_string()),
            arguments,
            parent: parent.to_string(),
            inherited: inherited.to_string(),
            should_return,
            var_name: component_declaration_variable_name(parent, index),
            already_formatted,
        }
    }

    pub fn new_with_definition(
        component_definition: fastn_js::SetPropertyValue,
        arguments: Vec<(String, fastn_js::SetPropertyValue, bool)>,
        parent: &str,
        inherited: &str,
        should_return: bool,
        index: usize,
        already_formatted: bool,
    ) -> InstantiateComponent {
        InstantiateComponent {
            component: fastn_js::InstantiateComponentData::Definition(component_definition),
            arguments,
            parent: parent.to_string(),
            inherited: inherited.to_string(),
            should_return,
            var_name: component_declaration_variable_name(parent, index),
            already_formatted,
        }
    }
}

fn component_declaration_variable_name(parent: &str, index: usize) -> String {
    format!("{parent}i{index}")
}
