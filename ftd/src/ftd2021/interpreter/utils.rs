pub(crate) fn invalid_kind_error<S>(
    message: S,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::interpreter::Error
where
    S: Into<String>,
{
    ftd::ftd2021::interpreter::Error::InvalidKind {
        message: message.into(),
        doc_id: doc_id.to_string(),
        line_number,
    }
}

pub fn parse_import(
    c: &Option<String>,
    doc_id: &str,
    line_number: usize,
) -> ftd::ftd2021::interpreter::Result<(String, String)> {
    let v = match c {
        Some(v) => v.trim(),
        None => {
            return Err(ftd::ftd2021::interpreter::Error::ParseError {
                message: "caption is missing in import statement".to_string(),
                doc_id: doc_id.to_string(),
                line_number,
            });
        }
    };

    if v.contains(" as ") {
        let mut parts = v.splitn(2, " as ");
        return match (parts.next(), parts.next()) {
            (Some(n), Some(a)) => Ok((n.to_string(), a.to_string())),
            _ => Err(ftd::ftd2021::interpreter::Error::ParseError {
                message: "invalid use of keyword as in import statement".to_string(),
                doc_id: doc_id.to_string(),
                line_number,
            }),
        };
    }

    if v.contains('/') {
        let mut parts = v.rsplitn(2, '/');
        return match (parts.next(), parts.next()) {
            (Some(t), Some(_)) => Ok((v.to_string(), t.to_string())),
            _ => Err(ftd::ftd2021::interpreter::Error::ParseError {
                message: "doc id must contain /".to_string(),
                doc_id: doc_id.to_string(),
                line_number,
            }),
        };
    }

    if let Some((t, _)) = v.split_once('.') {
        return Ok((v.to_string(), t.to_string()));
    }

    Ok((v.to_string(), v.to_string()))
}

pub fn resolve_name(name: &str, doc_name: &str, aliases: &ftd::Map<String>) -> String {
    if name.contains('#') {
        return name.to_string();
    }
    match ftd::ftd2021::interpreter::utils::split_module(name) {
        (Some(m), v, None) => match aliases.get(m) {
            Some(m) => format!("{m}#{v}"),
            None => format!("{doc_name}#{m}.{v}"),
        },
        (Some(m), v, Some(c)) => match aliases.get(m) {
            Some(m) => format!("{m}#{v}.{c}"),
            None => format!("{doc_name}#{m}.{v}.{c}"),
        },
        (None, v, None) => format!("{doc_name}#{v}"),
        _ => unimplemented!(),
    }
}

pub fn split_module(id: &str) -> (Option<&str>, &str, Option<&str>) {
    match id.split_once('.') {
        Some((p1, p2)) => match p2.split_once('.') {
            Some((p21, p22)) => (Some(p1), p21, Some(p22)),
            None => (Some(p1), p2, None),
        },
        None => (None, id, None),
    }
}
