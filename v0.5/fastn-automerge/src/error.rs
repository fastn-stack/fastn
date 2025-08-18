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

impl From<rusqlite::Error> for crate::Error {
    fn from(e: rusqlite::Error) -> Self {
        crate::Error::Database(e)
    }
}

impl From<automerge::AutomergeError> for crate::Error {
    fn from(e: automerge::AutomergeError) -> Self {
        crate::Error::Automerge(e)
    }
}

impl From<autosurgeon::HydrateError> for crate::Error {
    fn from(e: autosurgeon::HydrateError) -> Self {
        crate::Error::Autosurgeon(e)
    }
}

impl From<autosurgeon::ReconcileError> for crate::Error {
    fn from(e: autosurgeon::ReconcileError) -> Self {
        crate::Error::ReconcileError(e)
    }
}
