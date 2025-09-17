/// Hierarchical context for task management and cancellation
pub struct Context {
    /// Context name for debugging
    pub name: String,

    /// When this context was created
    pub created_at: std::time::Instant,

    /// Parent context (None for root)
    parent: Option<std::sync::Arc<Context>>,

    /// Child contexts
    children: std::sync::Arc<std::sync::Mutex<Vec<std::sync::Arc<Context>>>>,

    /// Cancellation token (proper async cancellation)
    cancellation_token: tokio_util::sync::CancellationToken,
}

impl Context {
    /// Create new root context (typically only used by main macro)
    pub fn new(name: &str) -> std::sync::Arc<Context> {
        std::sync::Arc::new(Context {
            name: name.to_string(),
            created_at: std::time::Instant::now(),
            parent: None,
            children: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            cancellation_token: tokio_util::sync::CancellationToken::new(),
        })
    }

    /// Create child context
    pub fn child(&self, name: &str) -> ContextBuilder {
        let child_context = std::sync::Arc::new(Context {
            name: name.to_string(),
            created_at: std::time::Instant::now(),
            parent: Some(std::sync::Arc::new(self.clone())),
            children: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            cancellation_token: self.cancellation_token.child_token(),
        });

        // Add to parent's children list
        if let Ok(mut children) = self.children.lock() {
            children.push(child_context.clone());
        }

        ContextBuilder {
            context: child_context,
        }
    }

    /// Simple spawn (inherits current context, no child creation)
    pub fn spawn<F>(&self, task: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio::spawn(task)
    }

    /// Spawn task with named child context (common case shortcut)
    pub fn spawn_child<F, Fut>(&self, name: &str, task: F) -> tokio::task::JoinHandle<Fut::Output>
    where
        F: FnOnce(std::sync::Arc<Context>) -> Fut + Send + 'static,
        Fut: std::future::Future + Send + 'static,
        Fut::Output: Send + 'static,
    {
        let child_ctx = self.child(name);
        child_ctx.spawn(task)
    }

    /// Wait for cancellation signal (for use in tokio::select!)
    pub async fn wait(&self) {
        // Poll-based future that completes when cancelled
        loop {
            if self.is_cancelled() {
                return;
            }
            // Yield to allow other tasks to run, then check again
            tokio::task::yield_now().await;
        }
    }

    /// Wait for cancellation signal (returns proper Future for tokio::select!)
    pub fn cancelled(&self) -> tokio_util::sync::WaitForCancellationFuture<'_> {
        self.cancellation_token.cancelled()
    }

    /// Check if this context is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    /// Cancel this context and all children recursively
    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }



    /// Get status information for this context and all children
    pub fn status(&self) -> crate::status::ContextStatus {
        let children = if let Ok(children_lock) = self.children.lock() {
            children_lock.iter().map(|child| child.status()).collect()
        } else {
            Vec::new()
        };

        crate::status::ContextStatus {
            name: self.name.clone(),
            is_cancelled: self.is_cancelled(),
            duration: self.created_at.elapsed(),
            children,
        }
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Context {
            name: self.name.clone(),
            created_at: self.created_at,
            parent: self.parent.clone(),
            children: self.children.clone(),
            cancellation_token: self.cancellation_token.clone(),
        }
    }
}

/// Builder for configuring child contexts before spawning
pub struct ContextBuilder {
    pub(crate) context: std::sync::Arc<Context>,
}

impl ContextBuilder {
    /// Spawn task with this child context
    pub fn spawn<F, Fut>(self, task: F) -> tokio::task::JoinHandle<Fut::Output>
    where
        F: FnOnce(std::sync::Arc<Context>) -> Fut + Send + 'static,
        Fut: std::future::Future + Send + 'static,
        Fut::Output: Send + 'static,
    {
        let context = self.context;
        tokio::spawn(async move { task(context).await })
    }
}

/// Global context storage
static GLOBAL_CONTEXT: std::sync::LazyLock<std::sync::Arc<Context>> =
    std::sync::LazyLock::new(|| Context::new("global"));

/// Get the global application context
pub fn global() -> std::sync::Arc<Context> {
    GLOBAL_CONTEXT.clone()
}
