pub fn code_with_theme(
    code: &str,
    ext: &str,
    theme: &str,
    doc_id: &str,
) -> ftd::p1::Result<ftd::Rendered> {
    Ok(ftd::Rendered {
        original: code.to_string(),
        rendered: ftd::code::code(
            code.replace("\n\\-- ", "\n-- ").as_str(),
            ext,
            theme,
            doc_id,
        )?,
    })
}