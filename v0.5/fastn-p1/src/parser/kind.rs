/// example: `list<string>` | `foo<a, b>` | `foo<bar<k>>` | `foo<a, b<asd>, c, d>` |
/// `foo<a, b, c, d, e>`
///
/// // |foo<>|
///
/// note that this function is not responsible for parsing the visibility or doc-comments,
/// it only parses the name and args
pub fn kind(scanner: &mut fastn_p1::parser::Scanner) -> Option<fastn_p1::Kind> {
    let qi = match fastn_p1::parser::qualified_identifier(scanner) {
        Some(qi) => qi,
        None => return None,
    };

    let index = scanner.index();

    // do a look ahead to see if it has <> part
    scanner.skip_spaces();

    if !scanner.take('<') {
        scanner.reset(index);
        return Some(qi.into());
    }
    scanner.skip_spaces();

    todo!()
}

#[cfg(test)]
mod test {
    fastn_p1::tt!(super::kind);

    #[test]
    fn kind() {
        t!("string", "string", "");

        // t!("list<string>", {"name": "list", "args": [{"name": "string"}]}, "");
        // t!("foo<a, b>", {"name": "foo", "args": [{"name": "a"},{"name": "b"}]}, "");
        // t!("foo<bar<k>>", {"name": "foo", "args": [{"name": "bar", "args": [{"name": "k"}]}]}, "");
        // t!(
        //     "foo<a, b<asd>, c, d>",
        //     {
        //         "name": "foo",
        //         "args": [
        //             {"name": "a"},
        //             {"name": "b", "args": [{"name": "asd"}]},
        //             {"name": "c"},
        //             {"name": "d"}
        //         ]
        //     },
        //     ""
        // );
        // t!(
        //     "foo<a, b, c, d, e>",
        //     {
        //         "name": "foo",
        //         "args": [
        //             {"name": "a"},
        //             {"name": "b"},
        //             {"name": "c"},
        //             {"name": "d"},
        //             {"name": "e"}
        //         ]
        //     },
        //     ""
        // );
    }
}
