use itertools::Itertools;
use taffy::prelude::{points, zero};
type Color = String;

enum Element {
    Container(Container),
    Text(Text),
    Image(Image),
}

struct Container {
    taffy: taffy::node::Node,
    border: Borders,
    background_color: Option<String>,
    children: Vec<Element>,
}

struct Text {
    taffy: taffy::node::Node,
    border: Borders,
    text: String,
    color: Color,
}

struct Image {
    taffy: taffy::node::Node,
    border: Borders,
    src: String,
}

#[derive(Default)]
pub struct Borders {
    top: BorderEdge,
    right: BorderEdge,
    bottom: BorderEdge,
    left: BorderEdge,
    top_left_radius: Dimension,
    top_right_radius: Dimension,
    bottom_left_radius: Dimension,
    bottom_right_radius: Dimension,
}

#[derive(Default)]
pub struct BorderEdge {
    color: Option<Color>,
    style: BorderStyle,
    width: Dimension,
}

#[derive(Default)]
pub enum BorderStyle {
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
    Hidden,
    #[default]
    None,
}

#[derive(Default)]
pub enum Dimension {
    Undefined,
    #[default]
    Auto,
    Px(u32),
    Percent(f32),
}

impl ftd::executor::Element {
    fn to_taffy(&self, t: &mut taffy::Taffy) -> Element {
        match self {
            ftd::executor::Element::Column(c) => c.to_taffy(t),
            ftd::executor::Element::Row(c) => c.to_taffy(t),
            ftd::executor::Element::Container(c) => c.to_taffy(t),
            ftd::executor::Element::Document(c) => c.to_taffy(t),
            ftd::executor::Element::Text(c) => c.to_taffy(t),
            ftd::executor::Element::Integer(c) => c.to_taffy(t),
            ftd::executor::Element::Boolean(c) => c.to_taffy(t),
            ftd::executor::Element::Decimal(c) => c.to_taffy(t),
            ftd::executor::Element::Image(c) => c.to_taffy(t),
            ftd::executor::Element::Code(c) => c.to_taffy(t),
            ftd::executor::Element::Iframe(c) => c.to_taffy(t),
            ftd::executor::Element::TextInput(c) => c.to_taffy(t),
            ftd::executor::Element::RawElement(c) => c.to_taffy(t),
            ftd::executor::Element::IterativeElement(c) => c.to_taffy(t),
            ftd::executor::Element::CheckBox(c) => c.to_taffy(t),
            ftd::executor::Element::WebComponent(c) => c.to_taffy(t),
            ftd::executor::Element::Rive(c) => c.to_taffy(t),
            _ => todo!(),
        }
    }
}

impl ftd::executor::Length {
    fn to_dimension(&self) -> taffy::prelude::Dimension {
        match self {
            ftd::executor::Length::Px(v) => taffy::prelude::Dimension::Points(*v as f32),
            _ => todo!(),
        }
    }
}

impl ftd::executor::Resizing {
    fn to_dimension(&self) -> taffy::prelude::Dimension {
        dbg!(match { self } {
            ftd::executor::Resizing::Fixed(f) => f.to_dimension(),
            _ => taffy::prelude::Dimension::Auto,
        })
    }
}
impl ftd::executor::Column {
    fn to_taffy(&self, t: &mut taffy::Taffy) -> Element {
        let s = taffy::style::Style {
            display: Default::default(),
            position: Default::default(),
            inset: zero(),
            size: taffy::prelude::Size {
                width: self
                    .common
                    .width
                    .value
                    .as_ref()
                    .map(|v| v.to_dimension())
                    .unwrap_or(taffy::prelude::auto()),
                height: taffy::prelude::auto(),
            },
            min_size: zero(),
            max_size: zero(),
            aspect_ratio: None,
            margin: zero(),
            padding: zero(),
            border: zero(),
            align_items: None,
            align_self: None,
            justify_items: None,
            justify_self: None,
            align_content: None,
            justify_content: None,
            gap: zero(),
            flex_direction: Default::default(),
            flex_wrap: Default::default(),
            flex_basis: taffy::prelude::Dimension::Auto,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            grid_template_rows: vec![],
            grid_template_columns: vec![],
            grid_auto_rows: vec![],
            grid_auto_columns: vec![],
            grid_auto_flow: Default::default(),
            grid_row: Default::default(),
            grid_column: Default::default(),
        };

        let children = self
            .container
            .children
            .iter()
            .map(|c| c.to_taffy(t))
            .collect_vec();

        Element::Container(Container {
            taffy: t.new_with_children(s, &children.iter().map(|v| v.root_taffy()).collect_vec()).unwrap(),
            border: Default::default(),
            background_color: None,
            children,
        })
    }
}
impl ftd::executor::Row {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> Element {
        todo!()
    }
}
impl ftd::executor::ContainerElement {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> Element {
        todo!()
    }
}
impl ftd::executor::Document {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> Element {
        todo!()
    }
}
impl ftd::executor::Text {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> Element {
        todo!()
    }
}
impl ftd::executor::Image {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> Element {
        todo!()
    }
}
impl ftd::executor::Code {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> Element {
        todo!()
    }
}
impl ftd::executor::Iframe {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> Element {
        todo!()
    }
}
impl ftd::executor::TextInput {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> Element {
        todo!()
    }
}
impl ftd::executor::RawElement {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> Element {
        todo!()
    }
}
impl ftd::executor::IterativeElement {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> Element {
        todo!()
    }
}
impl ftd::executor::CheckBox {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> Element {
        todo!()
    }
}
impl ftd::executor::WebComponent {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> Element {
        todo!()
    }
}
impl ftd::executor::Rive {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> Element {
        todo!()
    }
}

impl Element {
    fn render(&self, _t: &taffy::Taffy) {
        todo!()
    }

    fn root_taffy(&self) -> taffy::node::Node {
        match self {
            Element::Container(c) => c.taffy,
            _ => unreachable!(),
        }
    }
}

fn ftd() -> ftd::executor::Element {
    let doc = ftd::test_helper::ftd_v2_interpret_helper("foo", ftd::taffy())
        .unwrap_or_else(|e| panic!("{:?}", e));
    ftd::executor::Element::Column(
        ftd::executor::ExecuteDoc::from_interpreter(doc)
            .unwrap_or_else(|e| panic!("{:?}", e))
            .main,
    )
}

pub fn run() {
    t2();

    let mut taffy = taffy::Taffy::new();
    let f = ftd().to_taffy(&mut taffy);

    taffy
        .compute_layout(
            f.root_taffy(),
            taffy::prelude::Size {
                width: points(100.0),
                height: points(100.0),
            },
        )
        .unwrap();

    dbg!(taffy.layout(f.root_taffy()));
    f.render(&taffy)
}

fn t2() {
    use taffy::prelude::*;
    let mut taffy = Taffy::new();

    let (width, height) = crossterm::terminal::size().unwrap();

    // Create a tree of nodes using `taffy.new_leaf` and `taffy.new_with_children`.
    // These functions both return a node id which can be used to refer to that node
    // The Style struct is used to specify styling information
    let header_node = taffy
        .new_leaf(Style {
            size: Size {
                width: points(800.0),
                height: points(500.0),
            },
            ..Default::default()
        })
        .unwrap();

    let body_node = taffy
        .new_leaf(Style {
            size: Size {
                width: points(600.0),
                height: auto(),
            },
            flex_grow: 1.0,
            ..Default::default()
        })
        .unwrap();

    let root_node = taffy
        .new_with_children(
            Style {
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: points(800.0),
                    height: points(700.0),
                },
                ..Default::default()
            },
            &[header_node, body_node],
        )
        .unwrap();

    taffy
        .compute_layout(
            root_node,
            Size {
                width: points(width as f32),
                height: points(height as f32),
            },
        )
        .unwrap();

    dbg!(taffy.layout(root_node).unwrap());
    dbg!(taffy.layout(header_node).unwrap());
    dbg!(taffy.layout(body_node).unwrap());
}
