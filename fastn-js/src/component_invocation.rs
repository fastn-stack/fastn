#[derive(Clone, Debug)]
pub struct Kernel {
    pub element_kind: ElementKind,
    pub name: String,
    pub parent: String,
}

impl Kernel {
    pub fn from_component(component_name: &str, parent: &str, index: usize) -> Kernel {
        let element_kind = fastn_js::ElementKind::from_component_name(component_name);
        Kernel {
            element_kind,
            name: format!("{parent}i{index}"),
            parent: parent.to_string(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ElementKind {
    Row,
    Column,
    Integer,
    Decimal,
    Boolean,
    Text,
    Image,
    IFrame,
}

impl ElementKind {
    pub(crate) fn from_component_name(name: &str) -> ElementKind {
        match name {
            "ftd#text" => ElementKind::Text,
            "ftd#integer" => ElementKind::Integer,
            "ftd#row" => ElementKind::Row,
            "ftd#column" => ElementKind::Column,
            "ftd#decimal" => ElementKind::Decimal,
            "ftd#boolean" => ElementKind::Boolean,
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct InstantiateComponent {
    pub name: String,
    pub arguments: Vec<fastn_js::SetPropertyValue>,
    pub parent: String,
    pub inherited: String,
    pub should_return: bool,
}
