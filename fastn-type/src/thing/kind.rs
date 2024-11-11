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

impl Kind {
    pub fn get_name(&self) -> String {
        match self {
            Kind::String { .. } => "string".to_string(),
            Kind::Integer { .. } => "integer".to_string(),
            Kind::Boolean { .. } => "boolean".to_string(),
            Kind::Decimal { .. } => "decimal".to_string(),
            Kind::Constant { .. } => "constant".to_string(),
            Kind::List { .. } => "list".to_string(),
            Kind::Object { .. } => "object".to_string(),
            Kind::OrType { name, .. } => name.clone(),
            Kind::Optional { .. } => "optional".to_string(),
            Kind::Void { .. } => "void".to_string(),
            Kind::Module => "module".to_string(),
            Kind::KwArgs => "kw-args".to_string(),
            Kind::UI { name, .. } => name.clone().unwrap_or("record".to_string()),
            Kind::Record { name } => name.clone(),
        }
    }

    pub fn into_kind_data(self) -> KindData {
        KindData::new(self)
    }

    pub fn string() -> Kind {
        Kind::String
    }

    pub fn integer() -> Kind {
        Kind::Integer
    }

    pub fn decimal() -> Kind {
        Kind::Decimal
    }

    pub fn boolean() -> Kind {
        Kind::Boolean
    }

    pub fn module() -> Kind {
        Kind::Module
    }

    pub fn kwargs() -> Kind {
        Kind::KwArgs
    }

    pub fn ui() -> Kind {
        Kind::UI {
            name: None,
            subsection_source: false,
            is_web_component: false,
        }
    }

    pub fn ui_with_name(name: &str) -> Kind {
        Kind::UI {
            name: Some(name.to_string()),
            subsection_source: false,
            is_web_component: false,
        }
    }

    pub fn web_ui_with_name(name: &str) -> Kind {
        Kind::UI {
            name: Some(name.to_string()),
            subsection_source: false,
            is_web_component: true,
        }
    }

    pub fn subsection_ui() -> Kind {
        Kind::UI {
            name: None,
            subsection_source: true,
            is_web_component: false,
        }
    }

    pub fn object() -> Kind {
        Kind::Object
    }

    pub fn void() -> Kind {
        Kind::Void
    }

    pub fn record(name: &str) -> Kind {
        Kind::Record {
            name: name.to_string(),
        }
    }

    pub fn or_type(name: &str) -> Kind {
        Kind::OrType {
            name: name.to_string(),
            variant: None,
            full_variant: None,
        }
    }

    pub fn or_type_with_variant(name: &str, variant: &str, full_variant: &str) -> Kind {
        Kind::OrType {
            name: name.to_string(),
            variant: Some(variant.to_string()),
            full_variant: Some(full_variant.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct KindData {
    pub kind: Kind,
    pub caption: bool,
    pub body: bool,
}

impl KindData {
    pub fn new(kind: Kind) -> KindData {
        KindData {
            kind,
            caption: false,
            body: false,
        }
    }
}
