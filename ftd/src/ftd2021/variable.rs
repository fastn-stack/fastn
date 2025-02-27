#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: ftd::PropertyValue,
    pub conditions: Vec<(ftd::ftd2021::p2::Boolean, ftd::PropertyValue)>,
    pub flags: VariableFlags,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, Default, serde::Deserialize)]
pub struct VariableFlags {
    pub always_include: Option<bool>,
}

impl VariableFlags {
    pub(crate) fn from_p1(
        p1: &ftd::ftd2021::p1::Header,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::ftd2021::p1::Result<Self> {
        Ok(VariableFlags {
            always_include: p1.bool_optional(doc_id, line_number, "$always-include$")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum PropertyValue {
    Value {
        value: ftd::Value,
    },
    Reference {
        name: String,
        kind: ftd::ftd2021::p2::Kind,
    },
    Variable {
        name: String,
        kind: ftd::ftd2021::p2::Kind,
    },
}

impl PropertyValue {
    pub fn get_passed_by_variable(&self) -> Option<String> {
        match self {
            ftd::PropertyValue::Reference { name, kind }
            | ftd::PropertyValue::Variable { name, kind } => {
                if kind.is_reference() {
                    Some(name.to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn set_reference(&mut self) {
        match self {
            ftd::PropertyValue::Reference { kind, .. }
            | ftd::PropertyValue::Variable { kind, .. } => {
                *kind = kind.clone().set_reference(true);
            }
            _ => {}
        }
    }

    pub fn get_reference(&self) -> Option<String> {
        match self {
            ftd::PropertyValue::Reference { name, .. } => Some(name.to_string()),
            ftd::PropertyValue::Variable { name, .. } => Some(name.to_string()),
            _ => None,
        }
    }

    pub fn into_optional(self) -> Self {
        let mut s = self;
        match &mut s {
            PropertyValue::Value { value } => {
                *value = value.clone().into_optional();
            }
            PropertyValue::Reference { kind, .. } | PropertyValue::Variable { kind, .. } => {
                *kind = ftd::ftd2021::p2::Kind::Optional {
                    kind: Box::new(kind.clone()),
                    is_reference: false,
                };
            }
        }
        s
    }

    pub fn resolve_value(
        line_number: usize,
        value: &str,
        expected_kind: Option<ftd::ftd2021::p2::Kind>,
        doc: &ftd::ftd2021::p2::TDoc,
        arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
        source: Option<ftd::TextSource>,
    ) -> ftd::ftd2021::p1::Result<ftd::PropertyValue> {
        let property_type = if let Some(arg) = value.strip_prefix('$') {
            PropertyType::Variable(arg.to_string())
        } else if let Some(ftd::ftd2021::p2::Kind::UI { .. }) =
            expected_kind.as_ref().map(|v| v.inner())
        {
            if !value.contains(':') {
                return ftd::ftd2021::p2::utils::e2(
                    format!("expected `:`, found: `{}`", value),
                    doc.name,
                    line_number,
                );
            }
            let name = ftd::ftd2021::p2::utils::split(value.to_string(), ":")?.0;
            PropertyType::Component { name }
        } else {
            let value = if let Some(value) = value.strip_prefix('\\') {
                value.to_string()
            } else {
                value.to_string()
            };
            PropertyType::Value(value)
        };

        let (part1, mut part2) =
            ftd::ftd2021::p2::utils::get_doc_name_and_remaining(&property_type.string())?;

        return Ok(match property_type {
            PropertyType::Variable(ref string)
            | PropertyType::Component {
                name: ref string, ..
            } => {
                let (kind, is_doc) = match arguments.get(&part1) {
                    _ if part1.eq("MOUSE-IN") => (
                        ftd::ftd2021::p2::Kind::Boolean {
                            default: Some("false".to_string()),
                            is_reference: false,
                        },
                        false,
                    ),
                    _ if part1.eq("SIBLING-INDEX") || part1.eq("SIBLING-INDEX-0") => (
                        ftd::ftd2021::p2::Kind::Integer {
                            default: None,
                            is_reference: false,
                        },
                        false,
                    ),
                    _ if part1.eq("CHILDREN-COUNT") => (
                        ftd::ftd2021::p2::Kind::Integer {
                            default: Some("0".to_string()),
                            is_reference: false,
                        },
                        false,
                    ),
                    _ if part1.eq("CHILDREN-COUNT-MINUS-ONE") => (
                        ftd::ftd2021::p2::Kind::Integer {
                            default: Some("-1".to_string()),
                            is_reference: false,
                        },
                        false,
                    ),
                    _ if part1.eq("PARENT") => {
                        let kind = if part2.eq(&Some("CHILDREN-COUNT".to_string())) {
                            ftd::ftd2021::p2::Kind::Integer {
                                default: Some("0".to_string()),
                                is_reference: false,
                            }
                        } else if part2.eq(&Some("CHILDREN-COUNT-MINUS-ONE".to_string())) {
                            ftd::ftd2021::p2::Kind::Integer {
                                default: Some("-1".to_string()),
                                is_reference: false,
                            }
                        } else if let Some(ref kind) = expected_kind {
                            kind.clone()
                        } else {
                            return ftd::ftd2021::p2::utils::e2(
                                format!("{}.{:?} expected kind for parent variable", part1, part2),
                                doc.name,
                                line_number,
                            );
                        };
                        part2 = None;

                        (kind, false)
                    }
                    None => match doc.get_initial_thing(line_number, string) {
                        Ok((ftd::ftd2021::p2::Thing::Variable(v), name)) => {
                            part2 = name;
                            (v.value.kind(), true)
                        }
                        Ok((ftd::ftd2021::p2::Thing::Component(_), name)) => {
                            part2 = name;
                            (ftd::ftd2021::p2::Kind::UI { default: None }, true)
                        }
                        e => {
                            return ftd::ftd2021::p2::utils::e2(
                                format!("{} is not present in doc, {:?}", part1, e),
                                doc.name,
                                line_number,
                            );
                        }
                    },
                    Some(kind) => (kind.to_owned(), false),
                };

                let found_kind = get_kind(line_number, &kind, part2, doc, &expected_kind)?;
                if is_doc {
                    PropertyValue::Reference {
                        name: doc
                            .resolve_name(line_number, string.as_str())
                            .unwrap_or_else(|_| string.to_string()),
                        kind: found_kind,
                    }
                } else {
                    PropertyValue::Variable {
                        name: string.to_string(),
                        kind: found_kind,
                    }
                }
            }
            PropertyType::Value(string) => {
                if expected_kind.is_none() {
                    return ftd::ftd2021::p2::utils::e2(
                        "expected expected_kind while calling resolve_value",
                        doc.name,
                        line_number,
                    );
                }
                let expected_kind = expected_kind.unwrap();
                match expected_kind.inner() {
                    ftd::ftd2021::p2::Kind::Integer { .. } => ftd::PropertyValue::Value {
                        value: ftd::Value::Integer {
                            value: string.parse::<i64>().map_err(|e| {
                                ftd::ftd2021::p1::Error::ParseError {
                                    message: e.to_string(),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                }
                            })?,
                        },
                    },
                    ftd::ftd2021::p2::Kind::Decimal { .. } => ftd::PropertyValue::Value {
                        value: ftd::Value::Decimal {
                            value: string.parse::<f64>().map_err(|e| {
                                ftd::ftd2021::p1::Error::ParseError {
                                    message: e.to_string(),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                }
                            })?,
                        },
                    },
                    ftd::ftd2021::p2::Kind::Boolean { .. } => ftd::PropertyValue::Value {
                        value: ftd::Value::Boolean {
                            value: string.parse::<bool>().map_err(|e| {
                                ftd::ftd2021::p1::Error::ParseError {
                                    message: e.to_string(),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                }
                            })?,
                        },
                    },
                    ftd::ftd2021::p2::Kind::String { .. } => ftd::PropertyValue::Value {
                        value: ftd::Value::String {
                            text: string,
                            source: source.unwrap_or(ftd::TextSource::Header),
                        },
                    },
                    t => {
                        return ftd::ftd2021::p2::utils::e2(
                            format!("can't resolve value {} to expected kind {:?}", string, t),
                            doc.name,
                            line_number,
                        );
                    }
                }
            }
        });

        #[derive(Debug)]
        enum PropertyType {
            Value(String),
            Variable(String),
            Component { name: String },
        }

        impl PropertyType {
            fn string(&self) -> String {
                match self {
                    PropertyType::Value(s)
                    | PropertyType::Variable(s)
                    | PropertyType::Component { name: s, .. } => s.to_string(),
                }
            }
        }

        fn get_kind(
            line_number: usize,
            kind: &ftd::ftd2021::p2::Kind,
            p2: Option<String>,
            doc: &ftd::ftd2021::p2::TDoc,
            expected_kind: &Option<ftd::ftd2021::p2::Kind>,
        ) -> ftd::ftd2021::p1::Result<ftd::ftd2021::p2::Kind> {
            let mut found_kind = kind.to_owned();
            if let Some(ref p2) = p2 {
                let (name, fields) = match kind.inner() {
                    ftd::ftd2021::p2::Kind::Record { name, .. } => (
                        name.to_string(),
                        doc.get_record(line_number, &doc.resolve_name(line_number, name)?)?
                            .fields,
                    ),
                    ftd::ftd2021::p2::Kind::OrTypeWithVariant { name, variant, .. } => {
                        let name = doc.resolve_name(line_number, name)?;
                        (
                            name.to_string(),
                            doc.get_or_type(line_number, &name)?
                                .variants
                                .into_iter()
                                .find(|v| v.name.eq(&format!("{}.{}", name, variant)))
                                .ok_or_else(|| ftd::ftd2021::p1::Error::ParseError {
                                    message: format!(
                                        "expected variant `{}` in or_type `{}`",
                                        variant, name
                                    ),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                })?
                                .fields,
                        )
                    }
                    _ => Default::default(),
                };
                let mut p1 = p2.to_string();
                let mut p2 = None;
                if p1.contains('.') {
                    let split_txt = ftd::ftd2021::p2::utils::split(p1.to_string(), ".")?;
                    p1 = split_txt.0;
                    p2 = Some(split_txt.1);
                }
                found_kind = match fields.get(p1.as_str()) {
                    Some(kind) if p2.is_some() => {
                        get_kind(line_number, kind, p2, doc, expected_kind)?
                    }
                    Some(kind) => kind.to_owned(),
                    _ => {
                        return ftd::ftd2021::p2::utils::e2(
                            format!("{} is not present in {} of type {:?}", p1, name, fields),
                            doc.name,
                            line_number,
                        );
                    }
                };
            }
            if let Some(e_kind) = expected_kind {
                if !e_kind.is_same_as(&found_kind)
                    && !matches!(e_kind, ftd::ftd2021::p2::Kind::Element)
                {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected {:?} found {:?}", e_kind, found_kind),
                        doc.name,
                        line_number,
                    );
                }
                return Ok(e_kind.to_owned().set_default({
                    if found_kind.get_default_value_str().is_some() {
                        found_kind.get_default_value_str()
                    } else {
                        e_kind.get_default_value_str()
                    }
                }));
            }
            Ok(found_kind)
        }
    }

    pub fn kind(&self) -> ftd::ftd2021::p2::Kind {
        match self {
            Self::Value { value: v } => v.kind(),
            Self::Reference { kind, .. } => kind.to_owned(),
            Self::Variable { kind, .. } => kind.to_owned(),
        }
    }

    /// resolves all the internal fields too
    pub fn resolve(
        &self,
        line_number: usize,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<Value> {
        let mut value = self.partial_resolve(line_number, doc)?;
        // In case of Object resolve all the values
        if let ftd::Value::Object { values } = &mut value {
            for (_, v) in values.iter_mut() {
                *v = ftd::PropertyValue::Value {
                    value: v.partial_resolve(line_number, doc)?,
                };
            }
        }
        Ok(value)
    }

    pub fn partial_resolve(
        &self,
        line_number: usize,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<Value> {
        Ok(match self {
            ftd::PropertyValue::Value { value: v } => v.to_owned(),
            ftd::PropertyValue::Variable {
                name: reference_name,
                kind: reference_kind,
            }
            | ftd::PropertyValue::Reference {
                name: reference_name,
                kind: reference_kind,
            } => {
                assert_eq!(self.kind(), *reference_kind);
                let (default, condition) =
                    if let Ok(d) = doc.get_value_and_conditions(0, reference_name.as_str()) {
                        d
                    } else if let Ok(d) = doc.get_component(0, reference_name.as_str()) {
                        return d.to_value(reference_kind);
                    } else {
                        return reference_kind.to_value(line_number, doc.name);
                    };
                let mut value = default;
                for (boolean, property) in condition {
                    if boolean.eval(line_number, doc)? {
                        value = property;
                    }
                }
                value
            }
        })
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum TextSource {
    Header,
    Caption,
    Body,
    Default,
}

impl TextSource {
    pub fn from_kind(
        kind: &ftd::ftd2021::p2::Kind,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::ftd2021::p1::Result<Self> {
        Ok(match kind {
            ftd::ftd2021::p2::Kind::String { caption, body, .. } => {
                if *caption {
                    TextSource::Caption
                } else if *body {
                    TextSource::Body
                } else {
                    TextSource::Header
                }
            }
            ftd::ftd2021::p2::Kind::Element => TextSource::Header,
            t => {
                return ftd::ftd2021::p2::utils::e2(
                    format!("expected string kind, found: {:?}", t),
                    doc_id,
                    line_number,
                );
            }
        })
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
pub enum Value {
    None {
        kind: ftd::ftd2021::p2::Kind,
    },
    String {
        text: String,
        source: ftd::TextSource,
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
        kind: ftd::ftd2021::p2::Kind,
    },
    Optional {
        data: Box<Option<Value>>,
        kind: ftd::ftd2021::p2::Kind,
    },
    Map {
        data: ftd::Map<Value>,
        kind: ftd::ftd2021::p2::Kind,
    },
    UI {
        name: String,
        kind: ftd::ftd2021::p2::Kind,
        data: ftd::Map<ftd::ftd2021::component::Property>,
    },
}

impl Value {
    /// returns a default optional value from given kind
    pub fn default_optional_value_from_kind(kind: ftd::ftd2021::p2::Kind) -> Self {
        Value::Optional {
            data: Box::new(None),
            kind,
        }
    }

    pub fn inner_with_none(self) -> Self {
        match self {
            ftd::Value::Optional { data, kind } => data
                .as_ref()
                .as_ref()
                .map(|d| d.to_owned())
                .unwrap_or(ftd::Value::None { kind }),
            _ => self,
        }
    }

    pub fn inner(self) -> Option<Self> {
        match self {
            ftd::Value::Optional { data, .. } => data.as_ref().as_ref().map(|d| d.to_owned()),
            _ => Some(self),
        }
    }

    pub fn into_optional(self) -> ftd::Value {
        ftd::Value::Optional {
            kind: self.kind(),
            data: Box::new(Some(self)),
        }
    }
    pub fn is_null(&self) -> bool {
        if matches!(self, Self::None { .. }) {
            return true;
        }
        if let Self::String { text, .. } = self {
            return text.is_empty();
        }
        if let Self::Optional { data, .. } = self {
            let value = if let Some(ftd::Value::String { text, .. }) = data.as_ref() {
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

    pub fn is_optional(&self) -> bool {
        if matches!(self, Self::Optional { .. }) {
            return true;
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

    pub fn kind(&self) -> ftd::ftd2021::p2::Kind {
        match self {
            Value::None { kind: k } => k.to_owned(),
            Value::String { source, .. } => ftd::ftd2021::p2::Kind::String {
                caption: *source == TextSource::Caption,
                body: *source == TextSource::Body,
                default: None,
                is_reference: false,
            },
            Value::Integer { .. } => ftd::ftd2021::p2::Kind::integer(),
            Value::Decimal { .. } => ftd::ftd2021::p2::Kind::decimal(),
            Value::Boolean { .. } => ftd::ftd2021::p2::Kind::boolean(),
            Value::Object { .. } => ftd::ftd2021::p2::Kind::object(),
            Value::Record { name: id, .. } => ftd::ftd2021::p2::Kind::Record {
                name: id.to_string(),
                default: None,
                is_reference: false,
            },
            Value::OrType {
                name: id, variant, ..
            } => ftd::ftd2021::p2::Kind::OrTypeWithVariant {
                name: id.to_string(),
                variant: variant.to_string(),
                is_reference: false,
            },
            Value::List { kind, .. } => ftd::ftd2021::p2::Kind::List {
                kind: Box::new(kind.to_owned()),
                default: None,
                is_reference: false,
            },
            Value::Optional { kind, .. } => ftd::ftd2021::p2::Kind::Optional {
                kind: Box::new(kind.to_owned()),
                is_reference: false,
            },
            Value::Map { kind, .. } => ftd::ftd2021::p2::Kind::Map {
                kind: Box::new(kind.to_owned()),
                is_reference: false,
            },
            Value::UI { kind, .. } => kind.to_owned(),
        }
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        match (self.to_owned().inner(), other.to_owned().inner()) {
            (Some(Value::String { text: ref a, .. }), Some(Value::String { text: ref b, .. })) => {
                a == b
            }
            (a, b) => a == b,
        }
    }

    pub fn to_serde_value(&self) -> Option<serde_json::Value> {
        match self {
            Value::String { text, .. } => Some(serde_json::Value::String(text.to_string())),
            Value::Integer { value } => Some(serde_json::json!(value)),
            Value::Decimal { value } => Some(serde_json::json!(value)),
            Value::Boolean { value } => Some(serde_json::Value::Bool(value.to_owned())),
            Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    data.to_serde_value()
                } else {
                    Some(serde_json::Value::Null)
                }
            }
            Value::None { .. } => Some(serde_json::Value::Null),
            Value::Object { values } => {
                let mut new_values: ftd::Map<serde_json::Value> = Default::default();
                for (k, v) in values {
                    if let ftd::PropertyValue::Value { value } = v {
                        if let Some(v) = value.to_serde_value() {
                            new_values.insert(k.to_owned(), v);
                        }
                    }
                }
                serde_json::to_value(&new_values).ok()
            }
            Value::Record { fields, .. } => {
                let mut new_values: ftd::Map<serde_json::Value> = Default::default();
                for (k, v) in fields {
                    if let ftd::PropertyValue::Value { value } = v {
                        if let Some(v) = value.to_serde_value() {
                            new_values.insert(k.to_owned(), v);
                        }
                    }
                }
                serde_json::to_value(&new_values).ok()
            }
            _ => None,
        }
    }

    pub fn to_string(&self) -> Option<String> {
        match self {
            Value::String { text, .. } => Some(text.to_string()),
            Value::Integer { value } => Some(value.to_string()),
            Value::Decimal { value } => Some(value.to_string()),
            Value::Boolean { value } => Some(value.to_string()),
            Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    data.to_string()
                } else {
                    Some("".to_string())
                }
            }
            Value::None { .. } => Some("".to_string()),
            Value::Object { values } => {
                let mut new_values: ftd::Map<String> = Default::default();
                for (k, v) in values {
                    if let ftd::PropertyValue::Value { value } = v {
                        if let Some(v) = value.to_string() {
                            new_values.insert(k.to_owned(), v);
                        }
                    }
                }
                serde_json::to_string(&new_values).ok()
            }
            Value::Record { fields, .. } => {
                let mut new_values: ftd::Map<String> = Default::default();
                for (k, v) in fields {
                    if let ftd::PropertyValue::Value { value } = v {
                        if let Some(v) = value.to_string() {
                            new_values.insert(k.to_owned(), v);
                        }
                    }
                }
                serde_json::to_string(&new_values).ok()
            }
            _ => None,
        }
    }
}

impl Variable {
    pub fn list_from_p1(
        p1: &ftd::ftd2021::p1::Section,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<Self> {
        let var_data = ftd::ftd2021::variable::VariableData::get_name_kind(
            &p1.name,
            doc,
            p1.line_number,
            vec![].as_slice(),
        )?;
        let name = doc.resolve_name(p1.line_number, &var_data.name)?;
        let kind = ftd::ftd2021::p2::Kind::for_variable(
            p1.line_number,
            &p1.name,
            None,
            doc,
            None,
            &Default::default(),
        )?;
        if !kind.is_list() {
            return ftd::ftd2021::p2::utils::e2(
                format!("Expected list found: {:?}", p1),
                doc.name,
                p1.line_number,
            );
        }
        if let Some(ref caption) = p1.caption {
            if let Some(text) = caption.strip_prefix('$') {
                return Ok(Variable {
                    name,
                    value: ftd::PropertyValue::Reference {
                        name: doc.resolve_name(p1.line_number, text)?,
                        kind: ftd::ftd2021::p2::Kind::List {
                            kind: Box::new(kind.list_kind().to_owned()),
                            default: None,
                            is_reference: false,
                        },
                    },
                    conditions: vec![],
                    flags: ftd::ftd2021::variable::VariableFlags::from_p1(
                        &p1.header,
                        doc.name,
                        p1.line_number,
                    )?,
                });
            }
        }

        Ok(Variable {
            name,
            value: ftd::PropertyValue::Value {
                value: Value::List {
                    data: Default::default(),
                    kind: kind.list_kind().to_owned(),
                },
            },
            conditions: vec![],
            flags: ftd::ftd2021::variable::VariableFlags::from_p1(
                &p1.header,
                doc.name,
                p1.line_number,
            )?,
        })
    }

    pub fn map_from_p1(
        p1: &ftd::ftd2021::p1::Section,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<Self> {
        let name = doc.resolve_name(
            p1.line_number,
            ftd::ftd2021::p2::utils::get_name("map", p1.name.as_str(), doc.name)?,
        )?;
        Ok(Variable {
            name,
            value: ftd::PropertyValue::Value {
                value: Value::Map {
                    data: Default::default(),
                    kind: ftd::ftd2021::p2::Kind::from(
                        p1.line_number,
                        p1.header.str(doc.name, p1.line_number, "type")?,
                        doc,
                        None,
                    )?,
                },
            },
            conditions: vec![],
            flags: ftd::ftd2021::variable::VariableFlags::from_p1(
                &p1.header,
                doc.name,
                p1.line_number,
            )?,
        })
    }

    pub fn update_from_p1(
        &mut self,
        p1: &ftd::ftd2021::p1::Section,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<()> {
        fn read_value(
            line_number: usize,
            kind: &ftd::ftd2021::p2::Kind,
            p1: &ftd::ftd2021::p1::Section,
            doc: &ftd::ftd2021::p2::TDoc,
        ) -> ftd::ftd2021::p1::Result<ftd::PropertyValue> {
            Ok(match kind {
                ftd::ftd2021::p2::Kind::Integer { .. } => read_integer(p1, doc)?,
                ftd::ftd2021::p2::Kind::Decimal { .. } => read_decimal(p1, doc)?,
                ftd::ftd2021::p2::Kind::Boolean { .. } => read_boolean(p1, doc)?,
                ftd::ftd2021::p2::Kind::String { .. } => read_string(p1, doc)?,
                ftd::ftd2021::p2::Kind::Record { name, .. } => {
                    doc.get_record(line_number, name)?.create(p1, doc)?
                }
                _ => unimplemented!("{:?}", kind),
            })
        }

        let p1 = {
            let mut p1 = p1.clone();
            p1.name = if let Some(n) = p1.name.strip_prefix('$') {
                n.to_string()
            } else {
                p1.name
            };
            p1
        };

        match (
            &self.value.kind(),
            &self.value.resolve(p1.line_number, doc)?,
        ) {
            (ftd::ftd2021::p2::Kind::Record { name, .. }, _) => {
                self.value = doc.get_record(p1.line_number, name)?.create(&p1, doc)?
            }
            (ftd::ftd2021::p2::Kind::List { kind, .. }, ftd::Value::List { data, .. }) => {
                let mut data = data.clone();
                data.push(read_value(p1.line_number, kind, &p1, doc)?);
                self.value = ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data,
                        kind: kind.as_ref().clone(),
                    },
                };
            }
            (ftd::ftd2021::p2::Kind::Optional { kind, .. }, ftd::Value::Optional { .. }) => {
                self.value = read_value(p1.line_number, kind, &p1, doc)
                    .map(|v| v.into_optional())
                    .unwrap_or(ftd::PropertyValue::Value {
                        value: ftd::Value::Optional {
                            data: Box::new(None),
                            kind: kind.as_ref().inner().clone(),
                        },
                    });
            }
            (ftd::ftd2021::p2::Kind::Map { .. }, _) => {
                return ftd::ftd2021::p2::utils::e2("unexpected map", doc.name, p1.line_number);
            }
            (k, _) => self.value = read_value(p1.line_number, k, &p1, doc)?,
        };

        Ok(())
    }

    pub fn from_p1(
        p1: &ftd::ftd2021::p1::Section,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<Self> {
        let var_data = ftd::ftd2021::variable::VariableData::get_name_kind(
            &p1.name,
            doc,
            p1.line_number,
            vec![].as_slice(),
        )?;
        if !var_data.is_variable() {
            return ftd::ftd2021::p2::utils::e2(
                format!("expected variable, found: {}", p1.name),
                doc.name,
                p1.line_number,
            );
        }
        let name = var_data.name.clone();

        if var_data.is_optional() && p1.caption.is_none() && p1.body.is_none() {
            let kind = ftd::ftd2021::p2::Kind::for_variable(
                p1.line_number,
                &p1.name,
                None,
                doc,
                None,
                &Default::default(),
            )?;
            return Ok(Variable {
                name,
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: kind.inner().to_owned(),
                    },
                },
                conditions: vec![],
                flags: ftd::ftd2021::variable::VariableFlags::from_p1(
                    &p1.header,
                    doc.name,
                    p1.line_number,
                )?,
            });
        }

        let value = {
            let mut value = match var_data.kind.as_str() {
                "string" => read_string(p1, doc)?,
                "integer" => read_integer(p1, doc)?,
                "decimal" => read_decimal(p1, doc)?,
                "boolean" => read_boolean(p1, doc)?,
                "object" => read_object(p1, doc)?,
                t => match doc.get_thing(p1.line_number, t)? {
                    ftd::ftd2021::p2::Thing::Record(r) => r.create(p1, doc)?,
                    ftd::ftd2021::p2::Thing::OrTypeWithVariant { e, variant } => {
                        e.create(p1, variant, doc)?
                    }
                    t => {
                        return ftd::ftd2021::p2::utils::e2(
                            format!("unexpected thing found: {:?}", t),
                            doc.name,
                            p1.line_number,
                        );
                    }
                },
            };
            if var_data.is_optional() {
                let kind = ftd::ftd2021::p2::Kind::for_variable(
                    p1.line_number,
                    &p1.name,
                    None,
                    doc,
                    None,
                    &Default::default(),
                )?;
                //todo: use into_optional
                match &mut value {
                    PropertyValue::Value { value } => {
                        *value = ftd::Value::Optional {
                            data: Box::new(Some(value.clone())),
                            kind: kind.inner().to_owned(),
                        };
                    }
                    PropertyValue::Reference { kind: k, .. }
                    | PropertyValue::Variable { kind: k, .. } => {
                        *k = kind;
                    }
                }
            }
            value
        };

        Ok(Variable {
            name,
            value,
            conditions: vec![],
            flags: ftd::ftd2021::variable::VariableFlags::from_p1(
                &p1.header,
                doc.name,
                p1.line_number,
            )?,
        })
    }

    pub fn get_value(
        &self,
        p1: &ftd::ftd2021::p1::Section,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<ftd::PropertyValue> {
        match self.value.kind().inner() {
            ftd::ftd2021::p2::Kind::String { .. } => read_string(p1, doc),
            ftd::ftd2021::p2::Kind::Integer { .. } => read_integer(p1, doc),
            ftd::ftd2021::p2::Kind::Decimal { .. } => read_decimal(p1, doc),
            ftd::ftd2021::p2::Kind::Boolean { .. } => read_boolean(p1, doc),
            ftd::ftd2021::p2::Kind::Record { name, .. } => {
                match doc.get_thing(p1.line_number, name)? {
                    ftd::ftd2021::p2::Thing::Record(r) => r.create(p1, doc),
                    t => ftd::ftd2021::p2::utils::e2(
                        format!("expected record type, found: {:?}", t),
                        doc.name,
                        p1.line_number,
                    ),
                }
            }
            ftd::ftd2021::p2::Kind::OrType { name, .. }
            | ftd::ftd2021::p2::Kind::OrTypeWithVariant { name, .. } => {
                match doc.get_thing(p1.line_number, name)? {
                    ftd::ftd2021::p2::Thing::OrTypeWithVariant { e, variant } => {
                        e.create(p1, variant, doc)
                    }
                    t => ftd::ftd2021::p2::utils::e2(
                        format!("expected or-type type, found: {:?}", t),
                        doc.name,
                        p1.line_number,
                    ),
                }
            }
            t => ftd::ftd2021::p2::utils::e2(
                format!("unexpected type found: {:?}", t),
                doc.name,
                p1.line_number,
            ),
        }
    }
}

pub fn guess_type(s: &str, is_body: bool) -> ftd::ftd2021::p1::Result<Value> {
    if is_body {
        return Ok(Value::String {
            text: s.to_string(),
            source: TextSource::Body,
        });
    }
    let caption = match s {
        "true" => return Ok(Value::Boolean { value: true }),
        "false" => return Ok(Value::Boolean { value: false }),
        v => v,
    };

    if let Ok(v) = caption.parse::<i64>() {
        return Ok(Value::Integer { value: v });
    }

    if let Ok(v) = caption.parse::<f64>() {
        return Ok(Value::Decimal { value: v });
    }

    Ok(Value::String {
        text: caption.to_string(),
        source: TextSource::Caption,
    })
}

fn read_string(
    p1: &ftd::ftd2021::p1::Section,
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<ftd::PropertyValue> {
    let (text, source, line_number) = match (&p1.caption, &p1.body) {
        (Some(c), Some(b)) => {
            return ftd::ftd2021::p2::utils::e2(
                format!("both caption: `{}` and body: `{}` present", c, b.1),
                doc.name,
                p1.line_number,
            );
        }
        (Some(caption), None) => (caption.to_string(), TextSource::Caption, p1.line_number),
        (None, Some(body)) => (body.1.to_string(), TextSource::Body, body.0),
        (None, None) => {
            return ftd::ftd2021::p2::utils::e2(
                "either body or caption is required for string",
                doc.name,
                p1.line_number,
            );
        }
    };
    Ok(if let Some(text) = text.strip_prefix('$') {
        ftd::PropertyValue::Reference {
            name: doc.resolve_name(line_number, text)?,
            kind: ftd::ftd2021::p2::Kind::String {
                caption: source.eq(&ftd::TextSource::Caption),
                body: source.eq(&ftd::TextSource::Body),
                default: None,
                is_reference: false,
            },
        }
    } else {
        ftd::PropertyValue::Value {
            value: Value::String { text, source },
        }
    })
}

fn read_integer(
    p1: &ftd::ftd2021::p1::Section,
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<ftd::PropertyValue> {
    let caption = p1.caption(p1.line_number, doc.name)?;
    Ok(if let Some(text) = caption.strip_prefix('$') {
        ftd::PropertyValue::Reference {
            name: doc.resolve_name(p1.line_number, text)?,
            kind: ftd::ftd2021::p2::Kind::Integer {
                default: None,
                is_reference: false,
            },
        }
    } else {
        if let Ok(v) = caption.parse::<i64>() {
            return Ok(ftd::PropertyValue::Value {
                value: Value::Integer { value: v },
            });
        }
        return ftd::ftd2021::p2::utils::e2("not a valid integer", doc.name, p1.line_number);
    })
}

fn read_decimal(
    p1: &ftd::ftd2021::p1::Section,
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<ftd::PropertyValue> {
    let caption = p1.caption(p1.line_number, doc.name)?;
    Ok(if let Some(text) = caption.strip_prefix('$') {
        ftd::PropertyValue::Reference {
            name: doc.resolve_name(p1.line_number, text)?,
            kind: ftd::ftd2021::p2::Kind::Integer {
                default: None,
                is_reference: false,
            },
        }
    } else {
        if let Ok(v) = caption.parse::<f64>() {
            return Ok(ftd::PropertyValue::Value {
                value: Value::Decimal { value: v },
            });
        }
        return ftd::ftd2021::p2::utils::e2("not a valid float", doc.name, p1.line_number);
    })
}

fn read_boolean(
    p1: &ftd::ftd2021::p1::Section,
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<ftd::PropertyValue> {
    let caption = p1.caption(p1.line_number, doc.name)?;
    Ok(if let Some(text) = caption.strip_prefix('$') {
        ftd::PropertyValue::Reference {
            name: doc.resolve_name(p1.line_number, text)?,
            kind: ftd::ftd2021::p2::Kind::Integer {
                default: None,
                is_reference: false,
            },
        }
    } else {
        if let Ok(v) = caption.parse::<bool>() {
            return Ok(ftd::PropertyValue::Value {
                value: Value::Boolean { value: v },
            });
        }
        return ftd::ftd2021::p2::utils::e2("not a valid bool", doc.name, p1.line_number);
    })
}

fn read_object(
    p1: &ftd::ftd2021::p1::Section,
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<ftd::PropertyValue> {
    let mut values: ftd::Map<PropertyValue> = Default::default();
    if let Some(ref caption) = p1.caption {
        if let Some(text) = caption.strip_prefix('$') {
            return Ok(ftd::PropertyValue::Reference {
                name: doc.resolve_name(p1.line_number, text)?,
                kind: ftd::ftd2021::p2::Kind::Object {
                    default: None,
                    is_reference: false,
                },
            });
        }
    }
    for (line_number, k, v) in p1.header.0.iter() {
        let line_number = line_number.to_owned();
        let value = if v.trim().starts_with('$') {
            ftd::PropertyValue::resolve_value(line_number, v, None, doc, &Default::default(), None)?
        } else if let Ok(v) = ftd::PropertyValue::resolve_value(
            line_number,
            v,
            Some(ftd::ftd2021::p2::Kind::decimal()),
            doc,
            &Default::default(),
            None,
        ) {
            v
        } else if let Ok(v) = ftd::PropertyValue::resolve_value(
            line_number,
            v,
            Some(ftd::ftd2021::p2::Kind::boolean()),
            doc,
            &Default::default(),
            None,
        ) {
            v
        } else if let Ok(v) = ftd::PropertyValue::resolve_value(
            line_number,
            v,
            Some(ftd::ftd2021::p2::Kind::integer()),
            doc,
            &Default::default(),
            None,
        ) {
            v
        } else {
            ftd::PropertyValue::resolve_value(
                line_number,
                v,
                Some(ftd::ftd2021::p2::Kind::string()),
                doc,
                &Default::default(),
                None,
            )?
        };
        values.insert(k.to_string(), value);
    }

    Ok(ftd::PropertyValue::Value {
        value: Value::Object { values },
    })
}

#[derive(Debug, Clone)]
pub struct VariableData {
    pub name: String,
    pub kind: String,
    pub modifier: VariableModifier,
    pub type_: Type,
    pub is_reference: bool,
}

#[derive(Debug, Clone)]
pub enum VariableModifier {
    None,
    List,
    Optional,
}

#[derive(Debug, Clone)]
pub enum Type {
    Variable,
    Component,
}

impl VariableData {
    pub fn get_name_kind(
        s: &str,
        doc: &ftd::ftd2021::p2::TDoc,
        line_number: usize,
        var_types: &[String],
    ) -> ftd::ftd2021::p1::Result<VariableData> {
        if s.starts_with("record ")
            || s.starts_with("or-type ")
            || s.starts_with("map ")
            || s == "container"
        {
            return ftd::ftd2021::p2::utils::e2(
                format!("invalid declaration, found: `{}`", s),
                doc.name,
                line_number,
            );
        }
        let expr = s.split_whitespace().collect::<Vec<&str>>();
        if expr.len() > 4 || expr.len() <= 1 {
            return ftd::ftd2021::p2::utils::e2(
                format!("invalid declaration, found: `{}`", s),
                doc.name,
                line_number,
            );
        }
        let mut name = expr.get(1);
        let mut kind = expr.get(0).map(|k| k.to_string());
        let mut modifier = VariableModifier::None;
        if expr.len() == 4 {
            if expr.get(1).unwrap().eq(&"or") {
                kind = Some(expr[..3].join(" "));
                name = expr.get(3);
            } else {
                return ftd::ftd2021::p2::utils::e2(
                    format!("invalid variable or list declaration, found: `{}`", s),
                    doc.name,
                    line_number,
                );
            }
        } else if expr.len() == 3 {
            if expr.get(1).unwrap().eq(&"list") {
                modifier = VariableModifier::List;
                name = expr.get(2);
                kind = expr.get(0).map(|k| k.to_string());
            } else if expr.get(0).unwrap().eq(&"optional") {
                modifier = VariableModifier::Optional;
                name = expr.get(2);
                kind = expr.get(1).map(|k| k.to_string());
            } else {
                return ftd::ftd2021::p2::utils::e2(
                    format!("invalid variable or list declaration, found: `{}`", s),
                    doc.name,
                    line_number,
                );
            }
        }

        let var_kind = kind.ok_or(ftd::ftd2021::p1::Error::ParseError {
            message: format!("kind not found `{}`", s),
            doc_id: doc.name.to_string(),
            line_number,
        })?;

        let type_ = match var_kind.as_str() {
            "string" | "caption" | "body" | "body or caption" | "caption or body" | "integer"
            | "decimal" | "boolean" | "object" => Type::Variable,
            a if doc.get_record(line_number, a).is_ok()
                || doc.get_or_type(line_number, a).is_ok()
                || doc.get_or_type_with_variant(line_number, a).is_ok()
                || var_types.contains(&a.to_string()) =>
            {
                Type::Variable
            }
            _ => Type::Component,
        };

        let name = name.ok_or(ftd::ftd2021::p1::Error::ParseError {
            message: format!("name not found `{}`", s),
            doc_id: doc.name.to_string(),
            line_number,
        })?;

        let (name, is_reference) = if let Some(name) = name.strip_prefix('$') {
            (name.to_string(), true)
        } else {
            (name.to_string(), false)
        };

        Ok(VariableData {
            name,
            kind: var_kind,
            modifier,
            type_,
            is_reference,
        })
    }

    pub fn is_variable(&self) -> bool {
        matches!(self.type_, Type::Variable)
    }

    pub fn is_none(&self) -> bool {
        matches!(self.modifier, VariableModifier::None)
    }

    pub fn is_list(&self) -> bool {
        matches!(self.modifier, VariableModifier::List)
    }

    pub fn is_optional(&self) -> bool {
        matches!(self.modifier, VariableModifier::Optional)
    }
}
