pub fn module_name(
    scanner: &mut fastn_lang::Scanner<fastn_lang::token::Document>,
) -> Option<fastn_lang::token::ModuleName> {
    let package = fastn_lang::token::package_name(scanner)?;
    if !scanner.take('/') {
        return Some(fastn_lang::token::ModuleName {
            name: package.alias.clone().into(),
            package,
            path: vec![],
        });
    }

    let mut path = {
        let mut path = Vec::new();
        while let Some(identifier) = fastn_lang::token::identifier(scanner) {
            path.push(identifier);
            if !scanner.take('/') {
                break;
            }
        }
        path
    };

    if path.is_empty() {
        return None;
    }

    Some(fastn_lang::token::ModuleName {
        package,
        name: path.pop().unwrap().into(),
        path,
    })
}

#[cfg(test)]
mod test {
    fastn_lang::tt!(super::module_name);

    #[test]
    fn module_name() {
        t!("foo.com", "foo.com as foo");
        t!("foo.com/", null);
        t!("foo.com/ ", null, " ");
        t!("foo.com/asd", {"package": "foo.com as foo", "name": "asd"});
        t!("foo.com/asd/asda", {"package":"foo.com as foo", "name": "asda", "path": ["asd"]});
        t!(
            "foo.com/asd/asda/erere",
            {
                "package": "foo.com as foo",
                "name": "erere",
                "path": ["asd", "asda"]
            }
        );
    }
}
