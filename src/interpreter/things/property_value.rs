#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Value {
        value: ftd::Value,
    },
    Reference {
        name: String,
        kind: ftd::interpreter::Kind,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    None {
        kind: ftd::interpreter::Kind,
    },
    String {
        text: String,
    },
    Integer {
        value: i64,
    },
    Decimal {
        value: f64,
    },
    Boolean {
        value: bool,
    },
    Object {
        values: ftd::Map<PropertyValue>,
    },
    Record {
        name: String,
        fields: ftd::Map<PropertyValue>,
    },
    OrType {
        name: String,
        variant: String,
        fields: ftd::Map<PropertyValue>,
    },
    List {
        data: Vec<PropertyValue>,
        kind: ftd::interpreter::Kind,
    },
    Optional {
        data: Box<Option<Value>>,
        kind: ftd::interpreter::Kind,
    },
    Map {
        data: ftd::Map<Value>,
        kind: ftd::p2::Kind,
    },
    // TODO: UI
    // UI {
    //     name: String,
    //     component: ftd::interpreter::Component,
    // },
}
