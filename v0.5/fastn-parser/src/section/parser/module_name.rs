pub fn module_name(
    scanner: &mut fastn_parser::section::Scanner,
) -> Option<fastn_parser::ModuleName> {
    let package = fastn_parser::section::package_name(scanner)?;
    if !scanner.take('/') {
        return Some(fastn_parser::ModuleName {
            name: package.alias.clone().into(),
            package,
            path: vec![],
        });
    }

    let mut path = {
        let mut path = Vec::new();
        while let Some(identifier) = fastn_parser::section::identifier(scanner) {
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

    Some(fastn_parser::ModuleName {
        package,
        name: path.pop().unwrap().into(),
        path,
    })
}

#[cfg(test)]
mod test {
    fastn_parser::tt!(super::module_name);

    #[test]
    fn module_name() {
        t!("foo.com", "foo.com as foo");
        t!("foo.com/", null);
        t!("foo.com/ ", null, " ");
        t!("foo.com/asd", {"package": "foo.com as foo", "path": ["asd"]});
        t!("foo.com/asd/asda", {"package":"foo.com", "path": ["asd", "asda"]});
        t!("foo.com/asd/asda/erere", {"package":"foo.com", "path": ["asd", "asda", "erere"]});
    }
}
