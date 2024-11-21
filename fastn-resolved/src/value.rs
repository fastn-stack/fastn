#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PropertyValue {
    Value {
        value: fastn_resolved::Value,
        is_mutable: bool,
        line_number: usize,
    },
    Reference {
        name: String,
        kind: fastn_resolved::KindData,
        source: fastn_resolved::PropertyValueSource,
        is_mutable: bool,
        line_number: usize,
    },
    Clone {
        name: String,
        kind: fastn_resolved::KindData,
        source: fastn_resolved::PropertyValueSource,
        is_mutable: bool,
        line_number: usize,
    },
    FunctionCall(fastn_resolved::FunctionCall),
}

impl PropertyValue {
    pub fn line_number(&self) -> usize {
        match self {
            PropertyValue::Value { line_number, .. }
            | PropertyValue::Reference { line_number, .. }
            | PropertyValue::Clone { line_number, .. }
            | PropertyValue::FunctionCall(fastn_resolved::FunctionCall { line_number, .. }) => {
                *line_number
            }
        }
    }

    pub fn is_mutable(&self) -> bool {
        match self {
            PropertyValue::Value { is_mutable, .. }
            | PropertyValue::Reference { is_mutable, .. }
            | PropertyValue::Clone { is_mutable, .. }
            | PropertyValue::FunctionCall(fastn_resolved::FunctionCall { is_mutable, .. }) => {
                *is_mutable
            }
        }
    }

    pub fn get_reference_or_clone(&self) -> Option<&String> {
        match self {
            PropertyValue::Reference { name, .. } | PropertyValue::Clone { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn reference_name(&self) -> Option<&String> {
        match self {
            PropertyValue::Reference { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn kind(&self) -> fastn_resolved::Kind {
        match self {
            PropertyValue::Value { value, .. } => value.kind(),
            PropertyValue::Reference { kind, .. } => kind.kind.to_owned(),
            PropertyValue::Clone { kind, .. } => kind.kind.to_owned(),
            PropertyValue::FunctionCall(fastn_resolved::FunctionCall { kind, .. }) => {
                kind.kind.to_owned()
            }
        }
    }

    pub fn set_reference_or_clone(&mut self, new_name: &str) {
        match self {
            PropertyValue::Reference { name, .. } | PropertyValue::Clone { name, .. } => {
                *name = new_name.to_string();
            }
            _ => {}
        }
    }

    pub fn is_value(&self) -> bool {
        matches!(self, fastn_resolved::PropertyValue::Value { .. })
    }

    pub fn is_clone(&self) -> bool {
        matches!(self, fastn_resolved::PropertyValue::Clone { .. })
    }

    pub fn get_function(&self) -> Option<&fastn_resolved::FunctionCall> {
        match self {
            PropertyValue::FunctionCall(f) => Some(f),
            _ => None,
        }
    }

    pub fn new_none(
        kind: fastn_resolved::KindData,
        line_number: usize,
    ) -> fastn_resolved::PropertyValue {
        fastn_resolved::PropertyValue::Value {
            value: fastn_resolved::Value::new_none(kind),
            is_mutable: false,
            line_number,
        }
    }

    pub fn value_optional(&self) -> Option<&fastn_resolved::Value> {
        match self {
            fastn_resolved::PropertyValue::Value { value, .. } => Some(value),
            _ => None,
        }
    }

    pub fn set_mutable(&mut self, mutable: bool) {
        match self {
            PropertyValue::Value { is_mutable, .. }
            | PropertyValue::Reference { is_mutable, .. }
            | PropertyValue::Clone { is_mutable, .. }
            | PropertyValue::FunctionCall(fastn_resolved::FunctionCall { is_mutable, .. }) => {
                *is_mutable = mutable;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PropertyValueSource {
    Global,
    Local(String),
    Loop(String),
}

impl PropertyValueSource {
    pub fn is_global(&self) -> bool {
        PropertyValueSource::Global.eq(self)
    }

    pub fn is_local(&self, name: &str) -> bool {
        matches!(self, PropertyValueSource::Local(l_name) if l_name.eq(name))
    }

    pub fn get_name(&self) -> Option<String> {
        match self {
            PropertyValueSource::Local(s) | PropertyValueSource::Loop(s) => Some(s.to_owned()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Value {
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
        values: fastn_resolved::Map<PropertyValue>,
    },
    Record {
        name: String,
        fields: fastn_resolved::Map<PropertyValue>,
    },
    KwArgs {
        arguments: fastn_resolved::Map<PropertyValue>,
    },
    OrType {
        name: String,
        variant: String,
        full_variant: String,
        value: Box<PropertyValue>, // Todo: Make it optional
    },
    List {
        data: Vec<PropertyValue>,
        kind: fastn_resolved::KindData,
    },
    Optional {
        data: Box<Option<Value>>,
        kind: fastn_resolved::KindData,
    },
    UI {
        name: String,
        kind: fastn_resolved::KindData,
        component: fastn_resolved::ComponentInvocation,
    },
    Module {
        name: String,
        things: fastn_resolved::Map<fastn_resolved::ModuleThing>,
    },
}

impl Value {
    pub fn new_none(kind: fastn_resolved::KindData) -> fastn_resolved::Value {
        fastn_resolved::Value::Optional {
            data: Box::new(None),
            kind,
        }
    }

    pub fn new_string(text: &str) -> fastn_resolved::Value {
        fastn_resolved::Value::String {
            text: text.to_string(),
        }
    }

    pub fn new_or_type(
        name: &str,
        variant: &str,
        full_variant: &str,
        value: fastn_resolved::PropertyValue,
    ) -> fastn_resolved::Value {
        fastn_resolved::Value::OrType {
            name: name.to_string(),
            variant: variant.to_string(),
            full_variant: full_variant.to_string(),
            value: Box::new(value),
        }
    }

    pub fn inner(&self) -> Option<Self> {
        match self {
            Value::Optional { data, .. } => data.as_ref().to_owned(),
            t => Some(t.to_owned()),
        }
    }

    pub fn into_property_value(self, is_mutable: bool, line_number: usize) -> PropertyValue {
        PropertyValue::Value {
            value: self,
            is_mutable,
            line_number,
        }
    }

    pub fn kind(&self) -> fastn_resolved::Kind {
        match self {
            Value::String { .. } => fastn_resolved::Kind::string(),
            Value::Integer { .. } => fastn_resolved::Kind::integer(),
            Value::Decimal { .. } => fastn_resolved::Kind::decimal(),
            Value::Boolean { .. } => fastn_resolved::Kind::boolean(),
            Value::Object { .. } => fastn_resolved::Kind::object(),
            Value::Record { name, .. } => fastn_resolved::Kind::record(name),
            Value::KwArgs { .. } => fastn_resolved::Kind::kwargs(),
            Value::List { kind, .. } => kind.kind.clone().into_list(),
            Value::Optional { kind, .. } => fastn_resolved::Kind::Optional {
                kind: Box::new(kind.kind.clone()),
            },
            Value::UI { name, .. } => fastn_resolved::Kind::ui_with_name(name),
            Value::OrType {
                name,
                variant,
                full_variant,
                ..
            } => fastn_resolved::Kind::or_type_with_variant(name, variant, full_variant),
            Value::Module { .. } => fastn_resolved::Kind::module(),
        }
    }

    pub fn is_record(&self, rec_name: &str) -> bool {
        matches!(self, Self::Record { name, .. } if rec_name.eq(name))
    }

    pub fn is_or_type_variant(&self, or_variant: &str) -> bool {
        matches!(self, Self::OrType { variant, .. } if or_variant.eq(variant))
    }

    pub fn ref_inner(&self) -> Option<&Self> {
        match self {
            Value::Optional { data, .. } => data.as_ref().as_ref(),
            t => Some(t),
        }
    }

    pub fn module_name_optional(&self) -> Option<String> {
        match self {
            fastn_resolved::Value::Module { name, .. } => Some(name.to_string()),
            _ => None,
        }
    }

    pub fn mut_module_optional(
        &mut self,
    ) -> Option<(&str, &mut fastn_resolved::Map<fastn_resolved::ModuleThing>)> {
        match self {
            fastn_resolved::Value::Module { name, things } => Some((name, things)),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        if let Self::String { text, .. } = self {
            return text.is_empty();
        }
        if let Self::Optional { data, .. } = self {
            let value = if let Some(fastn_resolved::Value::String { text, .. }) = data.as_ref() {
                text.is_empty()
            } else {
                false
            };
            if data.as_ref().eq(&None) || value {
                return true;
            }
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        if let Self::List { data, .. } = self {
            if data.is_empty() {
                return true;
            }
        }
        false
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        match (self.to_owned().inner(), other.to_owned().inner()) {
            (Some(Value::String { text: ref a, .. }), Some(Value::String { text: ref b, .. })) => {
                a == b
            }
            (a, b) => a == b,
        }
    }
}
