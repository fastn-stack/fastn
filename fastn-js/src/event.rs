#[derive(Debug)]
pub struct EventHandler {
    pub event: fastn_js::Event,
    pub action: fastn_js::Function,
    pub element_name: String,
}

#[derive(Debug)]
pub enum Event {
    OnClick,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<fastn_js::SetPropertyValue>,
}
