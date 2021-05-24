lazy_static::lazy_static! {
    pub static ref SS: syntect::parsing::SyntaxSet = {
        let mut builder = syntect::parsing::SyntaxSet::load_defaults_newlines().into_builder();
        let path = std::path::Path::new("syntax");
        if path.exists() {
            builder.add_from_folder(path, true).unwrap();
        }
        builder.build()
    };
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

fn strip_image(s: &str) -> String {
    s.replace("![", MAGIC)
}

pub fn render(s: &str, auto_links: bool, hard_breaks: bool) -> String {
    let s = strip_image(s);
    let o = if auto_links && !hard_breaks {
        comrak::markdown_to_html(s.as_str(), &crate::render::MD)
    } else {
        let mut md = MD.clone();
        md.hardbreaks = hard_breaks;
        md.ext_autolink = auto_links;
        comrak::markdown_to_html(s.as_str(), &md)
    };
    o.replace(MAGIC, "![")
}

pub fn inline(s: &str) -> String {
    // this assumes the input is a single line of text
    let s = strip_image(s.trim());
    if s.contains('\n') {
        eprintln!("render_inline called on an input with newlines: {}", s);
    }
    let o = comrak::markdown_to_html(s.as_str(), &MD);
    let o = o.trim().replace("\n", "");
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

pub fn code(code: &str, ext: &str) -> String {
    let syntax = SS
        .find_syntax_by_extension(ext)
        .unwrap_or_else(|| SS.find_syntax_plain_text());
    let theme = &TS.themes["base16-ocean.dark"];

    let code = code
        .lines()
        .skip_while(|l| l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
        + "\n";

    // TODO: handle various params
    syntect::html::highlighted_html_for_string(code.as_str(), &SS, syntax, theme)
        .replacen("\n", "", 1)
}
