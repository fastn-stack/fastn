static SYNTAX_DIR: include_dir::Dir<'_> = include_dir::include_dir!("syntax");
pub const DEFAULT_THEME: &str = "base16-ocean.dark";

lazy_static::lazy_static! {
    pub static ref SS: syntect::parsing::SyntaxSet = {
        let mut builder = syntect::parsing::SyntaxSet::load_defaults_newlines().into_builder();
        for f in SYNTAX_DIR.files() {
            builder.add(syntect::parsing::syntax_definition::SyntaxDefinition::load_from_str(
                f.contents_utf8().unwrap(),
                true,
                f.path().file_stem().and_then(|x| x.to_str())
            ).unwrap());
        }
        builder.build()
    };
    pub static ref KNOWN_EXTENSIONS: std::collections::HashSet<String> =
        SS.syntaxes().iter().flat_map(|v| v.file_extensions.to_vec()).collect();
    pub static ref TS: syntect::highlighting::ThemeSet =
        syntect::highlighting::ThemeSet::load_defaults();
}

pub fn code(
    code: &str,
    ext: &str,
    theme: &str,
    doc_id: &str,
) -> ftd::p1::Result<String> {
    let syntax = SS
        .find_syntax_by_extension(ext)
        .unwrap_or_else(|| SS.find_syntax_plain_text());
    if !TS.themes.contains_key(theme) {
        return Err(ftd::p1::Error::ParseError {
            message: format!("'{}' is not a valid theme", theme),
            doc_id: doc_id.to_string(),
            line_number: 0,
        });
    }

    let theme = &TS.themes[theme];

    let code = code
        .lines()
        .skip_while(|l| l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
        + "\n";

    // TODO: handle various params
    Ok(
        syntect::html::highlighted_html_for_string(code.as_str(), &SS, syntax, theme)?
            .replacen('\n', "", 1),
    )
}
