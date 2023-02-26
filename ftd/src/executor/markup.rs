const MAGIC: &str = "MMMMMMMMMAMMAMSMASMDASMDAMSDMASMDASDMASMDASDMAASD";

pub static MD: once_cell::sync::Lazy<comrak::ComrakOptions> = once_cell::sync::Lazy::new(|| {
    let mut m = comrak::ComrakOptions::default();
    m.extension.strikethrough = true;
    m.extension.table = true;
    m.extension.autolink = true;
    m.extension.tasklist = true;
    m.extension.superscript = true;
    m.parse.smart = true;
    m
});

pub fn markup(i: &str) -> String {
    comrak::markdown_to_html(i.replace("![", MAGIC).trim(), &MD)
        .trim()
        .replace(MAGIC, "![")
        .replace('\n', " ")
}

pub fn markup_inline(i: &str) -> String {
    let o = markup(i);
    let (space_before, space_after) = spaces(i);

    // if output is wrapped in `<p>`, we are trying to remove it, because this is a single text
    // which may go in button etc.
    // Todo:
    /*if o.starts_with("<p>") {
        let l1 = o.chars().count();
        let l2 = "<p></p>".len();
        let l = if l1 > l2 { l1 - l2 } else { l1 };
        o = o.chars().skip("<p>".len()).take(l).collect::<String>();
    }*/

    format!(
        "{}{o}{}",
        repeated_space(space_before),
        repeated_space(space_after)
    )
}

fn repeated_space(n: usize) -> String {
    (0..n).map(|_| " ").collect::<String>()
}

/// find the count of spaces at beginning and end of the input string
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
