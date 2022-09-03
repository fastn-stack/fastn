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

pub fn parse_import(
    c: &Option<ftd::p11::Header>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p11::Result<(String, String)> {
    let v = match c {
        Some(ftd::p11::Header::KV(ftd::p11::header::KV { value: Some(v), .. })) => v,
        _ => {
            return ftd::interpreter::utils::e2(
                "Unknown caption passed import statement",
                doc_id,
                line_number,
            )
        }
    };

    if v.contains(" as ") {
        let mut parts = v.splitn(2, " as ");
        return match (parts.next(), parts.next()) {
            (Some(n), Some(a)) => Ok((n.to_string(), a.to_string())),
            _ => ftd::interpreter::utils::e2(
                "invalid use of keyword as in import statement",
                doc_id,
                line_number,
            ),
        };
    }

    if v.contains('/') {
        let mut parts = v.rsplitn(2, '/');
        return match (parts.next(), parts.next()) {
            (Some(t), Some(_)) => Ok((v.to_string(), t.to_string())),
            _ => ftd::interpreter::utils::e2("doc id must contain /", doc_id, line_number),
        };
    }

    if let Some((t, _)) = v.split_once('.') {
        return Ok((v.to_string(), t.to_string()));
    }

    Ok((v.to_string(), v.to_string()))
}

pub fn reorder(
    p1: &[ftd::p11::Section],
    doc: &ftd::interpreter::TDoc,
) -> ftd::p11::Result<(Vec<ftd::p11::Section>, Vec<String>)> {
    let mut p1_map: ftd::Map<ftd::p11::Section> = Default::default();
    let mut inserted_p1 = vec![];
    let mut new_p1 = vec![];
    let mut list_or_var = vec![];
    let mut var_types = vec![];
    for (idx, p1) in p1.iter().enumerate() {
        let var_data = ftd::interpreter::variable::VariableData::get_name_kind(
            &p1.name,
            &p1.kind,
            doc,
            p1.line_number,
            &var_types,
        );
        if p1.name == "import" || is_record(&p1.kind) || is_or_type(&p1.kind) || is_map(&p1.kind) {
            inserted_p1.push(idx);
            new_p1.push(p1.to_owned());
        }
        if let Ok(ftd::interpreter::variable::VariableData {
            type_: ftd::interpreter::variable::Type::Variable,
            ref name,
            ..
        }) = var_data
        {
            inserted_p1.push(idx);
            new_p1.push(p1.to_owned());
            list_or_var.push(name.to_string());
        }

        if is_record(&p1.kind) {
            var_types.push(p1.name.to_string());
        }

        if is_or_type(&p1.kind) {
            var_types.push(p1.name.to_string());
            for s in &p1.sub_sections {
                var_types.push(format!("{}.{}", p1.name, s.name));
            }
        }

        if list_or_var.contains(&p1.name) {
            inserted_p1.push(idx);
            new_p1.push(p1.to_owned());
        }

        if let Ok(ftd::interpreter::variable::VariableData {
            type_: ftd::interpreter::variable::Type::Component,
            ref name,
            ..
        }) = var_data
        {
            if p1_map.contains_key(name) {
                return ftd::interpreter::utils::e2(
                    format!("{} is already declared", name),
                    doc.name,
                    p1.line_number,
                );
            }
            p1_map.insert(name.to_string(), p1.to_owned());
            inserted_p1.push(idx);
        }
    }
    let mut new_p1_component = vec![];
    reorder_component(
        &p1_map,
        &mut new_p1_component,
        None,
        &mut vec![],
        doc,
        &var_types,
    )?;
    new_p1.extend(new_p1_component);

    for (idx, p1) in p1.iter().enumerate() {
        if inserted_p1.contains(&idx) {
            continue;
        }
        new_p1.push(p1.to_owned());
    }

    return Ok((new_p1, var_types));

    fn is_kernel_component(comp: String) -> bool {
        if ["ftd.row", "ftd.column"].contains(&comp.as_str()) {
            return true;
        }
        false
    }

    fn reorder_component(
        p1_map: &ftd::Map<ftd::p11::Section>,
        new_p1: &mut Vec<ftd::p11::Section>,
        dependent_p1: Option<String>,
        inserted: &mut Vec<String>,
        doc: &ftd::interpreter::TDoc,
        var_types: &[String],
    ) -> ftd::p11::Result<()> {
        if let Some(p1) = dependent_p1 {
            if inserted.contains(&p1) {
                return Ok(());
            }
            if let Some(v) = p1_map.get(&p1) {
                for sub_section in v.sub_sections.iter() {
                    if inserted.contains(&sub_section.name) || p1 == sub_section.name {
                        continue;
                    }
                    reorder_component(
                        p1_map,
                        new_p1,
                        Some(sub_section.name.to_string()),
                        inserted,
                        doc,
                        var_types,
                    )?;
                }
                let var_data = ftd::interpreter::variable::VariableData::get_name_kind(
                    &v.name,
                    &v.kind,
                    doc,
                    v.line_number,
                    var_types,
                )?;
                if !is_kernel_component(var_data.kind.to_string())
                    && !inserted.contains(&var_data.kind)
                {
                    reorder_component(
                        p1_map,
                        new_p1,
                        Some(var_data.kind),
                        inserted,
                        doc,
                        var_types,
                    )?;
                }
                new_p1.push(v.to_owned());
                inserted.push(p1.to_string());
            }
            return Ok(());
        }

        for (k, v) in p1_map {
            if inserted.contains(k) {
                continue;
            }
            for sub_section in v.sub_sections.iter() {
                for header in sub_section.headers.0.iter() {
                    let name = if header.is_section() {
                        header.get_key()
                    } else {
                        continue;
                    };
                    if inserted.contains(&name) || k == &name {
                        continue;
                    }
                    reorder_component(
                        p1_map,
                        new_p1,
                        Some(name.to_string()),
                        inserted,
                        doc,
                        var_types,
                    )?;
                }
                if inserted.contains(&sub_section.name) || k == &sub_section.name {
                    continue;
                }
                reorder_component(
                    p1_map,
                    new_p1,
                    Some(sub_section.name.to_string()),
                    inserted,
                    doc,
                    var_types,
                )?;
            }
            for header in v.headers.0.iter() {
                let name = if header.is_section() {
                    header.get_key()
                } else {
                    continue;
                };
                if inserted.contains(&name) || k == &name {
                    continue;
                }
                reorder_component(
                    p1_map,
                    new_p1,
                    Some(name.to_string()),
                    inserted,
                    doc,
                    var_types,
                )?;
            }
            let var_data =
                ftd::variable::VariableData::get_name_kind(&v.name, doc, v.line_number, var_types)?;
            if !is_kernel_component(var_data.kind.to_string()) && !inserted.contains(&var_data.kind)
            {
                reorder_component(
                    p1_map,
                    new_p1,
                    Some(var_data.kind),
                    inserted,
                    doc,
                    var_types,
                )?;
            }

            new_p1.push(v.to_owned());
            inserted.push(k.to_string());
        }
        Ok(())
    }
}

pub(crate) fn is_record(kind: &Option<String>) -> bool {
    if let Some(kind) = kind {
        kind.eq("record")
    } else {
        false
    }
}

pub(crate) fn is_or_type(kind: &Option<String>) -> bool {
    if let Some(kind) = kind {
        kind.eq("or-type")
    } else {
        false
    }
}

pub(crate) fn is_map(kind: &Option<String>) -> bool {
    if let Some(kind) = kind {
        kind.eq("map")
    } else {
        false
    }
}

pub(crate) fn get_doc_name_and_remaining(s: &str) -> ftd::p1::Result<(String, Option<String>)> {
    let mut part1 = "".to_string();
    let mut pattern_to_split_at = s.to_string();
    if let Some((p1, p2)) = s.split_once('#') {
        part1 = format!("{}#", p1);
        pattern_to_split_at = p2.to_string();
    }
    Ok(if pattern_to_split_at.contains('.') {
        let (p1, p2) = ftd::p2::utils::split(pattern_to_split_at, ".")?;
        (format!("{}{}", part1, p1), Some(p2))
    } else {
        (s.to_string(), None)
    })
}

pub fn resolve_name(
    line_number: usize,
    name: &str,
    doc_name: &str,
    aliases: &ftd::Map<String>,
) -> ftd::p11::Result<String> {
    if name.contains('#') {
        return Ok(name.to_string());
    }
    Ok(
        match ftd::interpreter::utils::split_module(name, doc_name, line_number)? {
            (Some(m), v, None) => match aliases.get(m) {
                Some(m) => format!("{}#{}", m, v),
                None => format!("{}#{}.{}", doc_name, m, v),
            },
            (Some(m), v, Some(c)) => match aliases.get(m) {
                Some(m) => format!("{}#{}.{}", m, v, c),
                None => format!("{}#{}.{}.{}", doc_name, m, v, c),
            },
            (None, v, None) => format!("{}#{}", doc_name, v),
            _ => unimplemented!(),
        },
    )
}

pub fn structure_header_to_properties(
    s: &str,
    arguments: &ftd::Map<crate::p2::Kind>,
    doc: &ftd::p2::TDoc,
    line_number: usize,
    p1: &ftd::p1::Header,
) -> ftd::p1::Result<ftd::Map<ftd::component::Property>> {
    let (name, caption) = ftd::p2::utils::split(s.to_string(), ":")?;
    match doc.get_thing(line_number, &name) {
        Ok(ftd::p2::Thing::Component(c)) => ftd::component::read_properties(
            line_number,
            p1,
            &if caption.is_empty() {
                None
            } else {
                Some(caption)
            },
            &None,
            "",
            "",
            &c.arguments,
            arguments,
            doc,
            &Default::default(),
            false,
        ),
        t => ftd::p2::utils::e2(
            format!("expected component, found: {:?}", t),
            doc.name,
            line_number,
        ),
    }
}
