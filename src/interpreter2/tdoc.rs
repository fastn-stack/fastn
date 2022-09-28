#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: &'a ftd::Map<ftd::interpreter2::Thing>,
}

impl<'a> TDoc<'a> {
    pub fn resolve_name(&self, name: &str) -> String {
        ftd::interpreter2::utils::resolve_name(name, self.name, self.aliases)
    }

    pub fn get_record(
        &'a self,
        line_number: usize,
        name: &'a str,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Record> {
        match self.get_thing(line_number, name)? {
            ftd::interpreter2::Thing::Record(r) => Ok(r),
            t => self.err(
                format!("Expected Record, found: `{:?}`", t).as_str(),
                name,
                "get_record",
                line_number,
            ),
        }
    }

    pub fn eq(&'a self, name1: &'a str, name2: &'a str) -> bool {
        let name1 = self.resolve_name(name1);
        let name2 = self.resolve_name(name2);
        name1.eq(&name2)
    }

    pub fn get_thing(
        &'a self,
        line_number: usize,
        name: &'a str,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Thing> {
        let name = if let Some(name) = name.strip_prefix('$') {
            name
        } else {
            name
        };

        let (initial_thing, _remaining) = self.get_initial_thing(line_number, name)?;

        Ok(initial_thing)
    }

    pub fn get_initial_thing(
        &'a self,
        line_number: usize,
        name: &'a str,
    ) -> ftd::interpreter2::Result<(ftd::interpreter2::Thing, Option<String>)> {
        if name.contains('#') {
            let (name, remaining_value) = {
                let mut full_name = (name.to_string(), None);
                if let Some((s, n)) = name.split_once('#') {
                    if let Some((v, remaining_value)) = n.split_once('.') {
                        full_name.0 = format!("{}#{}", s, v);
                        full_name.1 = Some(remaining_value.to_string());
                    }
                }
                full_name
            };
            return match self.bag.get(name.as_str()) {
                Some(a) => Ok((a.to_owned(), remaining_value)),
                None => self.err("not found", name, "get_thing", line_number),
            };
        }
        return Ok(match get_initial_thing_(self, None, self.name, name) {
            Some(a) => a,
            None => {
                if let Some((m, v)) = name.split_once('.') {
                    match get_initial_thing_(self, Some(m), m, v) {
                        None => return self.err("not found", name, "get_thing", line_number),
                        Some(a) => a,
                    }
                } else {
                    return self.err("not found", name, "get_thing", line_number);
                }
            }
        });

        fn get_initial_thing_(
            doc: &ftd::interpreter2::TDoc,
            root_name: Option<&str>,
            doc_name: &str,
            name: &str,
        ) -> Option<(ftd::interpreter2::Thing, Option<String>)> {
            let (name, remaining_value) = if let Some((v, remaining_value)) = name.split_once('.') {
                (v, Some(remaining_value.to_string()))
            } else {
                (name, None)
            };

            match doc
                .bag
                .get(format!("{}#{}", doc_name, name).as_str())
                .map(ToOwned::to_owned)
            {
                Some(a) => Some((a, remaining_value)),
                None => match root_name {
                    Some(doc_name) => match doc.aliases.get(doc_name) {
                        Some(g) => doc
                            .bag
                            .get(format!("{}#{}", g, name).as_str())
                            .map(|v| (v.clone(), remaining_value)),
                        None => None,
                    },
                    None => None,
                },
            }
        }
    }

    fn err<T, T2: std::fmt::Debug>(
        &self,
        msg: &str,
        ctx: T2,
        f: &str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<T> {
        ftd::interpreter2::utils::e2(
            format!("{}: {} ({:?}), f: {}", self.name, msg, ctx, f),
            self.name,
            line_number,
        )
    }
}
