#[derive(Debug)]
pub enum Element {
    Container(Container),
    Text(Box<Text>),
    Image(Image),
}

#[derive(Copy, Clone)]
pub enum ElementKind {
    Column,
    Row,
    Text,
    Image,
    Container,
    IFrame,
    Integer,
    Decimal,
    Boolean,
}

impl From<i32> for ElementKind {
    fn from(i: i32) -> ElementKind {
        match i {
            0 => ElementKind::Column,
            1 => ElementKind::Row,
            2 => ElementKind::Text,
            3 => ElementKind::Image,
            4 => ElementKind::Container,
            5 => ElementKind::IFrame,
            6 => ElementKind::Integer,
            7 => ElementKind::Decimal,
            8 => ElementKind::Boolean,
            _ => panic!("Unknown element kind: {}", i),
        }
    }
}

impl From<ElementKind> for i32 {
    fn from(s: ElementKind) -> i32 {
        match s {
            ElementKind::Column => 0,
            ElementKind::Row => 1,
            ElementKind::Text => 2,
            ElementKind::Image => 3,
            ElementKind::Container => 4,
            ElementKind::IFrame => 5,
            ElementKind::Integer => 6,
            ElementKind::Decimal => 7,
            ElementKind::Boolean => 8,
        }
    }
}

#[derive(Debug)]
pub struct I32Pointer(fastn_runtime::PointerKey);

#[derive(Debug)]
pub enum Align {
    Left,
    Right,
    Justify,
}

#[derive(Debug)]
pub struct CommonStyle {
    pub background_color: Option<I32Pointer>,
    pub padding: Option<fastn_runtime::PointerKey>,
    pub align: Option<Align>,
    // border: Borders,
}

#[derive(Debug)]
pub struct Container {
    #[cfg(not(feature = "browser"))]
    pub taffy_key: taffy::node::Node,
    pub style: CommonStyle,
}

#[cfg(not(feature = "browser"))]
impl Container {
    pub(crate) fn outer_column(taffy: &mut taffy::Taffy) -> Element {
        Element::Container(Container {
            taffy_key: taffy
                .new_leaf(taffy::style::Style {
                    size: taffy::prelude::Size {
                        width: taffy::prelude::percent(100.0),
                        height: taffy::prelude::percent(100.0),
                    },
                    gap: taffy::prelude::points(20.0),
                    ..Default::default()
                })
                .expect("this should never fail"),
            style: CommonStyle {
                // background_color: Some(
                //     fastn_runtime::Color {
                //         red: 20,
                //         green: 0,
                //         blue: 0,
                //         alpha: 1.0,
                //     }
                //     .into(),
                // ),
                background_color: None,
                padding: None,
                align: None,
            },
        })
    }
}

#[derive(Debug)]
pub struct Text {
    #[cfg(not(feature = "browser"))]
    pub taffy: taffy::node::Node,
    pub text: fastn_runtime::PointerKey,
    pub role: fastn_runtime::ResponsiveProperty<fastn_runtime::PointerKey>,
    pub style: CommonStyle,
}

#[derive(Debug)]
pub struct Image {
    #[cfg(not(feature = "browser"))]
    pub taffy: taffy::node::Node,
    pub style: CommonStyle,
    pub src: fastn_runtime::DarkModeProperty<fastn_runtime::PointerKey>,
}

// #[derive(Default, Debug)]
// pub struct Borders {
//     top: BorderEdge,
//     right: BorderEdge,
//     bottom: BorderEdge,
//     left: BorderEdge,
//     top_left_radius: Dimension,
//     top_right_radius: Dimension,
//     bottom_left_radius: Dimension,
//     bottom_right_radius: Dimension,
// }
//
// #[derive(Default, Debug)]
// pub struct BorderEdge {
//     color: Option<ftd::executor::Color>,
//     style: BorderStyle,
//     width: Dimension,
// }
//
// #[derive(Default, Debug)]
// pub enum BorderStyle {
//     Dotted,
//     Dashed,
//     Solid,
//     Double,
//     Groove,
//     Ridge,
//     Inset,
//     Outset,
//     Hidden,
//     #[default]
//     None,
// }

#[derive(Default, Debug)]
pub enum Dimension {
    Undefined,
    #[default]
    Auto,
    Px(u32),
    Percent(f32),
}

#[cfg(not(feature = "browser"))]
impl fastn_runtime::Element {
    pub fn render(&self, t: &taffy::Taffy) {
        dbg!(self);
        match self {
            fastn_runtime::Element::Container(c) => {
                dbg!(t.layout(c.taffy_key).unwrap());
                // for child in c.children.iter() {
                //     child.render(t);
                // }
            }
            fastn_runtime::Element::Text(c) => {
                dbg!(t.layout(c.taffy).unwrap());
            }
            fastn_runtime::Element::Image(c) => {
                dbg!(t.layout(c.taffy).unwrap());
            }
        };
    }

    pub fn taffy(&self) -> taffy::node::Node {
        match self {
            fastn_runtime::Element::Container(c) => c.taffy_key,
            fastn_runtime::Element::Text(t) => t.taffy,
            fastn_runtime::Element::Image(i) => i.taffy,
        }
    }

    pub fn common_styles(&mut self) -> &mut CommonStyle {
        match self {
            fastn_runtime::Element::Container(c) => &mut c.style,
            t => unimplemented!("{:?}", t),
        }
    }
}
