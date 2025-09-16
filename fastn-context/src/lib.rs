#![warn(unused_extern_crates)]
#![deny(unused_crate_dependencies)]

extern crate self as fastn_context;

use tokio as _; // used by main macro

/// Hierarchical context for task management and cancellation
pub struct Context {
    /// Context name for debugging
    pub name: String,
    
    /// Parent context (None for root)
    parent: Option<std::sync::Arc<Context>>,
    
    /// Child contexts
    children: std::sync::Arc<std::sync::Mutex<Vec<std::sync::Arc<Context>>>>,
    
    /// Simple cancellation flag
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Context {
    /// Create new root context (typically only used by main macro)
    pub fn new(name: &str) -> std::sync::Arc<Context> {
        std::sync::Arc::new(Context {
            name: name.to_string(),
            parent: None,
            children: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }
    
    /// Create child context
    pub fn child(&self, name: &str) -> ContextBuilder {
        let child_context = std::sync::Arc::new(Context {
            name: name.to_string(),
            parent: Some(std::sync::Arc::new(self.clone())),
            children: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
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
        // Simple polling approach for now
        loop {
            if self.is_cancelled() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }
    
    /// Check if this context is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::Relaxed) ||
        self.parent.as_ref().map_or(false, |p| p.is_cancelled())
    }
    
    /// Cancel this context and all children recursively
    pub fn cancel(&self) {
        self.cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
        
        // Cancel all children
        if let Ok(children) = self.children.lock() {
            for child in children.iter() {
                child.cancel();
            }
        }
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Context {
            name: self.name.clone(),
            parent: self.parent.clone(),
            children: self.children.clone(),
            cancelled: self.cancelled.clone(),
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

// Re-export main macro
pub use fastn_context_macros::main;