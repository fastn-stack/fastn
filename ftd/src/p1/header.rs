pub use crate::p1::{Error, Result};

#[derive(Debug, PartialEq, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Header(pub Vec<(String, String)>);

impl Header {
    pub fn add(&mut self, name: &str, value: &str) {
        self.0.push((name.to_string(), value.to_string()))
    }

    pub fn bool_with_default(&self, name: &str, def: bool) -> Result<bool> {
        match self.bool(name) {
            Ok(b) => Ok(b),
            Err(Error::NotFound { key: _ }) => Ok(def),
            e => e,
        }
    }

    pub fn bool_optional(&self, name: &str) -> Result<Option<bool>> {
        match self.bool(name) {
            Ok(b) => Ok(Some(b)),
            Err(Error::NotFound { key: _ }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn bool(&self, name: &str) -> Result<bool> {
        for (k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }
            if k == name {
                return if v == "true" || v == "false" {
                    Ok(v == "true")
                } else {
                    Err(Error::CantParseBool)
                };
            }
        }
        Err(Error::NotFound {
            key: name.to_string(),
        })
    }

    pub fn i32_with_default(&self, name: &str, def: i32) -> Result<i32> {
        match self.i32(name) {
            Ok(b) => Ok(b),
            Err(Error::NotFound { key: _ }) => Ok(def),
            e => e,
        }
    }

    pub fn i32_optional(&self, name: &str) -> Result<Option<i32>> {
        match self.i32(name) {
            Ok(b) => Ok(Some(b)),
            Err(Error::NotFound { key: _ }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn i32(&self, name: &str) -> Result<i32> {
        for (k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }
            if k == name {
                return Ok(v.parse()?);
            }
        }
        Err(Error::NotFound {
            key: name.to_string(),
        })
    }

    pub fn i64(&self, name: &str) -> Result<i64> {
        for (k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }

            if k == name {
                return Ok(v.parse()?);
            }
        }
        Err(Error::NotFound {
            key: name.to_string(),
        })
    }

    pub fn i64_optional(&self, name: &str) -> Result<Option<i64>> {
        match self.i64(name) {
            Ok(b) => Ok(Some(b)),
            Err(Error::NotFound { key: _ }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn f64(&self, name: &str) -> Result<f64> {
        for (k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }

            if k == name {
                return Ok(v.parse()?);
            }
        }
        Err(Error::NotFound {
            key: name.to_string(),
        })
    }

    pub fn f64_optional(&self, name: &str) -> Result<Option<f64>> {
        match self.f64(name) {
            Ok(b) => Ok(Some(b)),
            Err(Error::NotFound { key: _ }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn str_with_default<'a>(&'a self, name: &str, def: &'a str) -> Result<&'a str> {
        match self.str(name) {
            Ok(b) => Ok(b),
            Err(Error::NotFound { key: _ }) => Ok(def),
            e => e,
        }
    }

    pub fn get_events(
        &self,
        doc: &crate::p2::TDoc,
        locals: &std::collections::BTreeMap<String, crate::p2::Kind>,
        arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
    ) -> ftd::p1::Result<Vec<ftd::p2::Event>> {
        let events = {
            let mut events = vec![];
            for (k, v) in self.0.iter() {
                if k.starts_with("$event-") && k.ends_with('$') {
                    let mut event = k.replace("$event-", "");
                    event = event[..event.len() - 1].to_string();
                    events.push((event, v.to_string()));
                }
            }
            events
        };
        let mut event = vec![];
        for (e, a) in events {
            event.push(ftd::p2::Event::to_event(&e, &a, doc, locals, arguments)?);
        }
        Ok(event)
    }

    pub fn str_optional(&self, name: &str) -> Result<Option<&str>> {
        match self.str(name) {
            Ok(b) => Ok(Some(b)),
            Err(Error::NotFound { key: _ }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn conditional_str(&self, name: &str) -> Result<Vec<(String, Option<&str>)>> {
        let mut conditional_vector = vec![];
        for (k, v) in self.0.iter() {
            if k == name {
                conditional_vector.push((v.to_string(), None));
            }
            if k.contains(" if ") {
                let mut parts = k.splitn(2, " if ");
                let property_name = parts.next().unwrap().trim();
                if property_name == name {
                    let conditional_attribute = parts.next().unwrap().trim();
                    conditional_vector.push((v.to_string(), Some(conditional_attribute)));
                }
            }
        }
        if conditional_vector.is_empty() {
            Err(Error::NotFound {
                key: name.to_string(),
            })
        } else {
            Ok(conditional_vector)
        }
    }

    pub fn str(&self, name: &str) -> Result<&str> {
        for (k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }
            if k == name {
                return Ok(v.as_str());
            }
        }
        Err(Error::NotFound {
            key: name.to_string(),
        })
    }

    pub fn string(&self, name: &str) -> Result<String> {
        self.str(name).map(ToString::to_string)
    }

    pub fn string_optional(&self, name: &str) -> Result<Option<String>> {
        Ok(self.str_optional(name)?.map(ToString::to_string))
    }

    pub fn string_with_default(&self, name: &str, def: &str) -> Result<String> {
        self.str_with_default(name, def).map(ToString::to_string)
    }
}
