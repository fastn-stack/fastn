#[derive(Debug, PartialEq, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Header(pub Vec<(usize, String, String)>);


impl Header {
    pub fn without_line_number(&self) -> Self {
        let mut header: Header = Default::default();
        for (_, k, v) in self.0.iter() {
            header.add(&0, k, v);
        }
        header
    }
    pub fn duplicate_header(&self, doc_id: &str, line_number: usize) -> ftd::p1::Result<()> {
        let mut hm = std::collections::HashMap::new();
        for (_, k, _) in self.0.iter().filter(|(_, y, _)| !y.eq("/")) {
            if hm.contains_key(k) {
                return Err(ftd::p1::Error::ParseError {
                    doc_id: doc_id.to_string(),
                    line_number,
                    message: format!("`{}` header is repeated", k),
                });
            }
            hm.insert(k, true);
        }
        Ok(())
    }

    pub fn add(&mut self, line_number: &usize, name: &str, value: &str) {
        self.0
            .push((*line_number, name.to_string(), value.to_string()))
    }

    pub fn bool_with_default(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
        def: bool,
    ) -> ftd::p1::Result<bool> {
        match self.bool(doc_id, line_number, name) {
            Ok(b) => Ok(b),
            Err(ftd::p1::Error::NotFound { .. }) => Ok(def),
            e => e,
        }
    }

    pub fn bool_optional(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
    ) -> ftd::p1::Result<Option<bool>> {
        match self.bool(doc_id, line_number, name) {
            Ok(b) => Ok(Some(b)),
            Err(ftd::p1::Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn bool(&self, doc_id: &str, line_number: usize, name: &str) -> ftd::p1::Result<bool> {
        for (l, k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }
            if k == name {
                return if v == "true" || v == "false" {
                    Ok(v == "true")
                } else {
                    Err(ftd::p1::Error::ParseError {
                        message: "can't parse bool".to_string(),
                        doc_id: doc_id.to_string(),
                        line_number: *l,
                    })
                };
            }
        }
        Err(ftd::p1::Error::NotFound {
            doc_id: doc_id.to_string(),
            line_number,
            key: name.to_string(),
        })
    }

    pub fn i32_with_default(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
        def: i32,
    ) -> ftd::p1::Result<i32> {
        match self.i32(doc_id, line_number, name) {
            Ok(b) => Ok(b),
            Err(ftd::p1::Error::NotFound { .. }) => Ok(def),
            e => e,
        }
    }

    pub fn i32_optional(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
    ) -> ftd::p1::Result<Option<i32>> {
        match self.i32(doc_id, line_number, name) {
            Ok(b) => Ok(Some(b)),
            Err(ftd::p1::Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn i32(&self, doc_id: &str, line_number: usize, name: &str) -> ftd::p1::Result<i32> {
        for (l, k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }
            if k == name {
                return v.parse().map_err(|e: std::num::ParseIntError| {
                    ftd::p1::Error::ParseError {
                        message: format!("{:?}", e),
                        doc_id: doc_id.to_string(),
                        line_number: *l,
                    }
                });
            }
        }
        Err(ftd::p1::Error::NotFound {
            doc_id: doc_id.to_string(),
            line_number,
            key: name.to_string(),
        })
    }

    pub fn i64(&self, doc_id: &str, line_number: usize, name: &str) -> ftd::p1::Result<i64> {
        for (l, k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }

            if k == name {
                return v.parse().map_err(|e: std::num::ParseIntError| {
                    ftd::p1::Error::ParseError {
                        message: format!("{:?}", e),
                        doc_id: doc_id.to_string(),
                        line_number: *l,
                    }
                });
            }
        }
        Err(ftd::p1::Error::NotFound {
            doc_id: doc_id.to_string(),
            line_number,
            key: name.to_string(),
        })
    }

    pub fn i64_optional(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
    ) -> ftd::p1::Result<Option<i64>> {
        match self.i64(doc_id, line_number, name) {
            Ok(b) => Ok(Some(b)),
            Err(ftd::p1::Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn f64(&self, doc_id: &str, line_number: usize, name: &str) -> ftd::p1::Result<f64> {
        for (l, k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }

            if k == name {
                return v.parse().map_err(|e: std::num::ParseFloatError| {
                    ftd::p1::Error::ParseError {
                        message: format!("{:?}", e),
                        doc_id: doc_id.to_string(),
                        line_number: *l,
                    }
                });
            }
        }
        Err(ftd::p1::Error::NotFound {
            doc_id: doc_id.to_string(),
            line_number,
            key: name.to_string(),
        })
    }

    pub fn f64_optional(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
    ) -> ftd::p1::Result<Option<f64>> {
        match self.f64(doc_id, line_number, name) {
            Ok(b) => Ok(Some(b)),
            Err(ftd::p1::Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn str_with_default<'a>(
        &'a self,
        doc_id: &str,
        line_number: usize,
        name: &str,
        def: &'a str,
    ) -> ftd::p1::Result<&'a str> {
        match self.str(doc_id, line_number, name) {
            Ok(b) => Ok(b),
            Err(ftd::p1::Error::NotFound { .. }) => Ok(def),
            e => e,
        }
    }

    pub fn get_events(
        &self,
        line_number: usize,
        doc: &ftd::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    ) -> ftd::p1::Result<Vec<ftd::p2::Event>> {
        let events = {
            let mut events = vec![];
            for (_, k, v) in self.0.iter() {
                if k.starts_with("$on-") && k.ends_with('$') {
                    let mut event = k.replace("$on-", "");
                    event = event[..event.len() - 1].to_string();
                    events.push((event, v.to_string()));
                }
            }
            events
        };
        let mut event = vec![];
        for (e, a) in events {
            event.push(ftd::p2::Event::to_event(
                line_number,
                &e,
                &a,
                doc,
                arguments,
            )?);
        }
        Ok(event)
    }

    pub fn str_optional(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
    ) -> ftd::p1::Result<Option<&str>> {
        match self.str(doc_id, line_number, name) {
            Ok(b) => Ok(Some(b)),
            Err(ftd::p1::Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn conditional_str(
        &self,
        doc: &ftd::p2::TDoc,
        line_number: usize,
        name: &str,
        arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    ) -> ftd::p1::Result<Vec<(usize, String, Option<&str>)>> {
        let mut conditional_vector = vec![];
        for (idx, (_, k, v)) in self.0.iter().enumerate() {
            let v = doc.resolve_reference_name(line_number, v, arguments)?;
            if k == name {
                conditional_vector.push((idx, v.to_string(), None));
            }
            if k.contains(" if ") {
                let mut parts = k.splitn(2, " if ");
                let property_name = parts.next().unwrap().trim();
                if property_name == name {
                    let conditional_attribute = parts.next().unwrap().trim();
                    conditional_vector.push((idx, v.to_string(), Some(conditional_attribute)));
                }
            }
        }
        if conditional_vector.is_empty() {
            Err(ftd::p1::Error::NotFound {
                doc_id: doc.name.to_string(),
                line_number,
                key: format!("`{}` header is missing", name),
            })
        } else {
            Ok(conditional_vector)
        }
    }

    pub fn str(&self, doc_id: &str, line_number: usize, name: &str) -> ftd::p1::Result<&str> {
        for (_, k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }
            if k == name {
                return Ok(v.as_str());
            }
        }

        Err(ftd::p1::Error::NotFound {
            doc_id: doc_id.to_string(),
            line_number,
            key: format!("`{}` header is missing", name),
        })
    }

    pub fn string(&self, doc_id: &str, line_number: usize, name: &str) -> ftd::p1::Result<String> {
        self.str(doc_id, line_number, name).map(ToString::to_string)
    }

    pub fn string_optional(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
    ) -> ftd::p1::Result<Option<String>> {
        Ok(self
            .str_optional(doc_id, line_number, name)?
            .map(ToString::to_string))
    }

    pub fn string_with_default(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
        def: &str,
    ) -> ftd::p1::Result<String> {
        self.str_with_default(doc_id, line_number, name, def)
            .map(ToString::to_string)
    }
}
