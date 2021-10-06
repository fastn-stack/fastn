#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize)
)]
#[serde(tag = "type")]
pub enum Element {
    Text(Text),
    Image(Image),
    Row(Row),
    Column(Column),
    IFrame(IFrame),
    Input(Input),
    Integer(Text),
    Boolean(Text),
    Decimal(Text),
    Null,
}

impl Element {
    pub fn set_id(children: &mut [Self], index_vec: &[usize], external_id: Option<String>) {
        for (idx, child) in children.iter_mut().enumerate() {
            let index_string: String = {
                let mut index_vec = index_vec.to_vec();
                index_vec.push(idx);
                index_vec
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            };
            let mut id = match child {
                Self::Text(ftd_rt::Text {
                    common: ftd_rt::Common { id, .. },
                    ..
                }) => id,
                Self::Image(ftd_rt::Image {
                    common: ftd_rt::Common { id, .. },
                    ..
                }) => id,
                Self::Row(ftd_rt::Row {
                    common: ftd_rt::Common { id, .. },
                    container,
                }) => {
                    let mut index_vec = index_vec.to_vec();
                    index_vec.push(idx);
                    Self::set_id(&mut container.children, &index_vec, external_id.clone());
                    if let Some((id, container, external_children)) =
                        &mut container.external_children
                    {
                        if let Some(ftd_rt::Element::Column(col)) = external_children.first_mut() {
                            let index_string: String = index_vec
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<String>>()
                                .join(",");

                            let external_id = Some({
                                if let Some(ref ext_id) = external_id {
                                    format!("{}.{}-external:{}", ext_id, id, index_string)
                                } else {
                                    format!("{}-external:{}", id, index_string)
                                }
                            });
                            col.common.id = external_id.clone();
                            if let Some(val) = container.first_mut() {
                                index_vec.append(&mut val.to_vec());
                                Self::set_id(&mut col.container.children, &index_vec, external_id);
                            }
                        }
                    }
                    id
                }
                Self::Column(ftd_rt::Column {
                    common: ftd_rt::Common { id, .. },
                    container,
                }) => {
                    let mut index_vec = index_vec.to_vec();
                    index_vec.push(idx);
                    Self::set_id(&mut container.children, &index_vec, external_id.clone());
                    if let Some((id, container, external_children)) =
                        &mut container.external_children
                    {
                        if let Some(ftd_rt::Element::Column(col)) = external_children.first_mut() {
                            let index_string: String = index_vec
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<String>>()
                                .join(",");

                            let external_id = Some({
                                if let Some(ref ext_id) = external_id {
                                    format!("{}.{}-external:{}", ext_id, id, index_string)
                                } else {
                                    format!("{}-external:{}", id, index_string)
                                }
                            });
                            col.common.id = external_id.clone();
                            if let Some(val) = container.first_mut() {
                                index_vec.append(&mut val.to_vec());
                                Self::set_id(&mut col.container.children, &index_vec, external_id);
                            }
                        }
                    }
                    id
                }
                Self::IFrame(ftd_rt::IFrame {
                    common: ftd_rt::Common { id, .. },
                    ..
                }) => id,
                Self::Input(ftd_rt::Input {
                    common: ftd_rt::Common { id, .. },
                    ..
                }) => id,
                Self::Integer(ftd_rt::Text {
                    common: ftd_rt::Common { id, .. },
                    ..
                }) => id,
                Self::Boolean(ftd_rt::Text {
                    common: ftd_rt::Common { id, .. },
                    ..
                }) => id,
                Self::Decimal(ftd_rt::Text {
                    common: ftd_rt::Common { id, .. },
                    ..
                }) => id,
                Self::Null => continue,
            };

            let external_id = {
                if let Some(ref external_id) = external_id {
                    format!(":{}", external_id)
                } else {
                    "".to_string()
                }
            };

            if let Some(id) = &mut id {
                *id = format!("{}:{}{}", id, index_string, external_id);
            } else {
                *id = Some(format!("{}{}", index_string, external_id));
            }
        }
    }

    pub fn get_external_children_condition(
        &self,
        external_open_id: &Option<String>,
        external_children_container: &[Vec<usize>],
    ) -> Vec<ftd_rt::ExternalChildrenCondition> {
        let mut d: Vec<ftd_rt::ExternalChildrenCondition> = vec![];
        let mut ext_child_condition = None;
        let (id, open_id, children_container, children) = match self {
            Self::Row(ftd_rt::Row {
                common: ftd_rt::Common { id, .. },
                container:
                    ftd_rt::Container {
                        external_children,
                        children,
                        ..
                    },
            }) => (
                id,
                external_children
                    .as_ref()
                    .map(|(open_id, _, _)| open_id.to_string()),
                external_children
                    .as_ref()
                    .map(|(_, children_container, _)| children_container.to_vec()),
                children,
            ),
            Self::Column(ftd_rt::Column {
                common: ftd_rt::Common { id, .. },
                container:
                    ftd_rt::Container {
                        external_children,
                        children,
                        ..
                    },
            }) => (
                id,
                external_children
                    .as_ref()
                    .map(|(open_id, _, _)| open_id.to_string()),
                external_children
                    .as_ref()
                    .map(|(_, children_container, _)| children_container.to_vec()),
                children,
            ),
            _ => return d,
        };

        #[allow(clippy::blocks_in_if_conditions)]
        if *external_open_id
            == id.as_ref().map(|v| {
                if v.contains(':') {
                    let mut part = v.splitn(2, ':');
                    part.next().unwrap().trim().to_string()
                } else {
                    v.to_string()
                }
            })
            && external_children_container.is_empty()
        {
            ext_child_condition = id.clone();
            if open_id.is_none() {
                let id = ext_child_condition.expect("");
                d.push(ftd_rt::ExternalChildrenCondition {
                    condition: vec![id.to_string()],
                    set_at: id,
                });
                return d;
            }
        }

        let (open_id, external_children_container) =
            if open_id.is_some() && external_children_container.is_empty() {
                (open_id, {
                    if let Some(c) = children_container {
                        c
                    } else {
                        vec![]
                    }
                })
            } else {
                (
                    external_open_id.clone(),
                    external_children_container.to_vec(),
                )
            };

        let mut index = 0;
        for (i, v) in children.iter().enumerate() {
            let external_container = {
                let mut external_container = vec![];
                while index < external_children_container.len() {
                    if let Some(container) = external_children_container[index].get(0) {
                        if container < &i {
                            index += 1;
                            continue;
                        }
                        let external_child_container =
                            external_children_container[index][1..].to_vec();
                        if container == &i && !external_child_container.is_empty() {
                            external_container.push(external_child_container)
                        } else {
                            break;
                        }
                    }
                    index += 1;
                }
                external_container
            };
            let conditions =
                v.get_external_children_condition(&open_id, external_container.as_slice());
            for mut condition in conditions {
                if let Some(e) = &ext_child_condition {
                    condition.condition.push(e.to_string());
                }
                d.push(condition);
            }
        }
        d
    }

    pub fn get_external_children_dependencies(
        children: &[Self],
    ) -> ftd_rt::ExternalChildrenDependenciesMap {
        let mut d: ftd_rt::ExternalChildrenDependenciesMap = Default::default();
        for child in children {
            let container = match child {
                ftd_rt::Element::Row(ftd_rt::Row { container, .. }) => container,
                ftd_rt::Element::Column(ftd_rt::Column { container, .. }) => container,
                _ => continue,
            };
            let all_locals = Self::get_external_children_dependencies(&container.children);
            for (k, v) in all_locals {
                d.insert(k.to_string(), v);
            }
            if let Some((external_open_id, external_children_container, external_children)) =
                &container.external_children
            {
                if let Some(ftd_rt::Element::Column(col)) = external_children.first() {
                    let external_children_condition: Vec<ftd_rt::ExternalChildrenCondition> = child
                        .get_external_children_condition(
                            &Some(external_open_id.to_string()),
                            external_children_container,
                        );
                    d.insert(
                        col.common.id.as_ref().expect("").to_string(),
                        external_children_condition,
                    );
                    let all_locals =
                        Self::get_external_children_dependencies(&col.container.children);
                    for (k, v) in all_locals {
                        d.insert(k.to_string(), v);
                    }
                }
            }
        }
        d
    }

    pub fn get_event_dependencies(
        children: &[Self],
        data: &ftd_rt::Map,
    ) -> ftd_rt::DataDependenciesMap {
        let mut d: ftd_rt::DataDependenciesMap = Default::default();
        for child in children {
            let (condition, id) = match child {
                ftd_rt::Element::Column(ftd_rt::Column {
                    common: ftd_rt::Common { condition, id, .. },
                    container,
                })
                | ftd_rt::Element::Row(ftd_rt::Row {
                    common: ftd_rt::Common { condition, id, .. },
                    container,
                }) => {
                    let all_locals = Self::get_event_dependencies(&container.children, data);
                    for (k, v) in all_locals {
                        if let Some(d) = d.get_mut(&k) {
                            for (k, v) in v.dependencies {
                                d.dependencies.insert(k, v);
                            }
                        } else {
                            d.insert(k.to_string(), v);
                        }
                    }
                    if let Some((_, _, external_children)) = &container.external_children {
                        let all_locals = Self::get_event_dependencies(external_children, data);
                        for (k, v) in all_locals {
                            if let Some(d) = d.get_mut(&k) {
                                for (k, v) in v.dependencies {
                                    d.dependencies.insert(k, v);
                                }
                            } else {
                                d.insert(k.to_string(), v);
                            }
                        }
                    }
                    (condition, id)
                }
                ftd_rt::Element::Image(ftd_rt::Image {
                    common: ftd_rt::Common { condition, id, .. },
                    ..
                })
                | ftd_rt::Element::Text(ftd_rt::Text {
                    common: ftd_rt::Common { condition, id, .. },
                    ..
                })
                | ftd_rt::Element::IFrame(ftd_rt::IFrame {
                    common: ftd_rt::Common { condition, id, .. },
                    ..
                })
                | ftd_rt::Element::Input(ftd_rt::Input {
                    common: ftd_rt::Common { condition, id, .. },
                    ..
                })
                | ftd_rt::Element::Integer(ftd_rt::Text {
                    common: ftd_rt::Common { condition, id, .. },
                    ..
                })
                | ftd_rt::Element::Boolean(ftd_rt::Text {
                    common: ftd_rt::Common { condition, id, .. },
                    ..
                })
                | ftd_rt::Element::Decimal(ftd_rt::Text {
                    common: ftd_rt::Common { condition, id, .. },
                    ..
                }) => (condition, id),
                ftd_rt::Element::Null => continue,
            };
            if let Some(condition) = condition {
                let id = id.clone().expect("universal id should be present");
                let data_value = data
                    .get(&condition.variable)
                    .unwrap_or_else(|| panic!("{} should be declared", condition.variable));

                if let Some(ftd_rt::Data { dependencies, .. }) = d.get_mut(&condition.variable) {
                    dependencies.insert(id, condition.value.to_string());
                } else {
                    d.insert(
                        condition.variable.to_string(),
                        ftd_rt::Data {
                            value: data_value.to_string(),
                            dependencies: std::array::IntoIter::new([(
                                id,
                                condition.value.to_string(),
                            )])
                            .collect(),
                        },
                    );
                }
            }
        }
        d
    }

    pub fn get_locals(children: &[ftd_rt::Element]) -> ftd_rt::Map {
        let mut d: ftd_rt::Map = Default::default();
        for child in children {
            let locals = match child {
                ftd_rt::Element::Text(ftd_rt::Text {
                    common: ftd_rt::Common { locals, .. },
                    ..
                }) => locals.clone(),
                ftd_rt::Element::Image(ftd_rt::Image {
                    common: ftd_rt::Common { locals, .. },
                    ..
                }) => locals.clone(),
                ftd_rt::Element::Row(ftd_rt::Row {
                    common: ftd_rt::Common { locals, .. },
                    container,
                }) => {
                    let mut all_locals = Self::get_locals(&container.children);
                    for (k, v) in locals {
                        all_locals.insert(k.to_string(), v.to_string());
                    }
                    if let Some((_, _, ref c)) = container.external_children {
                        for (k, v) in Self::get_locals(c) {
                            all_locals.insert(k.to_string(), v.to_string());
                        }
                    }
                    all_locals
                }
                ftd_rt::Element::Column(ftd_rt::Column {
                    common: ftd_rt::Common { locals, .. },
                    container,
                }) => {
                    let mut all_locals = Self::get_locals(&container.children);
                    for (k, v) in locals {
                        all_locals.insert(k.to_string(), v.to_string());
                    }
                    if let Some((_, _, ref c)) = container.external_children {
                        for (k, v) in Self::get_locals(c) {
                            all_locals.insert(k.to_string(), v.to_string());
                        }
                    }
                    all_locals
                }
                ftd_rt::Element::IFrame(ftd_rt::IFrame {
                    common: ftd_rt::Common { locals, .. },
                    ..
                }) => locals.clone(),
                ftd_rt::Element::Input(ftd_rt::Input {
                    common: ftd_rt::Common { locals, .. },
                    ..
                }) => locals.clone(),
                ftd_rt::Element::Integer(ftd_rt::Text {
                    common: ftd_rt::Common { locals, .. },
                    ..
                }) => locals.clone(),
                ftd_rt::Element::Boolean(ftd_rt::Text {
                    common: ftd_rt::Common { locals, .. },
                    ..
                }) => locals.clone(),
                ftd_rt::Element::Decimal(ftd_rt::Text {
                    common: ftd_rt::Common { locals, .. },
                    ..
                }) => locals.clone(),
                ftd_rt::Element::Null => Default::default(),
            };

            for (k, v) in locals {
                d.insert(k.to_string(), v.to_string());
            }
        }
        d
    }

    pub fn is_open_container(&self) -> (bool, Option<String>) {
        match self {
            Self::Column(e) => e.container.is_open(),
            Self::Row(e) => e.container.is_open(),
            _ => (false, None),
        }
    }

    pub fn container_id(&self) -> Option<String> {
        match self {
            Self::Column(e) => e.common.id.clone(),
            Self::Row(e) => e.common.id.clone(),
            _ => None,
        }
    }

    pub fn set_container_id(&mut self, name: Option<String>) {
        match self {
            Self::Column(e) => e.common.id = name,
            Self::Row(e) => e.common.id = name,
            _ => {}
        }
    }

    pub fn set_condition(&mut self, condition: Option<ftd_rt::Condition>) {
        match self {
            Self::Column(ftd_rt::Column { common, .. }) => common,
            Self::Row(ftd_rt::Row { common, .. }) => common,
            Self::Text(ftd_rt::Text { common, .. }) => common,
            Self::Image(ftd_rt::Image { common, .. }) => common,
            Self::IFrame(ftd_rt::IFrame { common, .. }) => common,
            Self::Input(ftd_rt::Input { common, .. }) => common,
            Self::Integer(ftd_rt::Text { common, .. }) => common,
            Self::Boolean(ftd_rt::Text { common, .. }) => common,
            Self::Decimal(ftd_rt::Text { common, .. }) => common,
            Self::Null => return,
        }
        .condition = condition;
    }

    pub fn set_locals(&mut self, locals: ftd_rt::Map) {
        match self {
            Self::Column(ftd_rt::Column { common, .. }) => common,
            Self::Row(ftd_rt::Row { common, .. }) => common,
            Self::Text(ftd_rt::Text { common, .. }) => common,
            Self::Image(ftd_rt::Image { common, .. }) => common,
            Self::IFrame(ftd_rt::IFrame { common, .. }) => common,
            Self::Input(ftd_rt::Input { common, .. }) => common,
            Self::Integer(ftd_rt::Text { common, .. }) => common,
            Self::Boolean(ftd_rt::Text { common, .. }) => common,
            Self::Decimal(ftd_rt::Text { common, .. }) => common,
            Self::Null => return,
        }
        .locals = locals;
    }

    pub fn set_events(&mut self, events: &mut Vec<ftd_rt::Event>) {
        match self {
            Self::Column(ftd_rt::Column { common, .. }) => common,
            Self::Row(ftd_rt::Row { common, .. }) => common,
            Self::Text(ftd_rt::Text { common, .. }) => common,
            Self::Image(ftd_rt::Image { common, .. }) => common,
            Self::IFrame(ftd_rt::IFrame { common, .. }) => common,
            Self::Input(ftd_rt::Input { common, .. }) => common,
            Self::Integer(ftd_rt::Text { common, .. }) => common,
            Self::Boolean(ftd_rt::Text { common, .. }) => common,
            Self::Decimal(ftd_rt::Text { common, .. }) => common,
            Self::Null => return,
        }
        .events
        .append(events)
    }

    pub fn get_heading_region(&self) -> Option<&ftd_rt::Region> {
        match self {
            Self::Column(e) => e.common.region.as_ref().filter(|v| v.is_heading()),
            Self::Row(e) => e.common.region.as_ref().filter(|v| v.is_heading()),
            _ => None,
        }
    }
}

#[derive(serde::Deserialize, PartialEq)]
#[cfg_attr(not(feature = "wasm"), derive(Debug, Clone, serde::Serialize))]
#[serde(tag = "type")]
pub enum Length {
    Fill,
    Shrink,
    Auto,
    FitContent,
    Px { value: i64 },
    Portion { value: i64 },
    Percent { value: i64 },
    Calc { value: String },
}

impl Length {
    pub fn from(l: Option<String>) -> crate::Result<Option<Self>> {
        let l = match l {
            Some(l) => l,
            None => return Ok(None),
        };

        if l == "fill" {
            return Ok(Some(Length::Fill));
        }

        if l == "shrink" {
            return Ok(Some(Length::Shrink));
        }
        if l == "auto" {
            return Ok(Some(Length::Auto));
        }

        if l.starts_with("calc ") {
            let v = crate::get_name("calc", l.as_str())?;
            return match v.parse() {
                Ok(v) => Ok(Some(Length::Calc { value: v })),
                Err(_) => crate::e(format!("{} is not a valid integer", v)),
            };
        }

        if l == "fit-content" {
            return Ok(Some(Length::FitContent));
        }

        if l.starts_with("portion ") {
            let v = crate::get_name("portion", l.as_str())?;
            return match v.parse() {
                Ok(v) => Ok(Some(Length::Portion { value: v })),
                Err(_) => crate::e(format!("{} is not a valid integer", v)),
            };
        }
        if l.starts_with("percent ") {
            let v = crate::get_name("percent", l.as_str())?;
            return match v.parse() {
                Ok(v) => Ok(Some(Length::Percent { value: v })),
                Err(_) => crate::e(format!("{} is not a valid integer", v)),
            };
        }

        match l.parse() {
            Ok(v) => Ok(Some(Length::Px { value: v })),
            Err(_) => crate::e(format!("{} is not a valid integer", l)),
        }
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize)
)]
#[serde(tag = "type")]
pub enum Align {
    Center,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Default for Align {
    fn default() -> Self {
        Self::TopLeft
    }
}

impl Align {
    pub fn from(l: Option<String>) -> crate::Result<Self> {
        Ok(match l.as_deref() {
            Some("center") => Self::Center,
            Some("top") => Self::Top,
            Some("bottom") => Self::Bottom,
            Some("left") => Self::Left,
            Some("right") => Self::Right,
            Some("top-left") => Self::TopLeft,
            Some("top-right") => Self::TopRight,
            Some("bottom-left") => Self::BottomLeft,
            Some("bottom-right") => Self::BottomRight,
            Some(t) => return crate::e(format!("{} is not a valid alignment", t)),
            None => return Ok(Self::TopLeft),
        })
    }
}

// https://package.elm-lang.org/packages/mdgriffith/elm-ui/latest/Element-Region
#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize)
)]
pub enum Region {
    H0,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    H7,
    Title,
    MainContent,
    Navigation,
    Aside,
    Footer,
    Description,
    Announce,
    AnnounceUrgently,
}

impl ToString for Region {
    fn to_string(&self) -> String {
        match self {
            Self::H0 => "h0",
            Self::H1 => "h1",
            Self::H2 => "h2",
            Self::H3 => "h3",
            Self::H4 => "h4",
            Self::H5 => "h5",
            Self::H6 => "h6",
            Self::H7 => "h7",
            Self::Title => "title",
            Self::MainContent => "main",
            Self::Navigation => "navigation",
            Self::Aside => "aside",
            Self::Footer => "footer",
            Self::Description => "description",
            Self::Announce => "announce",
            Self::AnnounceUrgently => "announce-urgently",
        }
        .to_string()
    }
}

impl Region {
    pub fn from(l: Option<String>) -> crate::Result<Option<Self>> {
        Ok(Some(match l.as_deref() {
            Some("h0") => Self::H0,
            Some("h1") => Self::H1,
            Some("h2") => Self::H2,
            Some("h3") => Self::H3,
            Some("h4") => Self::H4,
            Some("h5") => Self::H5,
            Some("h6") => Self::H6,
            Some("h7") => Self::H7,
            Some("title") => Self::Title,
            Some("main") => Self::MainContent,
            Some("navigation") => Self::Navigation,
            Some("aside") => Self::Aside,
            Some("footer") => Self::Footer,
            Some("description") => Self::Description,
            Some("announce") => Self::Announce,
            Some("announce-urgently") => Self::AnnounceUrgently,
            Some(t) => return crate::e(format!("{} is not a valid alignment", t)),
            None => return Ok(None),
        }))
    }

    pub fn is_heading(&self) -> bool {
        matches!(
            self,
            Self::H0 | Self::H1 | Self::H2 | Self::H3 | Self::H4 | Self::H5 | Self::H6 | Self::H7
        )
    }

    pub fn is_primary_heading(&self) -> bool {
        matches!(self, Self::H0 | Self::H1)
    }

    pub fn is_title(&self) -> bool {
        matches!(self, Self::Title)
    }

    pub fn get_lower_priority_heading(&self) -> Vec<Self> {
        let mut list = vec![];
        if matches!(
            self,
            Self::Title
                | Self::MainContent
                | Self::Navigation
                | Self::Aside
                | Self::Footer
                | Self::Description
                | Self::Announce
                | Self::AnnounceUrgently
        ) {
            return list;
        }
        for region in [
            Self::H7,
            Self::H6,
            Self::H5,
            Self::H4,
            Self::H3,
            Self::H2,
            Self::H1,
        ] {
            if self.to_string() == region.to_string() {
                return list;
            }
            list.push(region);
        }
        list
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize)
)]
#[serde(tag = "type")]
pub enum Overflow {
    Hidden,
    Visible,
    Auto,
    Scroll,
}

impl Overflow {
    pub fn from(l: Option<String>) -> crate::Result<Option<Self>> {
        Ok(Option::from(match l.as_deref() {
            Some("hidden") => Self::Hidden,
            Some("visible") => Self::Visible,
            Some("auto") => Self::Auto,
            Some("scroll") => Self::Scroll,
            Some(t) => return crate::e(format!("{} is not a valid property", t)),
            None => return Ok(None),
        }))
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize, Default)
)]
pub struct Common {
    pub locals: ftd_rt::Map,
    pub condition: Option<ftd_rt::Condition>,
    pub events: Vec<ftd_rt::Event>,
    pub region: Option<Region>,
    pub padding: Option<i64>,
    pub padding_left: Option<i64>,
    pub padding_right: Option<i64>,
    pub padding_top: Option<i64>,
    pub padding_bottom: Option<i64>,
    pub border_top_radius: Option<i64>,
    pub border_bottom_radius: Option<i64>,
    pub border_left_radius: Option<i64>,
    pub border_right_radius: Option<i64>,
    pub width: Option<Length>,
    pub max_width: Option<Length>,
    pub min_width: Option<Length>,
    pub height: Option<Length>,
    pub min_height: Option<Length>,
    pub max_height: Option<Length>,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: i64,
    pub border_radius: i64,
    pub id: Option<String>,
    pub overflow_x: Option<Overflow>,
    pub overflow_y: Option<Overflow>,
    pub border_top: Option<i64>,
    pub border_left: Option<i64>,
    pub border_right: Option<i64>,
    pub border_bottom: Option<i64>,
    pub margin_top: Option<i64>,
    pub margin_left: Option<i64>,
    pub margin_right: Option<i64>,
    pub margin_bottom: Option<i64>,
    pub link: Option<String>,
    pub open_in_new_tab: bool,
    pub sticky: bool,
    pub top: Option<i64>,
    pub submit: Option<String>,
    pub cursor: Option<String>,
    // TODO: background-gradient
    // TODO: background-image, un-cropped, tiled, tiled{X, Y}
    // TODO: border-style: solid, dashed, dotted
    // TODO: border-{shadow, glow}
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize, Default)
)]
pub struct Container {
    pub children: Vec<ftd_rt::Element>,
    pub external_children: Option<(String, Vec<Vec<usize>>, Vec<ftd_rt::Element>)>,
    pub open: (Option<bool>, Option<String>),
    pub spacing: Option<i64>,
    pub align: Align,
    pub wrap: bool,
}

impl Container {
    pub fn is_open(&self) -> (bool, Option<String>) {
        (
            self.open.0.unwrap_or_else(|| self.children.is_empty()),
            self.open.1.clone(),
        )
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize, Default)
)]
pub struct Image {
    pub src: String,
    pub description: String,
    pub common: Common,
    pub align: Align,
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize, Default)
)]
pub struct Row {
    pub container: Container,
    pub common: Common,
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize, Default)
)]
pub struct Column {
    pub container: Container,
    pub common: Common,
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize)
)]
#[serde(tag = "type")]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
}

impl Default for TextAlign {
    fn default() -> Self {
        Self::Left
    }
}

impl TextAlign {
    pub fn from(l: Option<String>) -> crate::Result<Self> {
        Ok(match l.as_deref() {
            Some("center") => Self::Center,
            Some("left") => Self::Left,
            Some("right") => Self::Right,
            Some("justify") => Self::Justify,
            Some(t) => return crate::e(format!("{} is not a valid alignment", t)),
            None => return Ok(Self::Left),
        })
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize)
)]
#[serde(tag = "type")]
pub enum FontDisplay {
    Swap,
    Block,
}
impl Default for FontDisplay {
    fn default() -> Self {
        Self::Block
    }
}

impl FontDisplay {
    pub fn from(l: Option<String>) -> crate::Result<Self> {
        Ok(match l.as_deref() {
            Some("swap") => Self::Swap,
            Some("block") => Self::Block,
            Some(t) => return crate::e(format!("{} is not a valid alignment", t)),
            None => return Ok(Self::Block),
        })
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize)
)]
#[serde(tag = "type")]
pub enum NamedFont {
    Monospace,
    Serif,
    SansSerif,
    Named { value: String },
}

impl NamedFont {
    pub fn from(l: Option<String>) -> crate::Result<Self> {
        Ok(match l.as_deref() {
            Some("monospace") => Self::Monospace,
            Some("serif") => Self::Serif,
            Some("sansSerif") => Self::SansSerif,
            Some(t) => Self::Named {
                value: t.to_string(),
            },
            None => return Ok(Self::Serif),
        })
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize, Default)
)]
pub struct ExternalFont {
    pub url: String,
    pub name: String,
    pub display: FontDisplay,
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize)
)]
#[serde(tag = "type")]
pub enum Weight {
    Heavy,
    ExtraBold,
    Bold,
    SemiBold,
    Medium,
    Regular,
    Light,
    ExtraLight,
    HairLine,
}

impl Default for Weight {
    fn default() -> Self {
        Self::Regular
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize, Default)
)]
pub struct Style {
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub weight: Weight,
}

impl Style {
    pub fn from(l: Option<String>) -> crate::Result<Self> {
        let mut s = Style {
            italic: false,
            underline: false,
            strike: false,
            weight: Weight::default(),
        };
        let l = match l {
            Some(v) => v,
            None => return Ok(s),
        };
        // TODO: assert no word is repeated?
        for part in l.split_ascii_whitespace() {
            match part {
                "italic" => s.italic = true,
                "underline" => s.underline = true,
                "strike" => s.strike = true,
                "heavy" => s.weight = Weight::Heavy,
                "extra-bold" => s.weight = Weight::ExtraBold,
                "bold" => s.weight = Weight::Bold,
                "semi-bold" => s.weight = Weight::SemiBold,
                "medium" => s.weight = Weight::Medium,
                "regular" => s.weight = Weight::Regular,
                "light" => s.weight = Weight::Light,
                "extra-light" => s.weight = Weight::ExtraLight,
                "hairline" => s.weight = Weight::HairLine,
                t => return crate::e(format!("{} is not a valid style", t)),
            }
        }
        Ok(s)
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize)
)]
#[serde(tag = "type")]
pub enum TextFormat {
    // FTD, // TODO
    Markdown,
    Latex,
    Code { lang: String },
}

impl Default for TextFormat {
    fn default() -> Self {
        Self::Markdown
    }
}

impl TextFormat {
    pub fn from(l: Option<String>, lang: Option<String>) -> crate::Result<Self> {
        Ok(match l.as_deref() {
            Some("markdown") => Self::Markdown,
            Some("latex") => Self::Latex,
            Some("code") => Self::Code {
                lang: lang.unwrap_or_else(|| "txt".to_string()),
            },
            Some(t) => return crate::e(format!("{} is not a valid format", t)),
            None => return Ok(Self::Markdown),
        })
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize, Default)
)]
pub struct IFrame {
    pub src: String,
    pub common: Common,
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize, Default)
)]
pub struct Text {
    pub text: crate::Rendered,
    pub line: bool,
    pub common: Common,
    pub align: TextAlign,
    pub style: Style,
    pub format: TextFormat,
    pub size: Option<i64>,
    pub font: Vec<NamedFont>,
    pub external_font: Option<ExternalFont>,
    pub line_height: Option<String>,
    // TODO: line-height
    // TODO: region (https://package.elm-lang.org/packages/mdgriffith/elm-ui/latest/Element-Region)
    // TODO: family (maybe we need a type to represent font-family?)
    // TODO: letter-spacing
    // TODO: word-spacing
    // TODO: font-variants [small-caps, slashed-zero, feature/index etc]
    // TODO: shadow, glow
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize, Default)
)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub alpha: f32,
}

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(Debug, PartialEq, Clone, serde::Serialize, Default)
)]
pub struct Input {
    pub common: Common,
    pub placeholder: Option<String>,
}
