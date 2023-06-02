#[derive(Debug)]
pub enum Element {
    Container(Container),
    Text(Box<Text>),
    Image(Image),
}

#[derive(Debug)]
pub struct CommonStyleMinusTaffy {
    pub background_color: Option<fastn_runtime::ColorValue>,
    // border: Borders,
}

#[derive(Debug)]
pub struct Container {
    // if not wasm
    pub taffy_key: taffy::node::Node,
    pub style: CommonStyleMinusTaffy,
    pub events: ElementEvent,
}

pub type ElementEvent =
    std::collections::HashMap<fastn_runtime::EventKind, Vec<fastn_runtime::ClosurePointer>>;

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
            style: CommonStyleMinusTaffy {
                background_color: Some(fastn_runtime::ColorValue {
                    red: 20,
                    green: 0,
                    blue: 0,
                    alpha: 1.0,
                }),
            },
            events: Default::default(),
        })
    }
}

#[derive(Debug)]
pub struct Text {
    pub taffy: taffy::node::Node,
    pub text: fastn_runtime::Callable<String>,
    pub style: fastn_runtime::TextStyle,
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

    pub fn common_styles(&mut self) -> &mut CommonStyleMinusTaffy {
        match self {
            fastn_runtime::Element::Container(c) => &mut c.style,
            t => unimplemented!("{:?}", t),
        }
    }

    pub fn events(&mut self) -> &mut ElementEvent {
        match self {
            fastn_runtime::Element::Container(c) => &mut c.events,
            t => unimplemented!("{:?}", t),
        }
    }

    pub fn add_events(
        &mut self,
        event: fastn_runtime::EventKind,
        closure: fastn_runtime::ClosurePointer,
    ) {
        let events = self.events();
        if let Some(closures) = events.get_mut(&event) {
            closures.push(closure);
        } else {
            events.insert(event, vec![closure]);
        }
    }
}
