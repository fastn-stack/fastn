#[derive(Debug)]
pub struct EventHandler {
    pub event: fastn_js::Event,
    pub action: fastn_js::Function,
    pub element_name: String,
}

#[derive(Debug)]
pub enum Event {
    Click,
    MouseEnter,
    MouseLeave,
    ClickOutside,
    GlobalKey(Vec<String>),
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<(String, fastn_js::SetPropertyValue)>,
}
