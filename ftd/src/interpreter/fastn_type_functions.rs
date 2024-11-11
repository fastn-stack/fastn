pub(crate) trait KindExt {
    fn list_type(
        &self,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_type::Kind>;
}

impl KindExt for fastn_type::Kind {
    fn list_type(
        &self,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_type::Kind> {
        match &self {
            fastn_type::Kind::List { kind } => Ok(kind.as_ref().clone()),
            t => ftd::interpreter::utils::e2(
                format!("Expected List, found: `{:?}`", t),
                doc_name,
                line_number,
            ),
        }
    }
}

pub trait KindDataExt {
    fn from_ast_kind(
        var_kind: ftd_ast::VariableKind,
        known_kinds: &ftd::Map<fastn_type::Kind>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::KindData>>;

    fn into_by_ast_modifier(self, modifier: &ftd_ast::VariableModifier) -> Self;
    fn scan_ast_kind(
        var_kind: ftd_ast::VariableKind,
        known_kinds: &ftd::Map<fastn_type::Kind>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<()>;
}

impl KindDataExt for fastn_type::KindData {
    fn scan_ast_kind(
        var_kind: ftd_ast::VariableKind,
        known_kinds: &ftd::Map<fastn_type::Kind>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<()> {
        let ast_kind = var_kind.kind;
        match ast_kind.as_ref() {
            "string" | "object" | "integer" | "decimal" | "boolean" | "void" | "ftd.ui"
            | "children" => Ok(()),
            k if known_kinds.contains_key(k) => Ok(()),
            k => doc.scan_thing(k, line_number),
        }
    }

    fn from_ast_kind(
        var_kind: ftd_ast::VariableKind,
        known_kinds: &ftd::Map<fastn_type::Kind>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::KindData>> {
        let mut ast_kind = ftd_p1::AccessModifier::remove_modifiers(var_kind.kind.as_str());
        // let mut ast_kind = var_kind.kind.clone();
        let (caption, body) = check_for_caption_and_body(&mut ast_kind);
        if ast_kind.is_empty() {
            if !(caption || body) {
                return Err(ftd::interpreter::utils::invalid_kind_error(
                    ast_kind,
                    doc.name,
                    line_number,
                ));
            }

            let mut kind_data = fastn_type::KindData {
                kind: fastn_type::Kind::String,
                caption,
                body,
            };

            if let Some(ref modifier) = var_kind.modifier {
                kind_data = kind_data.into_by_ast_modifier(modifier);
            }

            return Ok(ftd::interpreter::StateWithThing::new_thing(kind_data));
        }
        let kind = match ast_kind.as_ref() {
            "string" => fastn_type::Kind::string(),
            "object" => fastn_type::Kind::object(),
            "integer" => fastn_type::Kind::integer(),
            "decimal" => fastn_type::Kind::decimal(),
            "boolean" => fastn_type::Kind::boolean(),
            "void" => fastn_type::Kind::void(),
            "ftd.ui" => fastn_type::Kind::ui(),
            "module" => fastn_type::Kind::module(),
            "kw-args" => fastn_type::Kind::kwargs(),
            "children" => {
                if let Some(modifier) = var_kind.modifier {
                    return ftd::interpreter::utils::e2(
                        format!("Can't add modifier `{:?}`", modifier),
                        doc.name,
                        line_number,
                    );
                }
                fastn_type::Kind::List {
                    kind: Box::new(fastn_type::Kind::subsection_ui()),
                }
            }
            k if known_kinds.contains_key(k) => known_kinds.get(k).unwrap().to_owned(),
            k => match try_ok_state!(doc.search_thing(k, line_number)?) {
                ftd::interpreter::Thing::Record(r) => fastn_type::Kind::record(r.name.as_str()),
                ftd::interpreter::Thing::Component(_) => fastn_type::Kind::ui(),
                ftd::interpreter::Thing::OrType(o) => fastn_type::Kind::or_type(o.name.as_str()),
                ftd::interpreter::Thing::OrTypeWithVariant { or_type, variant } => {
                    fastn_type::Kind::or_type_with_variant(
                        or_type.as_str(),
                        variant.name().as_str(),
                        variant.name().as_str(),
                    )
                }
                ftd::interpreter::Thing::Variable(v) => v.kind.kind,
                t => {
                    return ftd::interpreter::utils::e2(
                        format!("Can't get find for `{:?}`", t),
                        doc.name,
                        line_number,
                    )
                }
            },
        };

        let mut kind_data = fastn_type::KindData {
            kind,
            caption,
            body,
        };

        if let Some(ref modifier) = var_kind.modifier {
            kind_data = kind_data.into_by_ast_modifier(modifier);
        }

        Ok(ftd::interpreter::StateWithThing::new_thing(kind_data))
    }

    fn into_by_ast_modifier(self, modifier: &ftd_ast::VariableModifier) -> Self {
        match modifier {
            ftd_ast::VariableModifier::Optional => self.optional(),
            ftd_ast::VariableModifier::List => self.list(),
            ftd_ast::VariableModifier::Constant => self.constant(),
        }
    }
}

pub(crate) trait PropertyValueExt {
    fn resolve(
        self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_type::Value>;

    fn resolve_with_inherited(
        self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::interpreter::Result<fastn_type::Value>;
}
impl PropertyValueExt for fastn_type::PropertyValue {
    fn resolve(
        self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize, // Todo: Remove this line number instead use self.line_number()
    ) -> ftd::interpreter::Result<fastn_type::Value> {
        self.resolve_with_inherited(doc, line_number, &Default::default())
    }

    fn resolve_with_inherited(
        self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::interpreter::Result<fastn_type::Value> {
        match self {
            fastn_type::PropertyValue::Value { value, .. } => Ok(value),
            fastn_type::PropertyValue::Reference { name, kind, .. }
            | fastn_type::PropertyValue::Clone { name, kind, .. } => {
                doc.resolve_with_inherited(name.as_str(), &kind, line_number, inherited_variables)
            }
            fastn_type::PropertyValue::FunctionCall(fastn_type::FunctionCall {
                name,
                kind,
                values,
                line_number,
                ..
            }) => {
                let function = doc.get_function(name.as_str(), line_number)?;
                function.resolve(&kind, &values, doc, line_number)?.ok_or(
                    ftd::interpreter::Error::ParseError {
                        message: format!(
                            "Expected return value of type {:?} for function {}",
                            kind, name
                        ),
                        doc_id: doc.name.to_string(),
                        line_number,
                    },
                )
            }
        }
    }
}

pub(crate) trait ValueExt {
    fn string(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<String>;

    fn decimal(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<f64>;
    fn integer(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<i64>;
    fn bool(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<bool>;
    fn optional_integer(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<Option<i64>>;
    fn string_list(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<Vec<String>>;
    fn get_or_type(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<(&String, &String, &fastn_type::PropertyValue)>;
}

impl ValueExt for fastn_type::Value {
    fn string(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<String> {
        match self {
            ftd::interpreter::Value::String { text } => Ok(text.to_string()),
            t => ftd::interpreter::utils::e2(
                format!("Expected String, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn decimal(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<f64> {
        match self {
            ftd::interpreter::Value::Decimal { value } => Ok(*value),
            t => ftd::interpreter::utils::e2(
                format!("Expected Decimal, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn integer(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<i64> {
        match self {
            ftd::interpreter::Value::Integer { value } => Ok(*value),
            t => ftd::interpreter::utils::e2(
                format!("Expected Integer, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn bool(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<bool> {
        match self {
            ftd::interpreter::Value::Boolean { value } => Ok(*value),
            t => ftd::interpreter::utils::e2(
                format!("Expected Boolean, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn optional_integer(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<Option<i64>> {
        match self {
            ftd::interpreter::Value::Optional { data, kind } if kind.is_integer() => {
                if let Some(data) = data.as_ref() {
                    data.optional_integer(doc_id, line_number)
                } else {
                    Ok(None)
                }
            }
            ftd::interpreter::Value::Integer { value } => Ok(Some(*value)),
            t => ftd::interpreter::utils::e2(
                format!("Expected Optional Integer, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn string_list(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<Vec<String>> {
        match self {
            ftd::interpreter::Value::List { data, kind } if kind.is_string() => {
                let mut values = vec![];
                for item in data.iter() {
                    let line_number = item.line_number();
                    values.push(
                        item.to_owned()
                            .resolve(doc, line_number)?
                            .string(doc.name, line_number)?,
                    );
                }
                Ok(values)
            }
            ftd::interpreter::Value::String { text } => Ok(vec![text.to_string()]),
            t => ftd::interpreter::utils::e2(
                format!("Expected String list, found: `{:?}`", t),
                doc.name,
                line_number,
            ),
        }
    }

    fn get_or_type(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<(&String, &String, &fastn_type::PropertyValue)> {
        match self {
            Self::OrType {
                name,
                variant,
                value,
                ..
            } => Ok((name, variant, value)),
            t => ftd::interpreter::utils::e2(
                format!("Expected or-type, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }
}
pub fn check_for_caption_and_body(s: &mut String) -> (bool, bool) {
    use itertools::Itertools;

    let mut caption = false;
    let mut body = false;

    let mut expr = s.split_whitespace().collect_vec();

    if expr.is_empty() {
        return (caption, body);
    }

    if is_caption_or_body(expr.as_slice()) {
        caption = true;
        body = true;
        expr = expr[3..].to_vec();
    } else if is_caption(expr[0]) {
        caption = true;
        expr = expr[1..].to_vec();
    } else if is_body(expr[0]) {
        body = true;
        expr = expr[1..].to_vec();
    }

    *s = expr.join(" ");

    (caption, body)
}

pub(crate) fn is_caption_or_body(expr: &[&str]) -> bool {
    if expr.len() < 3 {
        return false;
    }
    if is_caption(expr[0]) && expr[1].eq("or") && is_body(expr[2]) {
        return true;
    }

    if is_body(expr[0]) && expr[1].eq("or") && is_caption(expr[2]) {
        return true;
    }

    false
}

pub(crate) fn is_caption(s: &str) -> bool {
    s.eq("caption")
}

pub fn is_body(s: &str) -> bool {
    s.eq("body")
}
