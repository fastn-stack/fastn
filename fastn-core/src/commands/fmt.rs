pub async fn fmt(
    config: &fastn_core::Config,
    file: Option<&str>,
    no_indentation: bool,
) -> fastn_core::Result<()> {
    use itertools::Itertools;

    let documents = config
        .get_files(&config.package)
        .await?
        .into_iter()
        .filter_map(|v| v.get_ftd_document())
        .collect_vec();

    for ftd_document in documents {
        if let Some(file) = file {
            if !ftd_document.id.eq(file) {
                continue;
            }
        }

        dbg!(&ftd_document.id);

        let parsed_content =
            parsed_to_sections(format!("{}\n\n", ftd_document.content.as_str()).as_str());
        dbg!(&parsed_content);
        let format_sections = dbg!(format_sections(parsed_content, !no_indentation));
        config
            .ds
            .write_content(&ftd_document.get_full_path(), format_sections.into_bytes())
            .await?;
    }

    Ok(())
}

#[derive(Debug)]
struct Section {
    value: String,
    kind: SectionKind,
}

#[derive(Debug)]
enum SectionKind {
    Comment,
    Empty,
    Section {
        name: String,
        end: bool,
        sub_sections: Vec<Section>,
    },
}

impl Section {
    fn new_comment(value: &str) -> Section {
        Section {
            value: value.to_string(),
            kind: SectionKind::Comment,
        }
    }

    fn new_empty(value: &str) -> Section {
        Section {
            value: value.to_string(),
            kind: SectionKind::Empty,
        }
    }

    fn new_section(name: &str, value: &str) -> Section {
        Section {
            value: value.to_string(),
            kind: SectionKind::Section {
                name: name.to_string(),
                end: false,
                sub_sections: vec![],
            },
        }
    }
}

fn format_sections(sections: Vec<Section>, indentation: bool) -> String {
    let mut output = vec![];
    for section in sections {
        output.push(format_section(
            &section,
            if indentation { Some(0) } else { None },
        ))
    }
    output.join("\n")
}

fn format_section(section: &Section, indentation: Option<usize>) -> String {
    match &section.kind {
        SectionKind::Comment => add_indentation(section.value.as_str(), indentation),
        SectionKind::Empty => section.value.to_string(),
        SectionKind::Section {
            name,
            end,
            sub_sections,
        } => format_section_kind(
            name.as_str(),
            *end,
            sub_sections.as_slice(),
            section.value.as_str(),
            indentation,
        ),
    }
}

fn format_section_kind(
    section_name: &str,
    end: bool,
    sub_sections: &[Section],
    value: &str,
    indentation: Option<usize>,
) -> String {
    let mut output = vec![add_indentation(value, indentation)];
    for section in sub_sections {
        output.push(format_section(section, indentation.map(|v| v + 1)));
    }
    if end {
        output.push(add_indentation(
            format!("-- end: {section_name}").as_str(),
            indentation,
        ));
    }
    output.join("\n")
}

fn add_indentation(input: &str, indentation: Option<usize>) -> String {
    let indentation = match indentation {
        Some(indentation) if !indentation.eq(&0) => indentation,
        _ => {
            return input.to_string();
        }
    };
    let mut value = vec![];
    for i in input.split('\n') {
        value.push(format!("{}{}", "\t".repeat(indentation), i));
    }
    value.join("\n")
}

fn parsed_to_sections(input: &str) -> Vec<Section> {
    let mut sections = vec![];
    let mut input = input.to_string();
    while !input.is_empty() {
        if end_section(&mut input, &mut sections) {
            continue;
        } else if let Some(empty_section) = empty_section(&mut input) {
            sections.push(empty_section);
        } else if let Some(comment_section) = comment_section(&mut input) {
            sections.push(comment_section);
        } else if let Some(section) = section(&mut input, &mut sections) {
            sections.push(section);
        } else {
            panic!(
                "`{}`: can't parse",
                input
                    .split_once('\n')
                    .map(|(v, _)| v.to_string())
                    .unwrap_or_else(|| input.to_string())
            );
        }
    }

    sections
}

fn end_section(input: &mut String, sections: &mut Vec<Section>) -> bool {
    let mut remaining = None;
    let first_line = if let Some((first_line, rem)) = input.split_once('\n') {
        remaining = Some(rem.to_string());
        first_line.to_string()
    } else {
        input.to_string()
    };

    let section_name = if let Some(section_name) = end_section_name(first_line.as_str()) {
        section_name
    } else {
        return false;
    };

    dbg!("end**", &section_name);

    *input = remaining.unwrap_or_default();

    let mut sub_sections = vec![];
    while let Some(mut section) = sections.pop() {
        dbg!("pop", &section);
        match &mut section.kind {
            SectionKind::Section {
                name,
                end,
                sub_sections: s,
            } if section_name.eq(name.trim_start_matches('$')) => {
                *end = true;
                *s = sub_sections;
                sections.push(section);
                return true;
            }
            _ => {
                sub_sections.push(section);
            }
        }
    }

    panic!("cannot find section {} to end", section_name)
}

fn end_section_name(input: &str) -> Option<String> {
    use itertools::Itertools;

    let input = input.split_whitespace().join(" ");
    input.strip_prefix("-- end:").map(|v| v.trim().to_string())
}

fn section(input: &mut String, sections: &mut Vec<Section>) -> Option<Section> {
    let section_name = get_section_name(input)?;
    let mut value = vec![];
    let mut first_time_encounter_section = true;
    dbg!("section**", &section_name, &input);
    if !input.trim().is_empty() && !input.contains('\n') {
        value.push(input.to_string());
        *input = String::new();
    }
    while let Some((first_line, remaining)) = input.split_once('\n') {
        let first_line = first_line.trim().to_string();
        dbg!(&first_line);
        if first_line.starts_with("-- ") || first_line.starts_with("/-- ") {
            if first_time_encounter_section {
                first_time_encounter_section = false;
                *input = remaining.to_string();
                value.push(first_line);
                continue;
            }
            *input = format!("{first_line}\n{remaining}");
            break;
        }
        value.push(first_line);
        *input = remaining.to_string();
    }

    if !value.is_empty() {
        let mut value = value.join("\n").to_string();
        if !value.trim().is_empty() {
            if let Some((v, probable_comment_section)) = value.rsplit_once("\n\n") {
                let mut probable_comment_section = probable_comment_section.to_string();
                dbg!(&probable_comment_section);
                if let Some(comment_section) = comment_section(&mut probable_comment_section) {
                    dbg!("after", &probable_comment_section, &comment_section);
                    if probable_comment_section.eq("\n") {
                        sections.push(comment_section);
                        value = format!("{v}\n\n");
                    }
                }
            }
        }
        Some(Section::new_section(section_name.as_str(), value.as_str()))
    } else {
        None
    }
}

fn get_section_name(input: &str) -> Option<String> {
    use itertools::Itertools;

    let first_line = if let Some((first_line, _)) = input.split_once('\n') {
        first_line.trim().to_string()
    } else {
        input.trim().to_string()
    };
    if !first_line.starts_with("-- ") && !first_line.starts_with("/-- ") {
        None
    } else if let Some((section_name_kind, _)) = first_line.split_once(':') {
        Some(dbg!(dbg!(dbg!(section_name_kind
            .split_whitespace()
            .join(" "))
        .rsplit_once(' '))
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| section_name_kind.to_string())))
    } else {
        None
    }
}

fn empty_section(input: &mut String) -> Option<Section> {
    let mut value = vec![];
    while let Some((first_line, remaining)) = input.split_once('\n') {
        let first_line = first_line.trim().to_string();
        if !first_line.is_empty() {
            *input = format!("{first_line}\n{remaining}");
            break;
        }
        value.push(first_line);
        *input = remaining.to_string();
    }

    if !value.is_empty() {
        Some(Section::new_empty(value.join("\n").as_str()))
    } else {
        None
    }
}

fn comment_section(input: &mut String) -> Option<Section> {
    let mut value = vec![];

    if !input.trim().is_empty() && !input.contains('\n') {
        value.push(input.to_string());
        *input = String::new();
    }

    while let Some((first_line, remaining)) = input.split_once('\n') {
        let first_line = first_line.trim().to_string();
        if first_line.starts_with("-- ")
            || first_line.starts_with("/-- ")
            || !first_line.starts_with(";;")
        {
            *input = format!("{first_line}\n{remaining}");
            break;
        }
        value.push(first_line);
        *input = remaining.to_string();
    }
    if !value.is_empty() {
        Some(Section::new_comment(value.join("\n").as_str()))
    } else {
        None
    }
}
