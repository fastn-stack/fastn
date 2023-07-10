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
    Device,
}

#[derive(Debug)]
pub struct InstantiateComponent {
    pub name: String,
    pub arguments: Vec<(String, fastn_js::SetPropertyValue)>,
    pub parent: String,
    pub inherited: String,
    pub should_return: bool,
}
