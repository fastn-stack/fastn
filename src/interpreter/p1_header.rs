impl ftd::p11::Headers {
    #[allow(clippy::type_complexity)]
    pub fn conditional_str(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
        name: &str,
        arguments: &ftd::Map<ftd::interpreter::Kind>,
    ) -> ftd::p11::Result<Vec<(usize, String, Option<String>, bool)>> {
        let mut conditional_vector = vec![];
        for (idx, header) in self.0.iter().enumerate() {
            let k = header.get_key();
            let v = header.get_value(doc.name)?;
            let v = doc.resolve_reference_name(line_number, v, arguments)?;
            let (k, is_referenced) = if let Some(k) = k.strip_prefix('$') {
                (k.to_string(), true)
            } else {
                (k.to_string(), false)
            };
            if k.eq(name) {
                conditional_vector.push((idx, v.to_string(), None, is_referenced));
            }
            if k.contains(" if ") {
                let mut parts = k.splitn(2, " if ");
                let property_name = parts.next().unwrap().trim();
                if property_name == name {
                    let conditional_attribute = parts.next().unwrap().trim();
                    conditional_vector.push((
                        idx,
                        v.to_string(),
                        Some(conditional_attribute.to_string()),
                        is_referenced,
                    ));
                }
            }
        }
        if conditional_vector.is_empty() {
            Err(ftd::p11::Error::NotFound {
                doc_id: doc.name.to_string(),
                line_number,
                key: format!("`{}` header is missing", name),
            })
        } else {
            Ok(conditional_vector)
        }
    }
}
