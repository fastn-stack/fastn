/// example: `list<string> foo` | `foo bar` | `bar`
pub fn section_init(_scanner: &mut fastn_p1::parser::Scanner) -> Option<fastn_p1::SectionInit> {
    // let dashdash = scanner.take("--");

    todo!()
}

#[cfg(test)]
mod test {
    fastn_p1::tt!(super::section_init);

    // #[test]
    fn section_init() {
        t!("-- foo:", {"name": "string"}, "");
    }
}
