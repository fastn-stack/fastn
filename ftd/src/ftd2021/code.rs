static SYNTAX_DIR: include_dir::Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/syntax");
pub const DEFAULT_THEME: &str = "base16-ocean.dark";

pub static SS: once_cell::sync::Lazy<syntect::parsing::SyntaxSet> =
    once_cell::sync::Lazy::new(|| {
        let mut builder = syntect::parsing::SyntaxSet::load_defaults_newlines().into_builder();
        for f in SYNTAX_DIR.files() {
            builder.add(
                syntect::parsing::syntax_definition::SyntaxDefinition::load_from_str(
                    f.contents_utf8().unwrap(),
                    true,
                    f.path().file_stem().and_then(|x| x.to_str()),
                )
                .unwrap(),
            );
        }
        builder.build()
    });
pub static KNOWN_EXTENSIONS: once_cell::sync::Lazy<std::collections::HashSet<String>> =
    once_cell::sync::Lazy::new(|| {
        SS.syntaxes()
            .iter()
            .flat_map(|v| v.file_extensions.to_vec())
            .collect()
    });
pub static TS: once_cell::sync::Lazy<syntect::highlighting::ThemeSet> =
    once_cell::sync::Lazy::new(syntect::highlighting::ThemeSet::load_defaults);

pub fn code(code: &str, ext: &str, theme: &str, doc_id: &str) -> ftd::ftd2021::p1::Result<String> {
    let syntax = SS
        .find_syntax_by_extension(ext)
        .unwrap_or_else(|| SS.find_syntax_plain_text());
    if !TS.themes.contains_key(theme) {
        return Err(ftd::ftd2021::p1::Error::ParseError {
            message: format!("'{theme}' is not a valid theme"),
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
