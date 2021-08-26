#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a std::collections::BTreeMap<String, String>,
    pub bag: &'a std::collections::BTreeMap<String, crate::p2::Thing>,
}

impl<'a> TDoc<'a> {
    pub fn format_name(&self, name: &str) -> String {
        format!("{}#{}", self.name, name)
    }

    pub fn resolve_name(&self, name: &str) -> crate::p1::Result<String> {
        if name.contains('#') {
            return Ok(name.to_string());
        }

        Ok(match ftd::split_module(name)? {
            (Some(m), v, None) => match self.aliases.get(m) {
                Some(m) => format!("{}#{}", m, v),
                None => return self.err("alias not found", m, "resolve_name"),
            },
            (_, _, Some(_)) => unimplemented!(),
            (None, v, None) => format!("{}#{}", self.name, v),
        })
    }

    pub fn get_record(&self, name: &str) -> crate::p1::Result<crate::p2::Record> {
        match self.get_thing(name)? {
            crate::p2::Thing::Record(v) => Ok(v),
            v => self.err("not a record", v, "get_record"),
        }
    }

    pub fn get_or_type(&self, name: &str) -> crate::p1::Result<crate::OrType> {
        match self.get_thing(name)? {
            crate::p2::Thing::OrType(v) => Ok(v),
            v => self.err("not an or-type", v, "get_or_type"),
        }
    }

    pub fn get_value(&self, name: &str) -> crate::p1::Result<crate::Value> {
        // TODO: name can be a.b.c, and a and a.b are records with right fields
        match self.get_thing(name)? {
            crate::p2::Thing::Variable(v) => Ok(v.value),
            v => self.err("not a variable", v, "get_value"),
        }
    }

    fn err<T, T2: std::fmt::Debug>(&self, msg: &str, ctx: T2, f: &str) -> crate::p1::Result<T> {
        crate::e2(format!("{}: {} ({:?})", self.name, msg, ctx), f)
    }

    pub fn get_component(&self, name: &str) -> crate::p1::Result<crate::Component> {
        match self.get_thing(name)? {
            crate::p2::Thing::Component(v) => Ok(v),
            v => self.err("not a component", v, "get_component"),
        }
    }

    // name = foo | alias.foo | a/b#foo
    pub fn get_thing(&'a self, name: &'a str) -> crate::p1::Result<crate::p2::Thing> {
        match if name.contains('#') {
            self.bag.get(name).map(ToOwned::to_owned)
        } else {
            match ftd::split_module(name)? {
                (Some(m), v, None) => match self.aliases.get(m) {
                    Some(m) => self
                        .bag
                        .get(format!("{}#{}", m, v).as_str())
                        .map(ToOwned::to_owned),
                    None => match self.get_thing(m)? {
                        crate::p2::Thing::OrType(e) => Some(crate::p2::Thing::OrTypeWithVariant {
                            e,
                            variant: v.to_string(),
                        }),
                        t => {
                            return self.err("not an or-type", t, "get_thing");
                        }
                    },
                },
                (Some(m), e, Some(v)) => match self.aliases.get(m) {
                    Some(m) => match self.bag.get(format!("{}#{}", m, e).as_str()) {
                        Some(crate::p2::Thing::OrType(e)) => {
                            Some(crate::p2::Thing::OrTypeWithVariant {
                                e: e.to_owned(),
                                variant: v.to_string(),
                            })
                        }
                        Some(t) => {
                            return self.err("not an or-type", t, "get_thing2");
                        }
                        None => return self.err("not found", format!("{}#{}", m, e), "get_thing3"),
                    },
                    None => return self.err("not found", name, "get_thing4"),
                },
                (None, v, None) => self
                    .bag
                    .get(format!("{}#{}", self.name, v).as_str())
                    .map(|v| v.to_owned()),
                (None, e, Some(v)) => match self.bag.get(format!("{}#{}", self.name, e).as_str()) {
                    Some(crate::p2::Thing::OrType(e)) => {
                        Some(crate::p2::Thing::OrTypeWithVariant {
                            e: e.to_owned(),
                            variant: v.to_string(),
                        })
                    }
                    Some(t) => {
                        return self.err("expected or-type, found", t, "get_thing5");
                    }
                    None => return self.err("not found", name, "get_thing6"),
                },
                // None => return crate::e2(format!("{} not found", name), "get_thing"),
            }
        } {
            Some(v) => Ok(v),
            None => self.err("not found", name, "get_thing"),
        }
    }
}
