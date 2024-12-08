#[derive(Debug, Clone, PartialEq)]
pub enum UR<U: std::fmt::Debug, R: std::fmt::Debug, E: std::fmt::Debug> {
    /// we are using Option<R> here because we want to convert from UnResolved to Resolved without
    /// cloning.
    /// most data going to be on the Resolved side is already there in the UnResolved, the Option
    /// allows us to use mem::replace. See
    Resolved(Option<R>),
    UnResolved(U),
    NotFound,
    /// if the resolution failed, we need not try to resolve it again, unless dependencies change.
    ///
    /// say when we are processing x.ftd we found out that the symbol foo is invalid, so when we are
    /// processing y.ftd, and we find foo, we can directly say that it is invalid.
    ///
    /// this is the goal, but we do not know why isn't `foo` valid, meaning on what another symbol
    /// does it depend on, so when do we "revalidate" the symbol?
    ///
    /// what if we store the dependencies it failed on, so when any of them changes, we can
    /// revalidate?
    Invalid(E),
    InvalidN(Vec<E>),
}

pub trait FromWith<X, W> {
    fn from(x: X, w: W) -> Self;
}

impl<U: std::fmt::Debug, R: std::fmt::Debug, E: std::fmt::Debug> From<U>
    for fastn_continuation::UR<U, R, E>
{
    fn from(u: U) -> fastn_continuation::UR<U, R, E> {
        fastn_continuation::UR::UnResolved(u)
    }
}

impl<U: std::fmt::Debug, R: std::fmt::Debug, E: std::fmt::Debug> fastn_continuation::UR<U, R, E> {
    pub fn unresolved(&self) -> Option<&U> {
        match self {
            fastn_continuation::UR::UnResolved(u) => Some(u),
            _ => None,
        }
    }

    pub fn resolved(&self) -> Option<&R> {
        match self {
            fastn_continuation::UR::Resolved(Some(v)) => Some(v),
            fastn_continuation::UR::Resolved(None) => unreachable!(),
            _ => None,
        }
    }

    pub fn into_resolved(self) -> R {
        match self {
            fastn_continuation::UR::Resolved(Some(r)) => r,
            _ => panic!("{self:?}"),
        }
    }

    pub fn resolve_it<W>(&mut self, w: W)
    where
        R: FromWith<U, W> + std::fmt::Debug,
    {
        match self {
            fastn_continuation::UR::UnResolved(_) => {}
            _ => panic!("cannot resolve it"),
        }

        let u = match std::mem::replace(self, fastn_continuation::UR::Resolved(None)) {
            fastn_continuation::UR::UnResolved(u) => u,
            _ => unreachable!(),
        };
        *self = fastn_continuation::UR::Resolved(Some(FromWith::from(u, w)));
    }
}
