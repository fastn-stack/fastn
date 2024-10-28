/// module name looks like <package-name>(/<identifier>)*/?)
pub fn module_name(scanner: &mut fastn_p1::parser::Scanner) -> Option<fastn_p1::ModuleName> {
    let package = fastn_p1::parser::package_name(scanner)?;
    if !scanner.take('/') {
        return Some(fastn_p1::ModuleName {
            package,
            path: vec![],
        });
    }

    let path = {
        let mut path = Vec::new();
        while let Some(identifier) = fastn_p1::parser::identifier(scanner) {
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

    Some(fastn_p1::ModuleName { package, path })
}

#[cfg(test)]
mod test {
    fastn_p1::tt!(super::module_name);

    #[test]
    fn module_name() {
        t!("foo.com", {"package":"foo.com"}, "");
        t!("foo.com/", null, "");
        t!("foo.com/ ", null, " ");
        t!("foo.com/asd", {"package":"foo.com", "path": ["asd"]}, "");
    }
}
