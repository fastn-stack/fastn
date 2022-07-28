static SYNTAX_DIR: include_dir::Dir<'_> = include_dir::include_dir!("syntax");

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
    pub static ref MD: comrak::ComrakOptions = {
        comrak::ComrakOptions {
            smart: true,
            ext_strikethrough: true,
            ext_table: true, // TODO: implement custom table
            ext_autolink: true,
            ext_tasklist: true, // TODO: implement custom todo
            ext_superscript: true,
            ..Default::default()
        }
    };
}
const MAGIC: &str = "MMMMMMMMMAMMAMSMASMDASMDAMSDMASMDASDMASMDASDMAASD";
pub const DEFAULT_THEME: &str = "base16-ocean.dark";

fn strip_image(s: &str) -> String {
    s.replace("![", MAGIC)
}

pub fn render(s: &str, auto_links: bool, hard_breaks: bool) -> String {
    let s = strip_image(s);
    let o = if auto_links && !hard_breaks {
        comrak::markdown_to_html(s.as_str(), &ftd::render::MD)
    } else {
        let mut md = MD.clone();
        md.hardbreaks = hard_breaks;
        md.ext_autolink = auto_links;
        comrak::markdown_to_html(s.as_str(), &md)
    };
    o.replace(MAGIC, "![")
}

pub fn markup_inline(string: &str) -> String {
    let s = strip_image(string.trim());
    let o = comrak::markdown_to_html(s.as_str(), &MD);
    let o = o.trim().replace('\n', " ");
    let (space_before, space_after) = spaces(string);
    if o.starts_with("<p>") {
        let l1 = o.chars().count();
        let l2 = "<p></p>".len();
        let l = if l1 > l2 { l1 - l2 } else { l1 };
        let result = o
            .chars()
            .skip("<p>".len())
            .take(l)
            .collect::<String>()
            .replace(MAGIC, "![");
        return format!(
            "{}{}{}",
            repeated_space(space_before),
            result,
            repeated_space(space_after)
        );
    }

    return format!(
        "{}{}{}",
        repeated_space(space_before),
        o.replace(MAGIC, "!["),
        repeated_space(space_after)
    );

    fn repeated_space(n: usize) -> String {
        (0..n).map(|_| " ").collect::<String>()
    }

    fn spaces(s: &str) -> (usize, usize) {
        let mut space_before = 0;
        for (i, c) in s.chars().enumerate() {
            if !c.eq(&' ') {
                space_before = i;
                break;
            }
            space_before = i + 1;
        }
        if space_before.eq(&s.len()) {
            return (space_before, 0);
        }
        let mut space_after = 0;
        for (i, c) in s.chars().rev().enumerate() {
            if !c.eq(&' ') {
                space_after = i;
                break;
            }
            space_after = i + 1;
        }
        (space_before, space_after)
    }
}

pub fn inline(s: &str) -> String {
    // this assumes the input is a single line of text
    let s = strip_image(s.trim());

    if s.contains('\n') {
        eprintln!("render_inline called on an input with newlines: {}", s);
    }
    let o = comrak::markdown_to_html(s.as_str(), &MD);
    let o = o.trim().replace('\n', "");
    let l1 = o.chars().count();
    let l2 = "<p></p>".len();
    let l = if l1 > l2 { l1 - l2 } else { l1 };
    o.chars()
        .skip("<p>".len())
        .take(l)
        .collect::<String>()
        .replace(MAGIC, "![")
}

#[cfg(test)]
mod tests {
    #[test]
    fn inline() {
        assert_eq!(super::inline("hello"), "hello");
        assert_eq!(super::inline("hello *world*"), "hello <em>world</em>");
        assert_eq!(super::inline("hello's world"), "hello’s world");
        assert_eq!(super::inline("hello \"s\" world"), "hello “s” world");
    }
}

pub fn code(code: &str, ext: &str, doc_id: &str) -> String {
    code_with_theme(code, ext, DEFAULT_THEME, doc_id).unwrap()
}

pub fn code_with_theme(
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
        syntect::html::highlighted_html_for_string(code.as_str(), &SS, syntax, theme)
            .replacen('\n', "", 1),
    )
}
