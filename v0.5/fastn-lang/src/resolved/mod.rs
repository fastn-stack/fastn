pub enum Definition {
    Component(Component),
    Function(Function),
}

pub struct Component {}
pub struct Function {}

pub struct Document {
    pub definitions: Vec<Definition>,
}
