pub fn document_full_id(
    config: &fastn_core::Config,
    doc: &ftd::interpreter::TDoc,
) -> ftd::ftd2021::p1::Result<String> {
    let full_document_id = config.doc_id().unwrap_or_else(|| {
        doc.name
            .to_string()
            .replace(config.package.name.as_str(), "")
    });

    if full_document_id.trim_matches('/').is_empty() {
        return Ok("/".to_string());
    }

    Ok(format!("/{}/", full_document_id.trim_matches('/')))
}

pub fn log_deprecation_warning(message: &str) {
    use colored::Colorize;

    println!("{}", format!("Warning: {}", message).bright_yellow());
}
