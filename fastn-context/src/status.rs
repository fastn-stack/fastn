/// Status snapshot of the context tree
#[derive(Debug, Clone)]
pub struct Status {
    pub global_context: ContextStatus,
    pub persisted_contexts: Option<Vec<ContextStatus>>,
    pub timestamp: std::time::SystemTime,
}

/// Status information for a single context
#[derive(Debug, Clone)]
pub struct ContextStatus {
    pub name: String,
    pub is_cancelled: bool,
    pub duration: std::time::Duration,
    pub children: Vec<ContextStatus>,
}

/// Persisted context for distributed tracing
#[derive(Debug, Clone)]
pub struct PersistedContext {
    pub name: String,
    pub context_path: String,
    pub duration: std::time::Duration,
    pub completion_time: std::time::SystemTime,
    pub success: bool,
    pub message: String,
}

/// Global storage for persisted contexts (circular buffer)
static PERSISTED_CONTEXTS: std::sync::LazyLock<
    std::sync::RwLock<std::collections::VecDeque<PersistedContext>>,
> = std::sync::LazyLock::new(|| std::sync::RwLock::new(std::collections::VecDeque::new()));

/// Maximum number of persisted contexts to keep (configurable via env)
const MAX_PERSISTED_CONTEXTS: usize = 10; // TODO: Make configurable via env var

/// Add a context to the persisted contexts circular buffer
pub fn add_persisted_context(persisted: PersistedContext) {
    if let Ok(mut contexts) = PERSISTED_CONTEXTS.write() {
        // Add to front
        contexts.push_front(persisted.clone());

        // Keep only max number
        if contexts.len() > MAX_PERSISTED_CONTEXTS {
            contexts.pop_back();
        }
    }

    // Log as trace event
    println!(
        "TRACE: {} completed in {:?} - {}",
        persisted.context_path, persisted.duration, persisted.message
    );
}

/// Get current status snapshot of entire context tree
pub fn status() -> Status {
    Status {
        global_context: crate::context::global().status(),
        persisted_contexts: None,
        timestamp: std::time::SystemTime::now(),
    }
}

/// Get status including recent completed contexts (distributed tracing)
pub fn status_with_latest() -> Status {
    let persisted = if let Ok(contexts) = PERSISTED_CONTEXTS.read() {
        Some(contexts.iter().cloned().collect())
    } else {
        None
    };

    Status {
        global_context: crate::context::global().status(),
        persisted_contexts: persisted,
        timestamp: std::time::SystemTime::now(),
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "fastn Context Status")?;
        writeln!(f, "Snapshot: {:?}", self.timestamp)?;
        writeln!(f)?;

        Self::display_context(&self.global_context, f, 0)?;

        // Show persisted contexts if included
        if let Some(persisted) = &self.persisted_contexts
            && !persisted.is_empty()
        {
            writeln!(f, "\nRecent completed contexts (last {}):", persisted.len())?;
            for ctx in persisted {
                let duration_str = if ctx.duration.as_secs() > 60 {
                    format!(
                        "{}m {}s",
                        ctx.duration.as_secs() / 60,
                        ctx.duration.as_secs() % 60
                    )
                } else {
                    format!("{:.1}s", ctx.duration.as_secs_f64())
                };

                let status_str = if ctx.success { "success" } else { "failed" };
                writeln!(
                    f,
                    "- {} ({}, {}: \"{}\")",
                    ctx.context_path, duration_str, status_str, ctx.message
                )?;
            }
        }

        Ok(())
    }
}

impl Status {
    fn display_context(
        ctx: &ContextStatus,
        f: &mut std::fmt::Formatter<'_>,
        depth: usize,
    ) -> std::fmt::Result {
        let indent = "  ".repeat(depth);
        let status_icon = if ctx.is_cancelled { "❌" } else { "✅" };

        let duration_str = if ctx.duration.as_secs() > 60 {
            format!(
                "{}m {}s",
                ctx.duration.as_secs() / 60,
                ctx.duration.as_secs() % 60
            )
        } else {
            format!("{:.1}s", ctx.duration.as_secs_f64())
        };

        writeln!(
            f,
            "{}{} {} ({}, {})",
            indent,
            status_icon,
            ctx.name,
            duration_str,
            if ctx.is_cancelled {
                "cancelled"
            } else {
                "active"
            }
        )?;

        for child in &ctx.children {
            Self::display_context(child, f, depth + 1)?;
        }

        Ok(())
    }
}
