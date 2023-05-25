use itertools::Itertools;
use taffy::prelude::points;

impl ftd::executor::Element {
    fn to_taffy(&self, t: &mut taffy::Taffy) -> fastn_runtime::Element {
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
    fn dim(&self) -> taffy::prelude::Dimension {
        match self {
            ftd::executor::Length::Px(v) => taffy::prelude::Dimension::Points(*v as f32),
            _ => todo!(),
        }
    }
    fn lpa(&self) -> taffy::prelude::LengthPercentageAuto {
        match self {
            ftd::executor::Length::Px(v) => taffy::prelude::LengthPercentageAuto::Points(*v as f32),
            _ => todo!(),
        }
    }
    fn lp(&self) -> taffy::prelude::LengthPercentage {
        match self {
            ftd::executor::Length::Px(v) => taffy::prelude::LengthPercentage::Points(*v as f32),
            _ => todo!(),
        }
    }
}

impl ftd::executor::Resizing {
    fn dim(&self) -> taffy::prelude::Dimension {
        match self {
            ftd::executor::Resizing::Fixed(f) => f.dim(),
            _ => taffy::prelude::Dimension::Auto,
        }
    }
}

impl ftd::executor::Value<Option<ftd::executor::Resizing>> {
    fn dim(&self) -> taffy::prelude::Dimension {
        self.value
            .as_ref()
            .map(|v| v.dim())
            .unwrap_or(taffy::prelude::auto())
    }
}

impl ftd::executor::Value<Option<ftd::executor::Length>> {
    fn lpa(&self, f1: &Self, f2: &Self) -> taffy::prelude::LengthPercentageAuto {
        self.value
            .as_ref()
            .or(f1.value.as_ref().or(f2.value.as_ref()))
            .map(|v| v.lpa())
            .unwrap_or(taffy::prelude::auto())
    }

    fn lp(&self, f1: &Self, f2: &Self) -> taffy::prelude::LengthPercentage {
        self.value
            .as_ref()
            .or(f1.value.as_ref().or(f2.value.as_ref()))
            .map(|v| v.lp())
            .unwrap_or(taffy::prelude::LengthPercentage::Points(0.0))
    }
}

impl ftd::executor::Common {
    fn to_style(&self) -> taffy::style::Style {
        taffy::style::Style {
            display: taffy::prelude::Display::Flex,
            size: taffy::prelude::Size {
                width: self.width.dim(),
                height: self.height.dim(),
            },
            margin: taffy::prelude::Rect {
                left: self.margin_left.lpa(&self.margin_vertical, &self.margin),
                right: self.margin_right.lpa(&self.margin_vertical, &self.margin),
                top: self.margin_top.lpa(&self.margin_horizontal, &self.margin),
                bottom: self
                    .margin_bottom
                    .lpa(&self.margin_horizontal, &self.margin),
            },
            padding: taffy::prelude::Rect {
                left: self.padding_left.lp(&self.padding_vertical, &self.padding),
                right: self.padding_right.lp(&self.padding_vertical, &self.padding),
                top: self.padding_top.lp(&self.padding_horizontal, &self.padding),
                bottom: self
                    .padding_bottom
                    .lp(&self.padding_horizontal, &self.padding),
            },
            ..Default::default()
        }
    }
}

impl ftd::executor::Value<Option<ftd::executor::Background>> {
    fn to_color(&self) -> Option<fastn_runtime::Color> {
        self.value.as_ref().map(|v| match v {
            ftd::executor::Background::Solid(c) => c.to_surface_color(),
            _ => todo!(),
        })
    }
}

impl ftd::executor::Container {
    fn child_elements(&self, t: &mut taffy::Taffy) -> Vec<fastn_runtime::Element> {
        self.children.iter().map(|c| c.to_taffy(t)).collect_vec()
    }
}

fn element_from_container(
    direction: taffy::prelude::FlexDirection,
    common: &ftd::executor::Common,
    container: &ftd::executor::Container,
    t: &mut taffy::Taffy,
) -> fastn_runtime::Element {
    let mut s = common.to_style();
    s.flex_direction = direction;
    let children = container.child_elements(t);

    fastn_runtime::Element::Container(fastn_runtime::Container {
        taffy_key: t
            .new_with_children(s, &children.iter().map(|v| v.taffy()).collect_vec())
            .unwrap(),
        // border: Default::default(), // TODO
        background_color: common.background.to_color(),
        children,
    })
}

impl ftd::executor::Column {
    fn to_taffy(&self, t: &mut taffy::Taffy) -> fastn_runtime::Element {
        element_from_container(
            taffy::prelude::FlexDirection::Column,
            &self.common,
            &self.container,
            t,
        )
    }
}

impl ftd::executor::Row {
    fn to_taffy(&self, t: &mut taffy::Taffy) -> fastn_runtime::Element {
        element_from_container(
            taffy::prelude::FlexDirection::Row,
            &self.common,
            &self.container,
            t,
        )
    }
}

impl ftd::executor::ContainerElement {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> fastn_runtime::Element {
        todo!()
    }
}

impl ftd::executor::Document {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> fastn_runtime::Element {
        todo!()
    }
}

impl ftd::executor::Text {
    fn to_taffy(&self, t: &mut taffy::Taffy) -> fastn_runtime::Element {
        fastn_runtime::Element::Text(fastn_runtime::Text {
            taffy: t.new_leaf(self.common.to_style()).unwrap(),
            // border: Default::default(),
            text: self.text.value.original.clone(),
            style: self.style.value.as_ref().map(|v| v.to_surface_style()),
            color: self
                .common
                .color
                .value
                .as_ref()
                .map(|v| v.to_surface_color()),
        })
    }
}

impl ftd::executor::TextStyle {
    fn to_surface_style(&self) -> fastn_runtime::TextStyle {
        todo!()
    }
}

impl ftd::executor::Color {
    fn to_surface_color(&self) -> fastn_runtime::Color {
        todo!()
    }
}

impl ftd::executor::Image {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> fastn_runtime::Element {
        todo!()
    }
}

impl ftd::executor::Code {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> fastn_runtime::Element {
        todo!()
    }
}

impl ftd::executor::Iframe {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> fastn_runtime::Element {
        todo!()
    }
}

impl ftd::executor::TextInput {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> fastn_runtime::Element {
        todo!()
    }
}

impl ftd::executor::RawElement {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> fastn_runtime::Element {
        todo!()
    }
}

impl ftd::executor::IterativeElement {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> fastn_runtime::Element {
        todo!()
    }
}

impl ftd::executor::CheckBox {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> fastn_runtime::Element {
        todo!()
    }
}

impl ftd::executor::WebComponent {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> fastn_runtime::Element {
        todo!()
    }
}

impl ftd::executor::Rive {
    fn to_taffy(&self, _t: &mut taffy::Taffy) -> fastn_runtime::Element {
        todo!()
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
            f.taffy(),
            taffy::prelude::Size {
                width: points(100.0),
                height: points(100.0),
            },
        )
        .unwrap();

    f.render(&taffy)
}

fn t2() {
    use taffy::prelude::*;
    let mut taffy = Taffy::new();

    let (width, height) = crossterm::terminal::size().unwrap();

    // Create a tree of nodes using `taffy_node.new_leaf` and `taffy_node.new_with_children`.
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
