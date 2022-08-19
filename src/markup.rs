const MAGIC: &str = "MMMMMMMMMAMMAMSMASMDASMDAMSDMASMDASDMASMDASDMAASD";
lazy_static::lazy_static! {
    pub static ref MD: comrak::ComrakOptions = {
        let mut m = comrak::ComrakOptions::default();
        m.extension.strikethrough = true;
        m.extension.table = true;
        m.extension.autolink = true;
        m.extension.tasklist = true;
        m.extension.superscript = true;
        m.parse.smart = true;
        m
    };
}

fn strip_image(s: &str) -> String {
    s.replace("![", MAGIC)
}

pub fn render(s: &str, auto_links: bool, hard_breaks: bool) -> String {
    let s = strip_image(s);
    let o = if auto_links && !hard_breaks {
        comrak::markdown_to_html(s.as_str(), &ftd::markup::MD)
    } else {
        let mut md = MD.clone();
        md.render.hardbreaks = hard_breaks;
        md.extension.autolink = auto_links;
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

