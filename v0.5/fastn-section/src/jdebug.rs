/// TODO: span has to keep track of the document as well now.
/// TODO: demote usize to u32.
///
/// the document would be document id as stored in sqlite documents table.
///
/// Note: instead of Range, we will use a custom struct, we can use a single 32bit data to store
/// both start, and length. or we keep our life simple, we have can have sections that are really
/// long, eg a long ftd file. lets assume this is the decision for v0.5. we can demote usize to u32
/// as we do not expect individual documents to be larger than few GBs.
#[derive(PartialEq, Hash, Debug, Eq, Clone, Default)]
pub struct Span {
    inner: arcstr::Substr, // this is currently a 32-byte struct.
}

pub trait JDebug {
    fn debug(&self) -> serde_json::Value;
}

impl fastn_section::JDebug for fastn_section::Span {
    fn debug(&self) -> serde_json::Value {
        if self.inner.is_empty() {
            "<empty>"
        } else {
            self.inner.as_str()
        }
        .into()
    }
}

impl AsRef<arcstr::Substr> for fastn_section::Span {
    fn as_ref(&self) -> &arcstr::Substr {
        &self.inner
    }
}

impl<T: fastn_section::JDebug> fastn_section::JDebug for Vec<T> {
    fn debug(&self) -> serde_json::Value {
        serde_json::Value::Array(self.iter().map(|v| v.debug()).collect())
    }
}

impl<T: fastn_section::JDebug> fastn_section::JDebug for Option<T> {
    fn debug(&self) -> serde_json::Value {
        self.as_ref()
            .map(|v| v.debug())
            .unwrap_or(serde_json::Value::Null)
    }
}

impl<K: AsRef<fastn_section::Span>, V: fastn_section::JDebug> fastn_section::JDebug
    for std::collections::HashMap<K, V>
{
    fn debug(&self) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        for (k, v) in self {
            let r = k.as_ref();
            o.insert(r.inner.to_string(), v.debug());
        }
        serde_json::Value::Object(o)
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

impl fastn_section::Span {
    pub fn inner_str(&self, s: &str) -> fastn_section::Span {
        fastn_section::Span {
            inner: self.inner.substr_from(s),
        }
    }

    pub fn wrap<T>(&self, value: T) -> fastn_section::Spanned<T> {
        fastn_section::Spanned {
            span: self.clone(),
            value,
        }
    }

    pub fn span(&self, start: usize, end: usize) -> fastn_section::Span {
        fastn_section::Span {
            inner: self.inner.substr(start..end),
        }
    }

    pub fn start(&self) -> usize {
        self.inner.range().start
    }

    pub fn end(&self) -> usize {
        self.inner.range().end
    }

    pub fn str(&self) -> &str {
        &self.inner
    }
}

impl From<arcstr::Substr> for Span {
    fn from(inner: arcstr::Substr) -> Self {
        fastn_section::Span { inner }
    }
}

impl<T> fastn_section::Spanned<T> {
    pub fn map<T2, F: FnOnce(T) -> T2>(self, f: F) -> fastn_section::Spanned<T2> {
        fastn_section::Spanned {
            span: self.span,
            value: f(self.value),
        }
    }
}

impl<T: fastn_section::JDebug> fastn_section::JDebug for fastn_section::Spanned<T> {
    fn debug(&self) -> serde_json::Value {
        self.value.debug()
    }
}

impl fastn_section::JDebug for () {
    fn debug(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}
