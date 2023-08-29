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
    GlobalKeySeq(Vec<String>),
    Input,
    Change,
    Blur,
    Focus,
}

#[derive(Debug)]
pub enum FunctionData {
    Name(String),
    // -- component bar:
    // module m:
    //
    // -- ftd.text: $bar.m.func(a = Hello)
    // -- end: bar
    Definition(fastn_js::SetPropertyValue),
}

#[derive(Debug)]
pub struct Function {
    pub name: Box<FunctionData>,
    pub parameters: Vec<(String, fastn_js::SetPropertyValue)>,
}
