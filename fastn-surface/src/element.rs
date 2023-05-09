#[derive(Debug)]
pub enum Element {
    Container(Container),
    Text(Text),
    Image(Image),
}

#[derive(Debug)]
pub struct CommonStyleMinusTaffy {
    pub background_color: Option<fastn_surface::Color>,
    // border: Borders,
}

#[derive(Debug)]
pub struct Container {
    pub taffy: taffy::node::Node,
    pub style: CommonStyleMinusTaffy,
}

impl Container {
    pub(crate) fn outer_column(taffy: &mut taffy::Taffy) -> Element {
        Element::Container(Container {
            taffy: taffy
                .new_leaf(taffy::style::Style::default())
                .expect("this should never fail"),
            style: CommonStyleMinusTaffy {
                background_color: None,
            },
        })
    }
}

#[derive(Debug)]
pub struct Text {
    pub taffy: taffy::node::Node,
    // border: Borders,
    pub text: String,
    pub style: Option<fastn_surface::TextStyle>,
    pub color: Option<fastn_surface::Color>,
}

#[derive(Debug)]
pub struct Image {
    pub taffy: taffy::node::Node,
    // border: Borders,
    pub src: String,
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

impl fastn_surface::Element {
    pub fn render(&self, t: &taffy::Taffy) {
        dbg!(self);
        match self {
            fastn_surface::Element::Container(c) => {
                dbg!(t.layout(c.taffy).unwrap());
                // for child in c.children.iter() {
                //     child.render(t);
                // }
            }
            fastn_surface::Element::Text(c) => {
                dbg!(t.layout(c.taffy).unwrap());
            }
            fastn_surface::Element::Image(c) => {
                dbg!(t.layout(c.taffy).unwrap());
            }
        };
    }

    pub fn taffy(&self) -> taffy::node::Node {
        match self {
            fastn_surface::Element::Container(c) => c.taffy,
            fastn_surface::Element::Text(t) => t.taffy,
            fastn_surface::Element::Image(i) => i.taffy,
        }
    }
}
