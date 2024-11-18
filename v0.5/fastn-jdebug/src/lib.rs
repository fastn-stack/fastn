#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_jdebug;

/// TODO: span has to keep track of the document as well now.
/// TODO: demote usize to u32.
///
/// the document would be document id as stored in sqlite documents table.
///
/// Note: instead of Range, we will use a custom struct, we can use a single 32bit data to store
/// both start, and length. or we keep our life simple, we have can have sections that are really
/// long, eg a long ftd file. lets assume this is the decision for v0.5. we can demote usize to u32
/// as we do not expect individual documents to be larger than few GBs.
#[derive(PartialEq, Hash, Debug, Eq, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub source: string_interner::DefaultSymbol,
}

pub trait JDebug {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value;
}

impl fastn_jdebug::JDebug for fastn_jdebug::Span {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        let t = &interner.resolve(self.source).unwrap()[self.start..self.end];
        if t.is_empty() { "<empty>" } else { t }.into()
    }
}

impl<T: fastn_jdebug::JDebug> fastn_jdebug::JDebug for Vec<T> {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        serde_json::Value::Array(self.iter().map(|v| v.debug(interner)).collect())
    }
}

impl<T: fastn_jdebug::JDebug> fastn_jdebug::JDebug for Option<T> {
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        self.as_ref()
            .map(|v| v.debug(interner))
            .unwrap_or(serde_json::Value::Null)
    }
}

impl<K: AsRef<fastn_jdebug::Span>, V: fastn_jdebug::JDebug> fastn_jdebug::JDebug
    for std::collections::HashMap<K, V>
{
    fn debug(&self, interner: &string_interner::DefaultStringInterner) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        for (k, v) in self {
            let r = k.as_ref();
            o.insert(
                interner.resolve(r.source).unwrap()[r.start..r.end].to_string(),
                v.debug(interner),
            );
        }
        serde_json::Value::Object(o)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

impl fastn_jdebug::Span {
    pub fn wrap<T>(&self, value: T) -> fastn_jdebug::Spanned<T> {
        fastn_jdebug::Spanned {
            span: self.clone(),
            value,
        }
    }

    pub fn str<'input>(
        &self,
        interner: &'input string_interner::DefaultStringInterner,
    ) -> &'input str {
        &interner.resolve(self.source).unwrap()[self.start..self.end]
    }
}
