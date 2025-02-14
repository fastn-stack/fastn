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
    Template,
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
            Kind::Template => "template".to_string(),
        }
    }

    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UI { .. }, Self::UI { .. }) => true,
            (Self::OrType { name: n1, .. }, Self::OrType { name: n2, .. }) => n1.eq(n2),
            (Self::Optional { kind, .. }, _) => kind.is_same_as(other),
            (_, Self::Optional { kind: other, .. }) => self.is_same_as(other),
            (Self::List { kind: k1 }, Self::List { kind: k2 }) => k1.is_same_as(k2),
            (Self::Template, Self::String) => true,
            (Self::String, Self::Template) => true,
            _ => self.eq(other),
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

    pub fn template() -> Kind {
        Kind::Template
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

    pub fn into_list(self) -> Kind {
        Kind::List {
            kind: Box::new(self),
        }
    }

    pub fn into_optional(self) -> Kind {
        Kind::Optional {
            kind: Box::new(self),
        }
    }

    pub fn inner(self) -> Kind {
        match self {
            Kind::Optional { kind } => kind.as_ref().to_owned(),
            t => t,
        }
    }

    pub fn mut_inner(&mut self) -> &mut Kind {
        match self {
            Kind::Optional { kind } => kind,
            t => t,
        }
    }

    pub fn ref_inner(&self) -> &Kind {
        match self {
            Kind::Optional { kind } => kind,
            t => t,
        }
    }

    pub fn inner_list(self) -> Kind {
        match self {
            Kind::List { kind } => kind.as_ref().to_owned(),
            t => t,
        }
    }

    pub fn ref_inner_list(&self) -> &Kind {
        match self {
            Kind::List { kind } => kind,
            t => t,
        }
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Kind::List { .. })
    }

    pub fn is_subsection_ui(&self) -> bool {
        matches!(
            self,
            Kind::UI {
                subsection_source: true,
                ..
            }
        )
    }

    pub fn is_ui(&self) -> bool {
        matches!(self, Kind::UI { .. })
    }

    pub fn is_optional(&self) -> bool {
        matches!(self, Kind::Optional { .. })
    }

    pub fn is_record(&self) -> bool {
        matches!(self, Kind::Record { .. })
    }

    pub fn is_or_type(&self) -> bool {
        matches!(self, Kind::OrType { .. })
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Kind::String { .. })
    }

    pub fn is_module(&self) -> bool {
        matches!(self, Kind::Module)
    }

    pub fn is_kwargs(&self) -> bool {
        matches!(self, Kind::KwArgs)
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Kind::Integer { .. })
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Kind::Boolean { .. })
    }

    pub fn is_template(&self) -> bool {
        matches!(self, Kind::Template { .. })
    }

    pub fn is_decimal(&self) -> bool {
        matches!(self, Kind::Decimal { .. })
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Kind::Void { .. })
    }

    pub fn get_or_type(&self) -> Option<(String, Option<String>, Option<String>)> {
        match self {
            Kind::OrType {
                name,
                variant,
                full_variant,
            } => Some((name.to_owned(), variant.to_owned(), full_variant.to_owned())),
            _ => None,
        }
    }

    pub fn get_record_name(&self) -> Option<&str> {
        match self {
            fastn_resolved::Kind::Record { ref name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn get_or_type_name(&self) -> Option<&str> {
        match self {
            fastn_resolved::Kind::OrType { ref name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn is_or_type_with_variant(&self, or_type_name: &str, variant_name: &str) -> bool {
        matches!(self, Kind::OrType { name, variant, .. } if name.eq(or_type_name) && variant.is_some() && variant.as_ref().unwrap().eq(variant_name))
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

    pub fn caption(self) -> KindData {
        let mut kind = self;
        kind.caption = true;
        kind
    }

    pub fn body(self) -> KindData {
        let mut kind = self;
        kind.body = true;
        kind
    }

    pub fn caption_or_body(self) -> KindData {
        let mut kind = self;
        kind.caption = true;
        kind.body = true;
        kind
    }

    pub fn is_list(&self) -> bool {
        self.kind.is_list()
    }

    pub fn is_or_type(&self) -> bool {
        self.kind.is_or_type()
    }

    pub fn is_optional(&self) -> bool {
        self.kind.is_optional()
    }

    pub fn into_optional(self) -> Self {
        KindData {
            caption: self.caption,
            body: self.body,
            kind: self.kind.into_optional(),
        }
    }

    pub fn is_string(&self) -> bool {
        self.kind.is_string()
    }

    pub fn is_module(&self) -> bool {
        self.kind.is_module()
    }

    pub fn is_integer(&self) -> bool {
        self.kind.is_integer()
    }

    pub fn is_record(&self) -> bool {
        self.kind.is_record()
    }

    pub fn is_boolean(&self) -> bool {
        self.kind.is_boolean()
    }

    pub fn is_subsection_ui(&self) -> bool {
        self.kind.is_subsection_ui()
    }

    pub fn is_ui(&self) -> bool {
        self.kind.is_ui()
    }

    pub fn is_decimal(&self) -> bool {
        self.kind.is_decimal()
    }

    pub fn is_void(&self) -> bool {
        self.kind.is_void()
    }

    pub fn is_kwargs(&self) -> bool {
        self.kind.is_kwargs()
    }

    pub fn optional(self) -> KindData {
        KindData {
            kind: Kind::Optional {
                kind: Box::new(self.kind),
            },
            caption: self.caption,
            body: self.body,
        }
    }

    pub fn list(self) -> KindData {
        KindData {
            kind: Kind::List {
                kind: Box::new(self.kind),
            },
            caption: self.caption,
            body: self.body,
        }
    }

    pub fn constant(self) -> KindData {
        KindData {
            kind: Kind::Constant {
                kind: Box::new(self.kind),
            },
            caption: self.caption,
            body: self.body,
        }
    }

    pub fn inner_list(self) -> KindData {
        let kind = match self.kind {
            Kind::List { kind } => kind.as_ref().to_owned(),
            t => t,
        };
        KindData {
            kind,
            caption: self.caption,
            body: self.body,
        }
    }

    pub fn inner(self) -> KindData {
        let kind = match self.kind {
            Kind::Optional { kind } => kind.as_ref().to_owned(),
            t => t,
        };
        KindData {
            kind,
            caption: self.caption,
            body: self.body,
        }
    }
}
