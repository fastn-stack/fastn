fn skip_all_whitespace(scanner: &mut fastn_section::Scanner<fastn_section::Document>) {
    // Skip all whitespace including spaces, tabs, and newlines
    // We need to loop because spaces and newlines might be interleaved
    loop {
        let start_index = scanner.index();
        scanner.skip_spaces();
        scanner.skip_new_lines();
        // If we didn't advance, we're done
        if scanner.index() == start_index {
            break;
        }
    }
}

pub fn kind(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::Kind> {
    let qi = fastn_section::parser::identifier_reference(scanner)?;

    // By scoping `index` here, it becomes eligible for garbage collection as soon
    // as it's no longer necessary, reducing memory usage.
    // This block performs a look-ahead to check for an optional `<>` part.
    {
        let index = scanner.index();
        skip_all_whitespace(scanner);

        // Check if there's a `<`, indicating the start of generic arguments.
        if !scanner.take('<') {
            scanner.reset(index);
            // No generics, return as simple `Kind`
            return Some(qi.into());
        }
    }

    skip_all_whitespace(scanner);
    // Parse arguments within the `<...>`
    let mut args = Vec::new();

    // Continue parsing arguments until `>` is reached
    while let Some(arg) = kind(scanner) {
        args.push(arg);

        skip_all_whitespace(scanner);

        // If a `>` is found, end of arguments
        if scanner.take('>') {
            break;
        }

        // If a comma is expected between arguments, consume it and move to the next
        if !scanner.take(',') {
            // If no comma and no `>`, the syntax is invalid
            return None;
        }

        skip_all_whitespace(scanner);
    }

    // Return a `Kind` with the parsed `name` and `args`
    Some(fastn_section::Kind {
        name: qi,
        args: Some(args),
    })
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::kind);

    #[test]
    fn kind() {
        t!("string", "string");
        t!("list<string>", {"name": "list", "args": ["string"]});
        t!("foo<a, b>", {"name": "foo", "args": ["a", "b"]});
        t!(
            "foo<bar   <k >>",
            {"name": "foo", "args": [{"name": "bar", "args": ["k"]}]}
        );
        t!(
            "foo \t <a, b< asd                 >, c, d>",
            {
                "name": "foo",
                "args": [
                    "a",
                    {"name": "b", "args": ["asd"]},
                    "c",
                    "d"
                ]
            }
        );
        t!(
            "foo \t <a, b< asd<e, f<g>>                 >, c, d>",
            {
                "name": "foo",
                "args": [
                    "a",
                    {"name": "b", "args": [
                        {
                            "name": "asd",
                            "args": [
                                "e",
                                {"name": "f", "args": ["g"]}
                        ]
                        }
                    ]},
                    "c",
                    "d"
                ]
            }
        );
        t!(
            "foo<a        , b\t,\tc, d, e>",
            {
                "name": "foo",
                "args": ["a","b","c","d","e"]
            }
        );

        t!(
            "foo < bar<k>> ",
            {"name": "foo", "args": [{"name": "bar", "args": ["k"]}]},
            " "
        );
        t!(
            "foo<bar<k>>  moo",
            {"name": "foo", "args": [{"name": "bar", "args": ["k"]}]},
            "  moo"
        );

        // Numbers in type names (after identifier updates)
        t!("vec3", "vec3");
        t!("list2<string>", {"name": "list2", "args": ["string"]});
        t!("map<key123, value456>", {"name": "map", "args": ["key123", "value456"]});
        t!("matrix3x3", "matrix3x3");
        t!(
            "vec2<float32>",
            {"name": "vec2", "args": ["float32"]}
        );

        // Qualified type names with dots
        t!("std.string", "std.string");
        t!("ftd.text", "ftd.text");
        t!(
            "module.List<item>",
            {"name": "module.List", "args": ["item"]}
        );

        // Can't start with space
        f!(" string");

        // Can't start with number
        f!("123type");

        // Test with newlines in generic parameters - now more readable with indoc!
        t!(
            "
            foo<
              bar
              <
                k>
            >  moo",
            {"name": "foo", "args": [{"name": "bar", "args": ["k"]}]},
            "  moo"
        );

        // Comments aren't handled yet - this test will fail
        // t!(
        //     "foo<\n  ;; some comment\n bar \n ;; more comments \n<\n k>\n>  moo",
        //     {"name": "foo", "args": [{"name": "bar", "args": ["k"]}]},
        //     "  moo"
        // );
    }
}
