pub struct Edges {
    pub top: Option<i32>,
    pub bottom: Option<i32>,
    pub left: i32,
    pub right: i32,
}

pub struct Row {
    pub padding: Edges,
    pub children: Vec<crate::raw::Element>,
}
