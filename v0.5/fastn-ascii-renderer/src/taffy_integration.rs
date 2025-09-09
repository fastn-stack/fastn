use taffy::{TaffyTree, NodeId, Style, Layout};

/// Taffy layout engine integration for FTD components
pub struct TaffyLayoutEngine {
    tree: TaffyTree,
    root_node: Option<NodeId>,
}

impl TaffyLayoutEngine {
    pub fn new() -> Self {
        Self {
            tree: TaffyTree::new(),
            root_node: None,
        }
    }

    /// Create a simple text node for Week 1 prototype
    pub fn create_text_node(&mut self, _text: &str, style: Style) -> Result<NodeId, taffy::TaffyError> {
        // For Week 1: Create leaf node with text content
        // Text measurement will be simplified - just character count
        let node = self.tree.new_leaf(style)?;
        
        // Store text content (simplified - no proper text measurement yet)
        // In Week 3 we'll add proper text measurement
        
        Ok(node)
    }

    /// Create container node (column/row)
    pub fn create_container_node(&mut self, style: Style, children: Vec<NodeId>) -> Result<NodeId, taffy::TaffyError> {
        let node = self.tree.new_with_children(style, &children)?;
        Ok(node)
    }

    /// Set root node for layout calculation
    pub fn set_root(&mut self, node: NodeId) {
        self.root_node = Some(node);
    }

    /// Compute layout with given available space
    pub fn compute_layout(&mut self, available_space: taffy::Size<taffy::AvailableSpace>) -> Result<(), taffy::TaffyError> {
        if let Some(root) = self.root_node {
            self.tree.compute_layout(root, available_space)?;
        }
        Ok(())
    }

    /// Get computed layout for a node
    pub fn get_layout(&self, node: NodeId) -> Result<&Layout, taffy::TaffyError> {
        self.tree.layout(node)
    }

    /// Get all layouts for debugging
    pub fn debug_layouts(&self) -> Vec<(NodeId, Layout)> {
        // For Week 1: Simple debug output
        if let Some(root) = self.root_node {
            self.collect_layouts_recursive(root)
        } else {
            vec![]
        }
    }

    fn collect_layouts_recursive(&self, node: NodeId) -> Vec<(NodeId, Layout)> {
        let mut layouts = vec![];
        
        if let Ok(layout) = self.tree.layout(node) {
            layouts.push((node, *layout));
            
            // Add children  
            if let Ok(children) = self.tree.children(node) {
                for child in children {
                    layouts.extend(self.collect_layouts_recursive(child));
                }
            }
        }
        
        layouts
    }
}

impl Default for TaffyLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}