pub(crate) mod expression;
pub(crate) mod kind;
pub(crate) mod property_value;
pub(crate) mod variable;

#[derive(Debug, PartialEq)]
pub enum Thing {
    Variable(ftd::interpreter::Variable),
}
