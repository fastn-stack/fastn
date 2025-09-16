/// Status snapshot of the context tree
#[derive(Debug, Clone)]
pub struct Status {
    pub global_context: ContextStatus,
    pub timestamp: std::time::SystemTime,
}

/// Status information for a single context
#[derive(Debug, Clone)]
pub struct ContextStatus {
    pub name: String,
    pub is_cancelled: bool,
    pub children: Vec<ContextStatus>,
}

/// Get current status snapshot of entire context tree
pub fn status() -> Status {
    Status {
        global_context: crate::context::global().status(),
        timestamp: std::time::SystemTime::now(),
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "fastn Context Status")?;
        writeln!(f, "Snapshot: {:?}", self.timestamp)?;
        writeln!(f)?;
        
        Self::display_context(&self.global_context, f, 0)
    }
}

impl Status {
    fn display_context(ctx: &ContextStatus, f: &mut std::fmt::Formatter<'_>, depth: usize) -> std::fmt::Result {
        let indent = "  ".repeat(depth);
        let status_icon = if ctx.is_cancelled { "❌" } else { "✅" };
        
        writeln!(f, "{}{} {} ({})", 
            indent, 
            status_icon,
            ctx.name,
            if ctx.is_cancelled { "cancelled" } else { "active" }
        )?;
        
        for child in &ctx.children {
            Self::display_context(child, f, depth + 1)?;
        }
        
        Ok(())
    }
}