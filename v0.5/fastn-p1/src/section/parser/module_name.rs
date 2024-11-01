pub fn module_name(scanner: &mut fastn_p1::section::Scanner) -> Option<fastn_p1::ModuleName> {
    let package = fastn_p1::section::package_name(scanner)?;
    if !scanner.take('/') {
        return Some(fastn_p1::ModuleName {
            name: package.alias.clone().into(),
            package,
            path: vec![],
        });
    }

    let mut path = {
        let mut path = Vec::new();
        while let Some(identifier) = fastn_p1::section::identifier(scanner) {
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

    Some(fastn_p1::ModuleName {
        package,
        name: path.pop().unwrap().into(),
        path,
    })
}

#[cfg(test)]
mod test {
    fastn_p1::tt!(super::module_name);

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
