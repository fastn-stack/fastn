/// FTD property types for Week 1 prototype
/// These will evolve as we integrate with actual fastn v0.5 types

#[derive(Debug, Clone)]
pub struct SimpleFtdComponent {
    pub component_type: ComponentType,
    pub text: Option<String>,
    pub width: Option<FtdSize>,
    pub height: Option<FtdSize>,
    pub padding: Option<u32>, // Simplified to px only for Week 1
    pub margin: Option<u32>,  // Simplified to px only for Week 1
    pub border_width: Option<u32>,
    pub spacing: Option<u32>, // For containers
    pub children: Vec<SimpleFtdComponent>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    Text,
    Column,
    Row,
    Container,
}

#[derive(Debug, Clone)]
pub enum FtdSize {
    Fixed { px: u32 },
    FillContainer,
    HugContent,
    Percent { value: u32 },
}

impl SimpleFtdComponent {
    pub fn text(content: &str) -> Self {
        Self {
            component_type: ComponentType::Text,
            text: Some(content.to_string()),
            width: None,
            height: None,
            padding: None,
            margin: None,
            border_width: None,
            spacing: None,
            children: vec![],
        }
    }

    pub fn column() -> Self {
        Self {
            component_type: ComponentType::Column,
            text: None,
            width: None,
            height: None,
            padding: None,
            margin: None,
            border_width: None,
            spacing: None,
            children: vec![],
        }
    }

    pub fn with_width(mut self, size: FtdSize) -> Self {
        self.width = Some(size);
        self
    }

    pub fn with_height(mut self, size: FtdSize) -> Self {
        self.height = Some(size);
        self
    }

    pub fn with_padding(mut self, px: u32) -> Self {
        self.padding = Some(px);
        self
    }

    pub fn with_border(mut self, px: u32) -> Self {
        self.border_width = Some(px);
        self
    }

    pub fn with_spacing(mut self, px: u32) -> Self {
        self.spacing = Some(px);
        self
    }

    pub fn with_children(mut self, children: Vec<SimpleFtdComponent>) -> Self {
        self.children = children;
        self
    }
}
