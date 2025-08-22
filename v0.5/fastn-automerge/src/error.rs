// Simplified error implementations for now - will be refined later

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

impl std::fmt::Display for crate::DocumentPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::DocumentPathError::Empty => write!(f, "Document path cannot be empty"),
            crate::DocumentPathError::TooManyPrefixes { count } => {
                write!(f, "Document path can contain at most one '/-/' prefix, found {count}")
            }
        }
    }
}

impl std::error::Error for crate::DocumentPathError {}

impl std::fmt::Display for crate::DocumentLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::DocumentLoadError::Get(e) => write!(f, "Load error: {e}"),
        }
    }
}

impl std::error::Error for crate::DocumentLoadError {}

impl std::fmt::Display for crate::DocumentCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::DocumentCreateError::Create(e) => write!(f, "Create error: {e}"),
        }
    }
}

impl std::error::Error for crate::DocumentCreateError {}

impl std::fmt::Display for crate::DocumentUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::DocumentUpdateError::Update(e) => write!(f, "Update error: {e}"),
        }
    }
}

impl std::error::Error for crate::DocumentUpdateError {}

// TODO: Add proper error implementations for specific database error types
// For now, they will use eyre::Result for flexibility