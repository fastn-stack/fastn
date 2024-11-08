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

impl Kind {
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

    pub fn object() -> Kind {
        Kind::Object
    }

    pub fn kwargs() -> Kind {
        Kind::KwArgs
    }

    pub fn ui_with_name(name: &str) -> Kind {
        Kind::UI {
            name: Some(name.to_string()),
            subsection_source: false,
            is_web_component: false,
        }
    }

    pub fn or_type_with_variant(name: &str, variant: &str, full_variant: &str) -> Kind {
        Kind::OrType {
            name: name.to_string(),
            variant: Some(variant.to_string()),
            full_variant: Some(full_variant.to_string()),
        }
    }

    pub fn into_list(self) -> Kind {
        Kind::List {
            kind: Box::new(self),
        }
    }
    pub fn record(name: &str) -> Kind {
        Kind::Record {
            name: name.to_string(),
        }
    }

    pub fn inner(self) -> Kind {
        match self {
            Kind::Optional { kind } => kind.as_ref().to_owned(),
            t => t,
        }
    }
}

impl KindData {
    pub fn is_list(&self) -> bool {
        matches!(self.kind, Kind::List { .. })
    }

    pub fn is_optional(&self) -> bool {
        matches!(self.kind, Kind::Optional { .. })
    }
}
