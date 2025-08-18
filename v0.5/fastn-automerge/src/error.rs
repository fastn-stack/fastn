impl std::fmt::Display for crate::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::Error::NotFound(path) => write!(f, "Document not found: {path}"),
            crate::Error::Database(e) => write!(f, "Database error: {e}"),
            crate::Error::Automerge(e) => write!(f, "Automerge error: {e}"),
            crate::Error::Autosurgeon(e) => write!(f, "Hydrate error: {e}"),
            crate::Error::ReconcileError(e) => write!(f, "Reconcile error: {e}"),
        }
    }
}

impl std::error::Error for crate::Error {}

impl From<rusqlite::Error> for Box<crate::Error> {
    fn from(err: rusqlite::Error) -> Self {
        Box::new(crate::Error::Database(err))
    }
}

impl From<automerge::AutomergeError> for Box<crate::Error> {
    fn from(err: automerge::AutomergeError) -> Self {
        Box::new(crate::Error::Automerge(err))
    }
}

impl From<autosurgeon::HydrateError> for Box<crate::Error> {
    fn from(err: autosurgeon::HydrateError) -> Self {
        Box::new(crate::Error::Autosurgeon(err))
    }
}

impl From<autosurgeon::ReconcileError> for Box<crate::Error> {
    fn from(err: autosurgeon::ReconcileError) -> Self {
        Box::new(crate::Error::ReconcileError(err))
    }
}
