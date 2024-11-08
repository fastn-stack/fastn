#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Kind {
    String,
    Object,
    Integer,
    Decimal,
    Boolean,
    Record {
        name: String,
    }, // the full name of the record (full document name.record name)
    OrType {
        name: String,
        variant: Option<String>,
        full_variant: Option<String>,
    },
    List {
        kind: Box<Kind>,
    },
    Optional {
        kind: Box<Kind>,
    },
    UI {
        name: Option<String>,
        subsection_source: bool,
        is_web_component: bool,
    },
    Constant {
        kind: Box<Kind>,
    },
    Void,
    Module,
    KwArgs,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct KindData {
    pub kind: Kind,
    pub caption: bool,
    pub body: bool,
}
