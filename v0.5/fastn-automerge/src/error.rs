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

// Specific database operation error implementations
impl std::fmt::Display for crate::db::CreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::CreateError::DocumentExists(path) => write!(f, "Document already exists: {path}"),
            crate::db::CreateError::Database(e) => write!(f, "Database error: {e}"),
            crate::db::CreateError::Automerge(e) => write!(f, "Automerge error: {e}"),
            crate::db::CreateError::Reconcile(e) => write!(f, "Reconcile error: {e}"),
        }
    }
}

impl std::error::Error for crate::db::CreateError {}

impl std::fmt::Display for crate::db::GetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::GetError::NotFound(path) => write!(f, "Document not found: {path}"),
            crate::db::GetError::Database(e) => write!(f, "Database error: {e}"),
            crate::db::GetError::Automerge(e) => write!(f, "Automerge error: {e}"),
            crate::db::GetError::Hydrate(e) => write!(f, "Hydrate error: {e}"),
        }
    }
}

impl std::error::Error for crate::db::GetError {}

impl std::fmt::Display for crate::db::UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::UpdateError::NotFound(path) => write!(f, "Document not found: {path}"),
            crate::db::UpdateError::Database(e) => write!(f, "Database error: {e}"),
            crate::db::UpdateError::Automerge(e) => write!(f, "Automerge error: {e}"),
            crate::db::UpdateError::Reconcile(e) => write!(f, "Reconcile error: {e}"),
        }
    }
}

impl std::error::Error for crate::db::UpdateError {}

impl std::fmt::Display for crate::db::DeleteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::DeleteError::NotFound(path) => write!(f, "Document not found: {path}"),
            crate::db::DeleteError::Database(e) => write!(f, "Database error: {e}"),
        }
    }
}

impl std::error::Error for crate::db::DeleteError {}

impl std::fmt::Display for crate::db::OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::OpenError::NotFound(path) => write!(f, "Database not found: {}. Run 'init' first.", path.display()),
            crate::db::OpenError::NotInitialized(path) => write!(f, "Database at {} exists but is not initialized. Run 'init' first.", path.display()),
            crate::db::OpenError::MissingActorCounter => write!(f, "Database missing actor counter - not properly initialized"),
            crate::db::OpenError::Database(e) => write!(f, "Database error: {e}"),
            crate::db::OpenError::Automerge(e) => write!(f, "Automerge error: {e}"),
            crate::db::OpenError::Hydrate(e) => write!(f, "Hydrate error: {e}"),
            crate::db::OpenError::InvalidEntity(msg) => write!(f, "Invalid entity: {msg}"),
        }
    }
}

impl std::error::Error for crate::db::OpenError {}

impl std::fmt::Display for crate::db::InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::InitError::DatabaseExists(path) => write!(f, "Database already exists: {}", path.display()),
            crate::db::InitError::Database(e) => write!(f, "Database error: {e}"),
            crate::db::InitError::Migration(e) => write!(f, "Migration error: {e}"),
            crate::db::InitError::Create(e) => write!(f, "Create error: {e}"),
        }
    }
}

impl std::error::Error for crate::db::InitError {}

impl std::fmt::Display for crate::db::ExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::ExistsError::Database(e) => write!(f, "Database error: {e}"),
        }
    }
}

impl std::error::Error for crate::db::ExistsError {}

impl std::fmt::Display for crate::db::ModifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::ModifyError::NotFound(path) => write!(f, "Document not found: {path}"),
            crate::db::ModifyError::Database(e) => write!(f, "Database error: {e}"),
            crate::db::ModifyError::Automerge(e) => write!(f, "Automerge error: {e}"),
            crate::db::ModifyError::Hydrate(e) => write!(f, "Hydrate error: {e}"),
            crate::db::ModifyError::Reconcile(e) => write!(f, "Reconcile error: {e}"),
        }
    }
}

impl std::error::Error for crate::db::ModifyError {}

impl std::fmt::Display for crate::db::SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::SaveError::Exists(e) => write!(f, "Exists check failed: {e}"),
            crate::db::SaveError::Create(e) => write!(f, "Create failed: {e}"),
            crate::db::SaveError::Update(e) => write!(f, "Update failed: {e}"),
        }
    }
}

impl std::error::Error for crate::db::SaveError {}