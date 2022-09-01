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

pub fn get_name<'a, 'b>(prefix: &'a str, s: &'b str, doc_id: &str) -> ftd::p1::Result<&'b str> {
    match s.split_once(' ') {
        Some((p1, p2)) => {
            if p1 != prefix {
                return ftd::p2::utils::e2(format!("must start with {}", prefix), doc_id, 0);
                // TODO
            }
            Ok(p2)
        }
        None => ftd::p2::utils::e2(
            format!("{} does not contain space (prefix={})", s, prefix),
            doc_id,
            0, // TODO
        ),
    }
}
