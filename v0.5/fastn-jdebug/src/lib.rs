#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_jdebug;

pub trait JDebug {
    fn debug(&self, source: &str) -> serde_json::Value;
}

impl<T: fastn_jdebug::JDebug> fastn_jdebug::JDebug for Vec<T> {
    fn debug(&self, source: &str) -> serde_json::Value {
        serde_json::Value::Array(self.iter().map(|v| v.debug(source)).collect())
    }
}

impl<T: fastn_jdebug::JDebug> fastn_jdebug::JDebug for Option<T> {
    fn debug(&self, source: &str) -> serde_json::Value {
        self.as_ref()
            .map(|v| v.debug(source))
            .unwrap_or(serde_json::Value::Null)
    }
}

impl<K: AsRef<std::ops::Range<usize>>, V: fastn_jdebug::JDebug> fastn_jdebug::JDebug
    for std::collections::HashMap<K, V>
{
    fn debug(&self, source: &str) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        for (k, v) in self {
            let r = k.as_ref();
            o.insert(source[r.start..r.end].to_string(), v.debug(source));
        }
        serde_json::Value::Object(o)
    }
}
