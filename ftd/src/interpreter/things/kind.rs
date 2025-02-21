pub trait KindExt {
    fn list_type(
        &self,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::Kind>;
}
impl KindExt for fastn_resolved::Kind {
    fn list_type(
        &self,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::Kind> {
        match &self {
            fastn_resolved::Kind::List { kind } => Ok(kind.as_ref().clone()),
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
        known_kinds: &ftd::Map<fastn_resolved::Kind>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::KindData>>;

    fn into_by_ast_modifier(self, modifier: &ftd_ast::VariableModifier) -> Self;
    fn scan_ast_kind(
        var_kind: ftd_ast::VariableKind,
        known_kinds: &ftd::Map<fastn_resolved::Kind>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<()>;
}
impl KindDataExt for fastn_resolved::KindData {
    fn scan_ast_kind(
        var_kind: ftd_ast::VariableKind,
        known_kinds: &ftd::Map<fastn_resolved::Kind>,
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
        known_kinds: &ftd::Map<fastn_resolved::Kind>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::KindData>> {
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

            let mut kind_data = fastn_resolved::KindData {
                kind: fastn_resolved::Kind::String,
                caption,
                body,
            };

            if let Some(ref modifier) = var_kind.modifier {
                kind_data = kind_data.into_by_ast_modifier(modifier);
            }

            return Ok(ftd::interpreter::StateWithThing::new_thing(kind_data));
        }
        let kind = match ast_kind.as_ref() {
            "string" => fastn_resolved::Kind::string(),
            "object" => fastn_resolved::Kind::object(),
            "integer" => fastn_resolved::Kind::integer(),
            "decimal" => fastn_resolved::Kind::decimal(),
            "boolean" => fastn_resolved::Kind::boolean(),
            "void" => fastn_resolved::Kind::void(),
            "ftd.ui" => fastn_resolved::Kind::ui(),
            "module" => fastn_resolved::Kind::module(),
            "kw-args" => fastn_resolved::Kind::kwargs(),
            "template" => fastn_resolved::Kind::template(),
            "children" => {
                if let Some(modifier) = var_kind.modifier {
                    return ftd::interpreter::utils::e2(
                        format!("Can't add modifier `{:?}`", modifier),
                        doc.name,
                        line_number,
                    );
                }
                fastn_resolved::Kind::List {
                    kind: Box::new(fastn_resolved::Kind::subsection_ui()),
                }
            }
            k if known_kinds.contains_key(k) => known_kinds.get(k).unwrap().to_owned(),
            k => match try_ok_state!(doc.search_thing(k, line_number)?) {
                ftd::interpreter::Thing::Record(r) => fastn_resolved::Kind::record(r.name.as_str()),
                ftd::interpreter::Thing::Component(_) => fastn_resolved::Kind::ui(),
                ftd::interpreter::Thing::OrType(o) => {
                    fastn_resolved::Kind::or_type(o.name.as_str())
                }
                ftd::interpreter::Thing::OrTypeWithVariant { or_type, variant } => {
                    fastn_resolved::Kind::or_type_with_variant(
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
                    );
                }
            },
        };

        let mut kind_data = fastn_resolved::KindData {
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
