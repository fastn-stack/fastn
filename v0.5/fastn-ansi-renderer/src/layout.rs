/// Layout constraints for component sizing
#[derive(Debug, Clone)]
pub struct LayoutConstraints {
    pub max_width: Option<usize>,
    pub max_height: Option<usize>,
    pub min_width: Option<usize>,
    pub min_height: Option<usize>,
}

impl Default for LayoutConstraints {
    fn default() -> Self {
        Self {
            max_width: None,
            max_height: None,
            min_width: None,
            min_height: None,
        }
    }
}

/// Calculated layout for a component
#[derive(Debug, Clone)]
pub struct ComponentLayout {
    pub width: usize,
    pub height: usize,
    pub content_width: usize,
    pub content_height: usize,
}

/// Overall ASCII layout tree
#[derive(Debug, Clone)]
pub struct AsciiLayout {
    pub width: usize,
    pub height: usize,
    pub children: Vec<ComponentLayout>,
}

impl AsciiLayout {
    pub fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            children: vec![],
        }
    }

    /// Render this layout to a canvas
    pub fn render(&self) -> crate::Canvas {
        crate::Canvas::new(self.width, self.height)
        // TODO: Render children
    }
}
