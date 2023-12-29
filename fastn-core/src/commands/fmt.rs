pub const INDENTATION: usize = 4;

pub async fn fmt(
    config: &fastn_core::Config,
    file: Option<&str>,
    indentation: bool,
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

        let p1 = ftd::p1::parse(ftd_document.content.as_str(), ftd_document.id.as_str())?;
        let format_sections =
            format_sections(p1.as_slice(), if indentation { Some(0) } else { None });
        tokio::fs::write(ftd_document.get_full_path(), format_sections).await?;
    }

    Ok(())
}

fn format_sections(sections: &[ftd::p1::Section], indentation: Option<usize>) -> String {
    let mut formatted_sections = vec![];
    for section in sections {
        formatted_sections.push(format_section(section, indentation));
    }
    formatted_sections.join("\n\n")
}

fn format_section(section: &ftd::p1::Section, indentation: Option<usize>) -> String {
    let (inline_caption, block_caption) =
        format_caption(&section.caption, section.name.as_str(), indentation);
    format!(
        indoc::indoc! {"
        -- {kind}{name}:{caption}
        "},
        kind = section
            .kind
            .as_ref()
            .map(|v| format!("{v} "))
            .unwrap_or_default(),
        name = section.name,
        caption = inline_caption.unwrap_or_default()
    )
}

fn format_caption(
    caption: &Option<ftd::p1::Header>,
    section_name: &str,
    indentation: Option<usize>,
) -> (Option<String>, Option<String>) {
    let caption = if let Some(caption) = caption {
        caption
    } else {
        return (None, None);
    };

    let mut inline_caption_value = None;
    let mut block_caption_value = None;

    match caption {
        ftd::p1::Header::KV(key_value) => {
            inline_caption_value = format_inline_caption_value(key_value);
            block_caption_value = format_block_caption_value(key_value, section_name, indentation);
        }
        ftd::p1::Header::Section(section) => {
            let sections = format_sections(
                section.section.as_slice(),
                indentation.map(|v| v + INDENTATION),
            );
            block_caption_value = Some(add_indentation(
                format!("-- {}.caption:\n\n{sections}", section_name).as_str(),
                indentation,
            ));
        }
        ftd::p1::Header::BlockRecordHeader(_) => todo!(),
    }
    (inline_caption_value, block_caption_value)
}

fn format_inline_caption_value(key_value: &ftd::p1::KV) -> Option<String> {
    key_value
        .value
        .as_ref()
        .map(|v| {
            if v.contains('\n') {
                None
            } else {
                Some(v.to_string())
            }
        })
        .flatten()
}

fn format_block_caption_value(
    key_value: &ftd::p1::KV,
    section_name: &str,
    indentation: Option<usize>,
) -> Option<String> {
    key_value
        .value
        .as_ref()
        .map(|v| {
            if v.contains('\n') {
                Some(add_indentation(
                    format!("-- {}.caption:\n\n{}", section_name, v).as_str(),
                    indentation,
                ))
            } else {
                None
            }
        })
        .flatten()
}

fn format_inline_kv(key_value: &ftd::p1::KV) -> String {
    use itertools::Itertools;

    let key = vec![
        format_access_modifier(&key_value.access_modifier),
        key_value.kind.clone().unwrap_or_default(),
        key_value.key.clone(),
    ]
    .into_iter()
    .filter(|v| !v.is_empty())
    .join(" ");

    format!(
        "{}:{}",
        key,
        key_value
            .value
            .as_ref()
            .map(|v| format!(" {v}"))
            .unwrap_or_default()
    )
}

fn format_access_modifier(modifier: &ftd::p1::AccessModifier) -> String {
    match modifier {
        ftd::p1::AccessModifier::Public => "".to_string(),
        ftd::p1::AccessModifier::Private => "private".to_string(),
    }
}

fn add_indentation(value: &str, indentation: Option<usize>) -> String {
    if let Some(indentation) = indentation {
        let mut indented_value_list = vec![];
        for line in value.split('\n') {
            if line.trim().is_empty() {
                indented_value_list.push(line.trim().to_string());
            } else {
                indented_value_list.push(format!("{}{}", " ".repeat(indentation), line.trim_end()));
            }
        }
        return indented_value_list.join("\n");
    }
    value.to_string()
}
