pub fn e2<T, S1>(m: S1, doc_id: &str, line_number: usize) -> ftd::p11::Result<T>
where
    S1: Into<String>,
{
    Err(ftd::p11::Error::ParseError {
        message: m.into(),
        doc_id: doc_id.to_string(),
        line_number,
    })
}

pub fn get_name<'a, 'b>(prefix: &'a str, s: &'b str, doc_id: &str) -> ftd::p11::Result<&'b str> {
    match s.split_once(' ') {
        Some((p1, p2)) => {
            if p1 != prefix {
                return ftd::interpreter::utils::e2(
                    format!("must start with {}", prefix),
                    doc_id,
                    0,
                );
                // TODO
            }
            Ok(p2)
        }
        None => ftd::interpreter::utils::e2(
            format!("{} does not contain space (prefix={})", s, prefix),
            doc_id,
            0, // TODO
        ),
    }
}

pub fn split_module<'a>(
    id: &'a str,
    _doc_id: &str,
    _line_number: usize,
) -> ftd::p11::Result<(Option<&'a str>, &'a str, Option<&'a str>)> {
    match id.split_once('.') {
        Some((p1, p2)) => match p2.split_once('.') {
            Some((p21, p22)) => Ok((Some(p1), p21, Some(p22))),
            None => Ok((Some(p1), p2, None)),
        },
        None => Ok((None, id, None)),
    }
}

pub fn split(name: String, split_at: &str) -> ftd::p11::Result<(String, String)> {
    if !name.contains(split_at) {
        return ftd::interpreter::utils::e2(
            format!("{} is not found in {}", split_at, name),
            "",
            0,
        );
    }
    let mut part = name.splitn(2, split_at);
    let part_1 = part.next().unwrap().trim();
    let part_2 = part.next().unwrap().trim();
    Ok((part_1.to_string(), part_2.to_string()))
}
