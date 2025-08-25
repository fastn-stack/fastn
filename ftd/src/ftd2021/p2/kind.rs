#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Kind {
    String {
        caption: bool,
        body: bool,
        default: Option<String>,
        is_reference: bool,
    },
    Object {
        default: Option<String>,
        is_reference: bool,
    },
    Integer {
        default: Option<String>,
        is_reference: bool,
    },
    Decimal {
        default: Option<String>,
        is_reference: bool,
    },
    Boolean {
        default: Option<String>,
        is_reference: bool,
    },
    Element,
    Elements,
    Message,
    StringMessage, // message that takes a string
    IntMessage,    // message that takes an int
    Record {
        name: String,
        default: Option<String>,
        is_reference: bool,
    }, // the full name of the record (full document name.record name)
    OrType {
        name: String,
        is_reference: bool,
    }, // the full name of the or-type
    OrTypeWithVariant {
        name: String,
        variant: String,
        is_reference: bool,
    },
    Map {
        kind: Box<Kind>,
        is_reference: bool,
    }, // map of String to Kind
    List {
        kind: Box<Kind>,
        default: Option<String>,
        is_reference: bool,
    },
    Optional {
        kind: Box<Kind>,
        is_reference: bool,
    },
    UI {
        default: Option<(String, ftd::ftd2021::p1::Header)>,
    },
}

impl Kind {
    pub fn is_reference(&self) -> bool {
        match self {
            Kind::String { is_reference, .. }
            | Kind::Object { is_reference, .. }
            | Kind::Integer { is_reference, .. }
            | Kind::Decimal { is_reference, .. }
            | Kind::Boolean { is_reference, .. }
            | Kind::Record { is_reference, .. }
            | Kind::OrType { is_reference, .. }
            | Kind::OrTypeWithVariant { is_reference, .. }
            | Kind::Map { is_reference, .. }
            | Kind::List { is_reference, .. }
            | Kind::Optional { is_reference, .. } => *is_reference,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Kind::String { .. })
    }

    pub fn is_decimal(&self) -> bool {
        matches!(self, Kind::Decimal { .. })
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Kind::Integer { .. })
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Kind::Boolean { .. })
    }

    pub fn is_optional(&self) -> bool {
        matches!(self, Kind::Optional { .. })
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Kind::List { .. })
    }

    pub fn is_string_list(&self) -> bool {
        let list_kind = self.strict_list_kind();
        if let Some(inner_kind) = list_kind {
            return matches!(inner_kind, Kind::String { .. });
        }
        false
    }

    pub fn is_record(&self) -> bool {
        matches!(self, Kind::Record { .. })
    }

    pub fn to_string(&self, line_number: usize, doc_id: &str) -> ftd::ftd2021::p1::Result<String> {
        Ok(match self.inner() {
            ftd::ftd2021::p2::Kind::String { .. } => "string",
            ftd::ftd2021::p2::Kind::Integer { .. } => "integer",
            ftd::ftd2021::p2::Kind::Decimal { .. } => "decimal",
            ftd::ftd2021::p2::Kind::Boolean { .. } => "boolean",
            ftd::ftd2021::p2::Kind::Object { .. } => "object",
            ftd::ftd2021::p2::Kind::List { .. } => "list",
            _ => return ftd::ftd2021::p2::utils::e2(format!("1 Kind supported for default value are string, integer, decimal and boolean with default value, found: kind `{:?}`", &self), doc_id, line_number),
        }.to_string())
    }

    pub fn to_value(
        &self,
        line_number: usize,
        doc_id: &str,
    ) -> ftd::ftd2021::p1::Result<ftd::Value> {
        Ok(match self {
            ftd::ftd2021::p2::Kind::String {
                default: Some(d), ..
            } => ftd::Value::String {
                text: d.to_string(),
                source: ftd::TextSource::Default,
            },
            ftd::ftd2021::p2::Kind::Integer {
                default: Some(d), ..
            } => ftd::Value::Integer {
                value: match d.parse::<i64>() {
                    Ok(v) => v,
                    Err(_) => {
                        return ftd::ftd2021::p2::utils::e2(
                            format!("{d} is not an integer"),
                            doc_id,
                            line_number,
                        );
                    }
                },
            },
            ftd::ftd2021::p2::Kind::Decimal {
                default: Some(d), ..
            } => ftd::Value::Decimal {
                value: d
                    .parse::<f64>()
                    .map_err(|e| ftd::ftd2021::p1::Error::ParseError {
                        message: e.to_string(),
                        doc_id: doc_id.to_string(),
                        line_number,
                    })?,
            },
            ftd::ftd2021::p2::Kind::Boolean {
                default: Some(d), ..
            } => ftd::Value::Boolean {
                value: d
                    .parse::<bool>()
                    .map_err(|e| ftd::ftd2021::p1::Error::ParseError {
                        message: e.to_string(),
                        doc_id: doc_id.to_string(),
                        line_number,
                    })?,
            },
            ftd::ftd2021::p2::Kind::Optional { kind, .. } => {
                if let Ok(f) = kind.to_value(line_number, doc_id) {
                    ftd::Value::Optional {
                        data: Box::new(Some(f)),
                        kind: kind.as_ref().to_owned(),
                    }
                } else {
                    ftd::Value::Optional {
                        data: Box::new(None),
                        kind: kind.as_ref().to_owned(),
                    }
                }
            }
            ftd::ftd2021::p2::Kind::List { kind, .. } => ftd::Value::List {
                data: vec![],
                kind: kind.as_ref().to_owned(),
            },
            _ => {
                return ftd::ftd2021::p2::utils::e2(
                    format!(
                        "2 Kind supported for default value are string, integer, decimal and boolean with default value, found: kind `{:?}`",
                        &self
                    ),
                    doc_id,
                    line_number,
                );
            }
        })
    }

    pub fn has_default_value(&self) -> bool {
        match self {
            Kind::String { default, .. }
            | Kind::Integer { default, .. }
            | Kind::Decimal { default, .. }
            | Kind::Boolean { default, .. }
            | Kind::Record { default, .. }
            | Kind::List { default, .. } => default.is_some(),
            Kind::UI { default } => default.is_some(),
            _ => false,
        }
    }
}

impl Kind {
    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String { .. }, Self::String { .. }) => matches!(other, Self::String { .. }),
            (Self::UI { .. }, Self::UI { .. }) => matches!(other, Self::UI { .. }),
            (Self::Optional { kind, .. }, _) => kind.is_same_as(other),
            (_, Self::Optional { kind: other, .. }) => self.is_same_as(other),
            _ => self.without_default() == other.without_default(),
        }
    }

    pub fn without_default(&self) -> Self {
        match self {
            Kind::Integer { .. } => Kind::Integer {
                default: None,
                is_reference: false,
            },
            Kind::Boolean { .. } => Kind::Boolean {
                default: None,
                is_reference: false,
            },
            Kind::Decimal { .. } => Kind::Decimal {
                default: None,
                is_reference: false,
            },
            Kind::String { caption, body, .. } => Kind::String {
                caption: *caption,
                body: *body,
                default: None,
                is_reference: false,
            },
            Kind::Record { name, .. } => Kind::Record {
                name: name.clone(),
                default: None,
                is_reference: false,
            },
            Kind::List { kind, .. } => Kind::List {
                kind: kind.clone(),
                default: None,
                is_reference: false,
            },
            _ => self.clone(),
        }
    }

    pub fn record(name: &str) -> Self {
        Kind::Record {
            name: name.to_string(),
            default: None,
            is_reference: false,
        }
    }

    pub fn integer() -> Self {
        Kind::Integer {
            default: None,
            is_reference: false,
        }
    }

    pub fn decimal() -> Self {
        Kind::Decimal {
            default: None,
            is_reference: false,
        }
    }

    pub fn boolean() -> Self {
        Kind::Boolean {
            default: None,
            is_reference: false,
        }
    }

    pub fn object() -> Self {
        Kind::Object {
            default: Default::default(),
            is_reference: false,
        }
    }

    pub fn string() -> Self {
        Kind::String {
            caption: false,
            body: false,
            default: None,
            is_reference: false,
        }
    }
    pub fn get_default_value_str(&self) -> Option<String> {
        match self {
            Kind::Integer { default, .. }
            | Kind::Boolean { default, .. }
            | Kind::Decimal { default, .. }
            | Kind::Record { default, .. }
            | Kind::List { default, .. }
            | Kind::String { default, .. } => default.clone(),
            Kind::UI { default, .. } => default.as_ref().map(|(v, _)| v.clone()),
            Kind::Optional { kind, .. } => kind.get_default_value_str(),
            _ => None,
        }
    }

    pub fn set_default(self, default: Option<String>) -> Self {
        match self {
            Kind::String {
                caption,
                body,
                is_reference,
                ..
            } => Kind::String {
                caption,
                body,
                default,
                is_reference,
            },
            Kind::Record {
                name, is_reference, ..
            } => Kind::Record {
                name,
                default,
                is_reference,
            },
            Kind::UI { .. } => Kind::UI {
                default: default.map(|v| (v, Default::default())),
            },
            Kind::Integer { is_reference, .. } => Kind::Integer {
                default,
                is_reference,
            },
            Kind::Decimal { is_reference, .. } => Kind::Decimal {
                is_reference,
                default,
            },
            Kind::Boolean { is_reference, .. } => Kind::Boolean {
                is_reference,
                default,
            },
            Kind::Optional { is_reference, kind } => Kind::Optional {
                kind: Box::from(kind.set_default(default)),
                is_reference,
            },
            Kind::List {
                is_reference, kind, ..
            } => Kind::List {
                is_reference,
                kind,
                default,
            },
            _ => self,
        }
    }

    pub fn set_reference(self, is_reference: bool) -> Self {
        match self {
            Kind::String {
                caption,
                body,
                default,
                ..
            } => Kind::String {
                caption,
                body,
                default,
                is_reference,
            },
            Kind::Record { name, default, .. } => Kind::Record {
                name,
                default,
                is_reference,
            },
            Kind::Integer { default, .. } => Kind::Integer {
                default,
                is_reference,
            },
            Kind::Decimal { default, .. } => Kind::Decimal {
                is_reference,
                default,
            },
            Kind::Boolean { default, .. } => Kind::Boolean {
                is_reference,
                default,
            },
            Kind::Optional { kind, .. } => Kind::Optional { kind, is_reference },
            Kind::List { default, kind, .. } => Kind::List {
                is_reference,
                kind,
                default,
            },
            _ => self,
        }
    }

    pub fn caption() -> Self {
        Kind::String {
            caption: true,
            body: false,
            default: None,
            is_reference: false,
        }
    }

    pub fn body() -> Self {
        Kind::String {
            caption: false,
            body: true,
            default: None,
            is_reference: false,
        }
    }

    pub fn caption_or_body() -> Self {
        Kind::String {
            caption: true,
            body: true,
            default: None,
            is_reference: false,
        }
    }

    pub fn optional(k: Self) -> Self {
        Kind::Optional {
            kind: Box::new(k),
            is_reference: false,
        }
    }

    pub fn list(k: Self) -> Self {
        Kind::List {
            kind: Box::new(k),
            default: None,
            is_reference: false,
        }
    }

    pub fn map(k: Self) -> Self {
        Kind::Map {
            kind: Box::new(k),
            is_reference: false,
        }
    }

    pub fn into_optional(self) -> Self {
        Kind::Optional {
            kind: Box::new(self),
            is_reference: false,
        }
    }

    pub fn inner(&self) -> &Self {
        match self {
            Kind::Optional { kind, .. } => kind,
            _ => self,
        }
    }

    pub fn mut_inner(&mut self) -> &mut Self {
        match self {
            Kind::Optional { kind, .. } => kind,
            _ => self,
        }
    }

    pub fn strict_list_kind(&self) -> Option<&Self> {
        match self {
            Kind::List { kind, .. } => Some(kind),
            _ => None,
        }
    }

    pub fn list_kind(&self) -> &Self {
        match self {
            Kind::List { kind, .. } => kind,
            _ => self,
        }
    }

    pub fn string_any(&self) -> Self {
        match self {
            Self::String { .. } => Self::String {
                caption: true,
                body: true,
                default: None,
                is_reference: false,
            },
            _ => self.to_owned(),
        }
    }

    pub fn read_section(
        &self,
        line_number: usize,
        p1: &ftd::ftd2021::p1::Header,
        p1_caption: &Option<String>,
        p1_body: &Option<(usize, String)>,
        name: &str,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<ftd::PropertyValue> {
        let (v, source) = match p1.str_optional(doc.name, line_number, name)? {
            Some(v) => (v.to_string(), ftd::TextSource::Header),
            None => {
                let optional = match self {
                    Kind::Optional { kind, .. } => match kind.as_ref() {
                        ftd::ftd2021::p2::Kind::String { .. }
                        | ftd::ftd2021::p2::Kind::Integer { .. }
                        | ftd::ftd2021::p2::Kind::Decimal { .. }
                        | ftd::ftd2021::p2::Kind::Boolean { .. } => true,
                        _ => {
                            return Ok(ftd::PropertyValue::Value {
                                value: ftd::Value::None {
                                    kind: *kind.clone(),
                                },
                            });
                        }
                    },
                    ftd::ftd2021::p2::Kind::String { .. }
                    | ftd::ftd2021::p2::Kind::Integer { .. }
                    | ftd::ftd2021::p2::Kind::Decimal { .. }
                    | ftd::ftd2021::p2::Kind::Boolean { .. } => false,
                    t => {
                        return ftd::ftd2021::p2::utils::e2(
                            format!("`{name}` is {t:?}"),
                            doc.name,
                            line_number,
                        );
                    }
                };

                let (caption, body) = if let Kind::String { caption, body, .. } = self.inner() {
                    (*caption, *body)
                } else {
                    (false, false)
                };

                if caption && p1_caption.is_some() {
                    (
                        p1_caption.as_ref().expect("asd").to_string(),
                        ftd::TextSource::Caption,
                    )
                } else if body && p1_body.is_some() {
                    (
                        p1_body.as_ref().expect("asd").1.to_string(),
                        ftd::TextSource::Body,
                    )
                } else if optional {
                    return Ok(ftd::PropertyValue::Value {
                        value: ftd::Value::None {
                            kind: self.inner().to_owned(),
                        },
                    });
                } else if let Some(default) = self.get_default_value_str() {
                    (default, ftd::TextSource::Default)
                } else {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("`{name}` is required"),
                        doc.name,
                        line_number,
                    );
                }
            }
        };

        if v.starts_with('$') {
            return ftd::PropertyValue::resolve_value(
                line_number,
                &v,
                Some(self.to_owned()),
                doc,
                &Default::default(),
                None,
            );
        }

        match self.inner() {
            Kind::Integer { .. } => Ok(ftd::PropertyValue::Value {
                value: ftd::Value::Integer {
                    value: p1.i64(doc.name, line_number, name).unwrap_or(
                        v.parse::<i64>()
                            .map_err(|e| ftd::ftd2021::p1::Error::ParseError {
                                message: e.to_string(),
                                doc_id: doc.name.to_string(),
                                line_number,
                            })?,
                    ),
                },
            }),
            Kind::Decimal { .. } => Ok(ftd::PropertyValue::Value {
                value: ftd::Value::Decimal {
                    value: p1.f64(doc.name, line_number, name).unwrap_or(
                        v.parse::<f64>()
                            .map_err(|e| ftd::ftd2021::p1::Error::ParseError {
                                message: e.to_string(),
                                doc_id: doc.name.to_string(),
                                line_number,
                            })?,
                    ),
                },
            }),
            Kind::Boolean { .. } => Ok(ftd::PropertyValue::Value {
                value: ftd::Value::Boolean {
                    value: p1.bool(doc.name, line_number, name).unwrap_or(
                        v.parse::<bool>()
                            .map_err(|e| ftd::ftd2021::p1::Error::ParseError {
                                message: e.to_string(),
                                doc_id: doc.name.to_string(),
                                line_number,
                            })?,
                    ),
                },
            }),
            Kind::String { .. } => Ok(ftd::PropertyValue::Value {
                value: ftd::Value::String { text: v, source },
            }),
            v => ftd::ftd2021::p2::utils::e2(
                format!("unknown kind found: {v:?}"),
                doc.name,
                line_number,
            ),
        }
    }

    pub fn from(
        line_number: usize,
        s: &str,
        doc: &ftd::ftd2021::p2::TDoc,
        object_kind: Option<(&str, Self)>,
    ) -> ftd::ftd2021::p1::Result<Self> {
        let (optional, k) = if s.starts_with("optional ") {
            (
                true,
                ftd::ftd2021::p2::utils::get_name("optional", s, doc.name)?,
            )
        } else {
            (false, s)
        };

        if k.starts_with("list ") {
            return Ok(Kind::List {
                kind: Box::new(Self::from(
                    line_number,
                    ftd::ftd2021::p2::utils::get_name("list", k, doc.name)?,
                    doc,
                    object_kind,
                )?),
                default: None,
                is_reference: false,
            });
        }

        if let Some((obj_name, obj_kind)) = object_kind
            && k == obj_name
        {
            return Ok(obj_kind);
        }

        let (key, default) = {
            if k.contains("with default") {
                let mut parts = k.splitn(2, " with default");
                let k = parts.next().unwrap().trim();
                let d = parts.next().unwrap().trim();
                (k, Some(d.to_string()))
            } else {
                (k, None)
            }
        };

        let k = match key {
            "string" => Kind::string(),
            "caption" => Kind::caption(),
            "body" => Kind::body(),
            "body or caption" | "caption or body" => Kind::caption_or_body(),
            "integer" => Kind::integer(),
            "decimal" => Kind::decimal(),
            "object" => Kind::object(),
            "boolean" => Kind::boolean(),
            "element" => Kind::Element,
            "elements" => Kind::Elements,
            "message" => Kind::Message,
            "string-message" => Kind::StringMessage,
            "int-message" => Kind::IntMessage,
            "ftd.ui" => Kind::UI { default: None },
            _ => match doc.get_thing(line_number, k)? {
                ftd::ftd2021::p2::Thing::Record(r) => Kind::Record {
                    name: r.name,
                    default: None,
                    is_reference: false,
                },
                ftd::ftd2021::p2::Thing::OrType(e) => Kind::OrType {
                    name: e.name,
                    is_reference: false,
                },
                t => unimplemented!(
                    "{} is {:?}, line number: {}, doc: {}",
                    k,
                    t,
                    line_number,
                    doc.name.to_string()
                ),
            },
        }
        .set_default(default);

        Ok(if optional { Self::optional(k) } else { k })
    }

    pub fn for_variable(
        line_number: usize,
        s: &str,
        default: Option<String>,
        doc: &ftd::ftd2021::p2::TDoc,
        object_kind: Option<(&str, Self)>,
        arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
    ) -> ftd::ftd2021::p1::Result<Self> {
        let default = {
            // resolve the default value
            let mut default = default;
            if let Some(ref v) = default {
                default = Some(doc.resolve_reference_name(line_number, v, arguments)?);
            }
            default
        };

        let var_data = ftd::ftd2021::variable::VariableData::get_name_kind(
            s,
            doc,
            line_number,
            vec![].as_slice(),
        )?;

        let k = match object_kind {
            Some(object_kind) if var_data.kind.eq(object_kind.0) => object_kind.1,
            _ => match var_data.kind.as_str() {
                "string" => Kind::string(),
                "caption" => Kind::caption(),
                "body" => Kind::body(),
                "body or caption" | "caption or body" => Kind::caption_or_body(),
                "integer" => Kind::integer(),
                "decimal" => Kind::decimal(),
                "object" => Kind::object(),
                "boolean" => Kind::boolean(),
                "element" => Kind::Element,
                "elements" => Kind::Elements,
                "message" => Kind::Message,
                "string-message" => Kind::StringMessage,
                "int-message" => Kind::IntMessage,
                "ftd.ui" => Kind::UI { default: None },
                k => match doc.get_thing(line_number, k) {
                    Ok(ftd::ftd2021::p2::Thing::Record(r)) => Kind::Record {
                        name: r.name,
                        default: None,
                        is_reference: false,
                    },
                    Ok(ftd::ftd2021::p2::Thing::OrType(e)) => Kind::OrType {
                        name: e.name,
                        is_reference: false,
                    },
                    t => match default {
                        None => unimplemented!(
                            "{} is {:?}, line number: {}, doc: {}",
                            var_data.name,
                            t,
                            line_number,
                            doc.name.to_string()
                        ),
                        Some(ref d) => ftd::ftd2021::variable::guess_type(d, false)?.kind(),
                    },
                },
            },
        };

        if var_data.is_list() {
            return Ok(Kind::List {
                kind: Box::new(k),
                default,
                is_reference: var_data.is_reference,
            });
        }

        Ok(if var_data.is_optional() {
            Self::optional(k.set_default(default))
        } else {
            k.set_default(default)
        }
        .set_reference(var_data.is_reference))
    }
}
