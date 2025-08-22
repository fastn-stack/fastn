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

// Error implementations for new specific error types

impl std::fmt::Display for crate::DocumentPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::DocumentPathError::Empty => write!(f, "Document ID cannot be empty"),
            crate::DocumentPathError::TooManyPrefixes { count } => {
                write!(f, "Document ID can contain at most one '/-/' prefix, found {count}")
            }
        }
    }
}

impl std::error::Error for crate::DocumentPathError {}

impl std::fmt::Display for crate::LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::LoadError::NotFound(path) => write!(f, "Database not found: {}. Run 'init' first.", path.display()),
            crate::LoadError::NotInitialized(path) => write!(f, "Database at {} exists but is not initialized. Run 'init' first.", path.display()),
            crate::LoadError::MissingActorCounter => write!(f, "Database missing actor counter - not properly initialized"),
            crate::LoadError::DatabaseError(e) => write!(f, "Database error: {e}"),
        }
    }
}

impl std::error::Error for crate::LoadError {}

impl std::fmt::Display for crate::ActorIdNotSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Database not initialized - call set_actor_id() first")
    }
}

impl std::error::Error for crate::ActorIdNotSet {}

impl std::fmt::Display for crate::ActorIdAlreadySet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Actor ID already initialized - cannot change")
    }
}

impl std::error::Error for crate::ActorIdAlreadySet {}

impl std::fmt::Display for crate::CreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::CreateError::// ActorNotSet removed - no longer needed
            crate::db::CreateError::DocumentExists(id) => write!(f, "Document already exists: {id}"),
            crate::db::CreateError::Database(e) => write!(f, "Database error: {e}"),
            crate::db::CreateError::Automerge(e) => write!(f, "Automerge error: {e}"),
            crate::db::CreateError::Reconcile(e) => write!(f, "Reconcile error: {e}"),
        }
    }
}

impl std::error::Error for crate::CreateError {}

impl std::fmt::Display for crate::GetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::GetError::// ActorNotSet removed - no longer needed
            crate::GetError::NotFound(id) => write!(f, "Document not found: {id}"),
            crate::GetError::Database(e) => write!(f, "Database error: {e}"),
            crate::GetError::Automerge(e) => write!(f, "Automerge error: {e}"),
            crate::GetError::Hydrate(e) => write!(f, "Hydrate error: {e}"),
        }
    }
}

impl std::error::Error for crate::GetError {}

impl std::fmt::Display for crate::UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::UpdateError::// ActorNotSet removed - no longer needed
            crate::UpdateError::NotFound(id) => write!(f, "Document not found: {id}"),
            crate::UpdateError::Database(e) => write!(f, "Database error: {e}"),
            crate::UpdateError::Automerge(e) => write!(f, "Automerge error: {e}"),
            crate::UpdateError::Reconcile(e) => write!(f, "Reconcile error: {e}"),
        }
    }
}

impl std::error::Error for crate::UpdateError {}

// Missing Error trait implementations
impl std::error::Error for crate::db::LoadError {}
impl std::error::Error for crate::db::InitError {}
impl std::error::Error for crate::db::CreateError {}
impl std::error::Error for crate::db::UpdateError {}
impl std::error::Error for crate::db::DeleteError {}
impl std::error::Error for crate::db::ExistsError {}

// Add missing Display implementations that were removed
impl std::fmt::Display for crate::db::LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::LoadError::NotFound(path) => write!(f, "Database not found: {}. Run 'init' first.", path.display()),
            crate::db::LoadError::NotInitialized(path) => write!(f, "Database at {} exists but is not initialized. Run 'init' first.", path.display()),
            crate::db::LoadError::MissingActorCounter => write!(f, "Database missing actor counter - not properly initialized"),
            crate::db::LoadError::DatabaseError(e) => write!(f, "Database error: {e}"),
        }
    }
}

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

impl std::fmt::Display for crate::db::CreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::CreateError::// ActorNotSet removed - no longer needed
            crate::db::CreateError::DocumentExists(id) => write!(f, "Document already exists: {id}"),
            crate::db::CreateError::Database(e) => write!(f, "Database error: {e}"),
            crate::db::CreateError::Automerge(e) => write!(f, "Automerge error: {e}"),
            crate::db::CreateError::Reconcile(e) => write!(f, "Reconcile error: {e}"),
        }
    }
}

impl std::fmt::Display for crate::db::UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::UpdateError::// ActorNotSet removed - no longer needed
            crate::db::UpdateError::NotFound(id) => write!(f, "Document not found: {id}"),
            crate::db::UpdateError::Database(e) => write!(f, "Database error: {e}"),
            crate::db::UpdateError::Automerge(e) => write!(f, "Automerge error: {e}"),
            crate::db::UpdateError::Reconcile(e) => write!(f, "Reconcile error: {e}"),
        }
    }
}

impl std::fmt::Display for crate::db::ExistsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::ExistsError::// ActorNotSet removed - no longer needed
            crate::db::ExistsError::Database(e) => write!(f, "Database error: {e}"),
        }
    }
}

impl std::fmt::Display for crate::db::DeleteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::db::DeleteError::// ActorNotSet removed - no longer needed
            crate::db::DeleteError::NotFound(id) => write!(f, "Document not found: {id}"),
            crate::db::DeleteError::Database(e) => write!(f, "Database error: {e}"),
        }
    }
}

// From implementations for CreateError
impl From<rusqlite::Error> for crate::db::CreateError {
    fn from(err: rusqlite::Error) -> Self {
        crate::db::CreateError::Database(err)
    }
}

impl From<automerge::AutomergeError> for crate::db::CreateError {
    fn from(err: automerge::AutomergeError) -> Self {
        crate::db::CreateError::Automerge(err)
    }
}

impl From<autosurgeon::ReconcileError> for crate::db::CreateError {
    fn from(err: autosurgeon::ReconcileError) -> Self {
        crate::db::CreateError::Reconcile(err)
    }
}

// From implementations for InitError
impl From<rusqlite::Error> for crate::db::InitError {
    fn from(err: rusqlite::Error) -> Self {
        crate::db::InitError::Database(err)
    }
}

// From implementations for UpdateError
impl From<rusqlite::Error> for crate::db::UpdateError {
    fn from(err: rusqlite::Error) -> Self {
        crate::db::UpdateError::Database(err)
    }
}

impl From<automerge::AutomergeError> for crate::db::UpdateError {
    fn from(err: automerge::AutomergeError) -> Self {
        crate::db::UpdateError::Automerge(err)
    }
}

impl From<autosurgeon::ReconcileError> for crate::db::UpdateError {
    fn from(err: autosurgeon::ReconcileError) -> Self {
        crate::db::UpdateError::Reconcile(err)
    }
}
