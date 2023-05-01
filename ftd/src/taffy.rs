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

pub struct BorderEdge {
    color: Option<Color>,
    style: BorderStyle,
    width: Dimension,
}

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
    None,
}

pub enum Dimension {
    Undefined,
    Auto,
    Px(u32),
    Percent(f32),
}

fn ftd() -> ftd::executor::Element {
    let doc = ftd::test_helper::ftd_v2_interpret_helper("foo", ftd::terminal())
        .unwrap_or_else(|e| panic!("{:?}", e));
    ftd::executor::Element::Column(
        ftd::executor::ExecuteDoc::from_interpreter(doc)
            .unwrap_or_else(|e| panic!("{:?}", e))
            .main,
    )
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

impl ftd::executor::Column {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> Element {
        todo!()
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

pub fn run() {
    t2();

    let mut taffy = taffy::Taffy::new();
    let f = ftd().to_taffy(&mut taffy);
    use taffy::prelude::TaffyMaxContent;

    taffy
        .compute_layout(f.root_taffy(), taffy::prelude::Size::MAX_CONTENT)
        .unwrap();
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
                width: points(800.0),
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
                    height: points(500.0),
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
