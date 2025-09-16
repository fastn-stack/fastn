#![warn(unused_extern_crates)]
#![deny(unused_crate_dependencies)]

extern crate self as fastn_context;

use tokio as _; // used by main macro
use eyre as _; // used by main macro
use tokio_util as _; // used for cancellation tokens

/// Hierarchical context for task management and cancellation
pub struct Context {
    /// Context name for debugging
    pub name: String,
    
    /// Parent context (None for root)
    parent: Option<std::sync::Arc<Context>>,
    
    /// Child contexts
    children: std::sync::Arc<std::sync::Mutex<Vec<std::sync::Arc<Context>>>>,
    
    /// Cancellation token for this context and children
    cancellation: tokio_util::sync::CancellationToken,
}

impl Context {
    /// Create new root context (typically only used by main macro)
    pub fn new(name: &str) -> std::sync::Arc<Context> {
        std::sync::Arc::new(Context {
            name: name.to_string(),
            parent: None,
            children: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            cancellation: tokio_util::sync::CancellationToken::new(),
        })
    }
    
    /// Create child context
    pub fn child(&self, name: &str) -> ContextBuilder {
        let child_context = std::sync::Arc::new(Context {
            name: name.to_string(),
            parent: Some(std::sync::Arc::new(self.clone())),
            children: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            cancellation: self.cancellation.child_token(),
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
    
    /// Wait for cancellation signal
    pub async fn wait(&self) {
        self.cancellation.cancelled().await;
    }
    
    /// Cancel this context and all children recursively
    pub fn cancel(&self) {
        self.cancellation.cancel();
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Context {
            name: self.name.clone(),
            parent: self.parent.clone(),
            children: self.children.clone(),
            cancellation: self.cancellation.clone(),
        }
    }
}

/// Builder for configuring child contexts before spawning
pub struct ContextBuilder {
    context: std::sync::Arc<Context>,
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
        tokio::spawn(async move {
            task(context).await
        })
    }
}

/// Global context storage
static GLOBAL_CONTEXT: std::sync::LazyLock<std::sync::Arc<Context>> = 
    std::sync::LazyLock::new(|| Context::new("global"));

/// Get the global application context
pub fn global() -> std::sync::Arc<Context> {
    GLOBAL_CONTEXT.clone()
}