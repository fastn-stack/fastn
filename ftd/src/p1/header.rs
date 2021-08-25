pub use crate::p1::{Error, Result};

#[derive(Debug, PartialEq, Default)]
pub struct Header(pub(crate) Vec<(String, String)>);

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
            if k == name {
                return Ok(v.parse()?);
            }
        }
        Err(Error::NotFound {
            key: name.to_string(),
        })
    }

    pub fn f64(&self, name: &str) -> Result<f64> {
        for (k, v) in self.0.iter() {
            if k == name {
                return Ok(v.parse()?);
            }
        }
        Err(Error::NotFound {
            key: name.to_string(),
        })
    }

    pub fn str_with_default<'a>(&'a self, name: &str, def: &'a str) -> Result<&'a str> {
        match self.str(name) {
            Ok(b) => Ok(b),
            Err(Error::NotFound { key: _ }) => Ok(def),
            e => e,
        }
    }

    pub fn str_optional(&self, name: &str) -> Result<Option<&str>> {
        match self.str(name) {
            Ok(b) => Ok(Some(b)),
            Err(Error::NotFound { key: _ }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn str(&self, name: &str) -> Result<&str> {
        for (k, v) in self.0.iter() {
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
