#[derive(serde::Deserialize, Clone)]
#[cfg_attr(not(feature = "wasm"), derive(Debug, PartialEq, serde::Serialize))]
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
    pub fn set_id(
        children: &mut [ftd_rt::Element],
        index_vec: &[usize],
        external_id: Option<String>,
    ) {
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
        children: &[ftd_rt::Element],
    ) -> ftd_rt::ExternalChildrenDependenciesMap {
        let mut d: ftd_rt::ExternalChildrenDependenciesMap = Default::default();
        for child in children {
            let container = match child {
                ftd_rt::Element::Row(ftd_rt::Row { container, .. }) => container,
                ftd_rt::Element::Column(ftd_rt::Column { container, .. }) => container,
                _ => continue,
            };
            let all_locals =
                ftd_rt::Element::get_external_children_dependencies(&container.children);
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
                    let all_locals = ftd_rt::Element::get_external_children_dependencies(
                        &col.container.children,
                    );
                    for (k, v) in all_locals {
                        d.insert(k.to_string(), v);
                    }
                }
            }
        }
        d
    }

    pub fn get_value_event_dependencies(
        children: &[ftd_rt::Element],
        data: &mut ftd_rt::DataDependenciesMap,
    ) {
        for child in children {
            let (reference, id) = match child {
                ftd_rt::Element::Column(ftd_rt::Column {
                    common: ftd_rt::Common { reference, id, .. },
                    container,
                })
                | ftd_rt::Element::Row(ftd_rt::Row {
                    common: ftd_rt::Common { reference, id, .. },
                    container,
                }) => {
                    ftd_rt::Element::get_value_event_dependencies(&container.children, data);
                    if let Some((_, _, external_children)) = &container.external_children {
                        ftd_rt::Element::get_value_event_dependencies(external_children, data);
                    }
                    (reference, id)
                }
                ftd_rt::Element::Image(ftd_rt::Image { common, .. })
                | ftd_rt::Element::Text(ftd_rt::Text { common, .. })
                | ftd_rt::Element::IFrame(ftd_rt::IFrame { common, .. })
                | ftd_rt::Element::Input(ftd_rt::Input { common, .. })
                | ftd_rt::Element::Integer(ftd_rt::Text { common, .. })
                | ftd_rt::Element::Boolean(ftd_rt::Text { common, .. })
                | ftd_rt::Element::Decimal(ftd_rt::Text { common, .. }) => {
                    (&common.reference, &common.id)
                }
                ftd_rt::Element::Null => continue,
            };
            if let Some(reference) = reference {
                let id = id.clone().expect("universal id should be present");

                if let Some(ftd_rt::Data { dependencies, .. }) = data.get_mut(reference) {
                    dependencies.insert(id, "value".to_string());
                }
            }
        }
    }

    pub fn get_visible_event_dependencies(
        children: &[ftd_rt::Element],
        data: &mut ftd_rt::DataDependenciesMap,
    ) {
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
                    ftd_rt::Element::get_visible_event_dependencies(&container.children, data);
                    if let Some((_, _, external_children)) = &container.external_children {
                        ftd_rt::Element::get_visible_event_dependencies(external_children, data);
                    }
                    (condition, id)
                }
                ftd_rt::Element::Image(ftd_rt::Image { common, .. })
                | ftd_rt::Element::Text(ftd_rt::Text { common, .. })
                | ftd_rt::Element::IFrame(ftd_rt::IFrame { common, .. })
                | ftd_rt::Element::Input(ftd_rt::Input { common, .. })
                | ftd_rt::Element::Integer(ftd_rt::Text { common, .. })
                | ftd_rt::Element::Boolean(ftd_rt::Text { common, .. })
                | ftd_rt::Element::Decimal(ftd_rt::Text { common, .. }) => {
                    (&common.condition, &common.id)
                }
                ftd_rt::Element::Null => continue,
            };
            if let Some(condition) = condition {
                let id = id.clone().expect("universal id should be present");

                if let Some(ftd_rt::Data { dependencies, .. }) = data.get_mut(&condition.variable) {
                    dependencies.insert(id, condition.value.to_string());
                } else {
                    panic!("{} should be declared", condition.variable)
                }
            }
        }
    }

    pub fn get_locals(children: &[ftd_rt::Element]) -> ftd_rt::Map {
        let mut d: ftd_rt::Map = Default::default();
        for child in children {
            let locals = match child {
                ftd_rt::Element::Row(ftd_rt::Row {
                    common: ftd_rt::Common { locals, .. },
                    container,
                })
                | ftd_rt::Element::Column(ftd_rt::Column {
                    common: ftd_rt::Common { locals, .. },
                    container,
                }) => {
                    let mut all_locals = ftd_rt::Element::get_locals(&container.children);
                    for (k, v) in locals {
                        all_locals.insert(k.to_string(), v.to_string());
                    }
                    if let Some((_, _, ref c)) = container.external_children {
                        for (k, v) in ftd_rt::Element::get_locals(c) {
                            all_locals.insert(k.to_string(), v.to_string());
                        }
                    }
                    all_locals
                }
                ftd_rt::Element::Decimal(ftd_rt::Text { common, .. })
                | ftd_rt::Element::Text(ftd_rt::Text { common, .. })
                | ftd_rt::Element::Image(ftd_rt::Image { common, .. })
                | ftd_rt::Element::IFrame(ftd_rt::IFrame { common, .. })
                | ftd_rt::Element::Input(ftd_rt::Input { common, .. })
                | ftd_rt::Element::Integer(ftd_rt::Text { common, .. })
                | ftd_rt::Element::Boolean(ftd_rt::Text { common, .. }) => common.locals.clone(),
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
            ftd_rt::Element::Column(e) => e.container.is_open(),
            ftd_rt::Element::Row(e) => e.container.is_open(),
            _ => (false, None),
        }
    }

    pub fn container_id(&self) -> Option<String> {
        match self {
            ftd_rt::Element::Column(e) => e.common.id.clone(),
            ftd_rt::Element::Row(e) => e.common.id.clone(),
            _ => None,
        }
    }

    pub fn set_container_id(&mut self, name: Option<String>) {
        match self {
            ftd_rt::Element::Column(e) => e.common.id = name,
            ftd_rt::Element::Row(e) => e.common.id = name,
            _ => {}
        }
    }

    pub fn set_condition(&mut self, condition: Option<ftd_rt::Condition>) {
        match self {
            ftd_rt::Element::Column(ftd_rt::Column { common, .. }) => common,
            ftd_rt::Element::Row(ftd_rt::Row { common, .. }) => common,
            ftd_rt::Element::Text(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Image(ftd_rt::Image { common, .. }) => common,
            ftd_rt::Element::IFrame(ftd_rt::IFrame { common, .. }) => common,
            ftd_rt::Element::Input(ftd_rt::Input { common, .. }) => common,
            ftd_rt::Element::Integer(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Boolean(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Decimal(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Null => return,
        }
        .condition = condition;
    }

    pub fn set_non_visibility(&mut self, is_not_visible: bool) {
        match self {
            ftd_rt::Element::Column(ftd_rt::Column { common, .. }) => common,
            ftd_rt::Element::Row(ftd_rt::Row { common, .. }) => common,
            ftd_rt::Element::Text(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Image(ftd_rt::Image { common, .. }) => common,
            ftd_rt::Element::IFrame(ftd_rt::IFrame { common, .. }) => common,
            ftd_rt::Element::Input(ftd_rt::Input { common, .. }) => common,
            ftd_rt::Element::Integer(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Boolean(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Decimal(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Null => return,
        }
        .is_not_visible = is_not_visible;
    }

    pub fn set_locals(&mut self, locals: ftd_rt::Map) {
        match self {
            ftd_rt::Element::Column(ftd_rt::Column { common, .. }) => common,
            ftd_rt::Element::Row(ftd_rt::Row { common, .. }) => common,
            ftd_rt::Element::Text(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Image(ftd_rt::Image { common, .. }) => common,
            ftd_rt::Element::IFrame(ftd_rt::IFrame { common, .. }) => common,
            ftd_rt::Element::Input(ftd_rt::Input { common, .. }) => common,
            ftd_rt::Element::Integer(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Boolean(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Decimal(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Null => return,
        }
        .locals = locals;
    }

    pub fn set_events(&mut self, events: &mut Vec<ftd_rt::Event>) {
        match self {
            ftd_rt::Element::Column(ftd_rt::Column { common, .. }) => common,
            ftd_rt::Element::Row(ftd_rt::Row { common, .. }) => common,
            ftd_rt::Element::Text(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Image(ftd_rt::Image { common, .. }) => common,
            ftd_rt::Element::IFrame(ftd_rt::IFrame { common, .. }) => common,
            ftd_rt::Element::Input(ftd_rt::Input { common, .. }) => common,
            ftd_rt::Element::Integer(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Boolean(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Decimal(ftd_rt::Text { common, .. }) => common,
            ftd_rt::Element::Null => return,
        }
        .events
        .append(events)
    }

    pub fn get_heading_region(&self) -> Option<&ftd_rt::Region> {
        match self {
            ftd_rt::Element::Column(e) => e.common.region.as_ref().filter(|v| v.is_heading()),
            ftd_rt::Element::Row(e) => e.common.region.as_ref().filter(|v| v.is_heading()),
            _ => None,
        }
    }

    pub fn renesting_on_region(elements: &mut Vec<ftd_rt::Element>) {
        let mut region: Option<(usize, &Region)> = None;
        let mut insert: Vec<(usize, usize)> = Default::default();
        for (idx, element) in elements.iter().enumerate() {
            match element {
                ftd_rt::Element::Column(ftd_rt::Column { common, .. })
                | ftd_rt::Element::Row(ftd_rt::Row { common, .. }) => {
                    let r = common.region.as_ref().filter(|v| v.is_heading());
                    if let Some(r) = r {
                        if let Some((place_at, r1)) = region {
                            if r.get_lower_priority_heading().contains(r1) || r == r1 {
                                insert.push((place_at, idx));
                                region = Some((idx, r));
                            }
                        } else {
                            region = Some((idx, r));
                        }
                    }
                }
                _ => continue,
            }
        }
        if let Some((place_at, _)) = region {
            insert.push((place_at, elements.len()));
        }

        for (place_at, end) in insert.iter().rev() {
            let mut children = elements[place_at + 1..*end].to_vec();
            match elements[*place_at] {
                ftd_rt::Element::Column(ftd_rt::Column {
                    ref mut container, ..
                })
                | ftd_rt::Element::Row(ftd_rt::Row {
                    ref mut container, ..
                }) => {
                    container.children.append(&mut children);
                }
                _ => continue,
            }
            for idx in (place_at + 1..*end).rev() {
                elements.remove(idx);
            }
        }

        for element in &mut *elements {
            match element {
                ftd_rt::Element::Column(ftd_rt::Column {
                    ref mut container, ..
                })
                | ftd_rt::Element::Row(ftd_rt::Row {
                    ref mut container, ..
                }) => {
                    if let Some((_, _, ref mut e)) = container.external_children {
                        ftd_rt::Element::renesting_on_region(e);
                    }
                    ftd_rt::Element::renesting_on_region(&mut container.children);
                }
                _ => continue,
            }
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
    pub fn from(l: Option<String>) -> ftd_rt::Result<Option<ftd_rt::Length>> {
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
    fn default() -> ftd_rt::Align {
        Self::TopLeft
    }
}

impl Align {
    pub fn from(l: Option<String>) -> ftd_rt::Result<ftd_rt::Align> {
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
    pub fn from(l: Option<String>) -> ftd_rt::Result<Option<ftd_rt::Region>> {
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
            ftd_rt::Region::H0
                | ftd_rt::Region::H1
                | ftd_rt::Region::H2
                | ftd_rt::Region::H3
                | ftd_rt::Region::H4
                | ftd_rt::Region::H5
                | ftd_rt::Region::H6
                | ftd_rt::Region::H7
        )
    }

    pub fn is_primary_heading(&self) -> bool {
        matches!(self, ftd_rt::Region::H0 | ftd_rt::Region::H1)
    }

    pub fn is_title(&self) -> bool {
        matches!(self, ftd_rt::Region::Title)
    }

    pub fn get_lower_priority_heading(&self) -> Vec<ftd_rt::Region> {
        let mut list = vec![];
        if matches!(
            self,
            ftd_rt::Region::Title
                | ftd_rt::Region::MainContent
                | ftd_rt::Region::Navigation
                | ftd_rt::Region::Aside
                | ftd_rt::Region::Footer
                | ftd_rt::Region::Description
                | ftd_rt::Region::Announce
                | ftd_rt::Region::AnnounceUrgently
        ) {
            return list;
        }
        for region in [
            ftd_rt::Region::H7,
            ftd_rt::Region::H6,
            ftd_rt::Region::H5,
            ftd_rt::Region::H4,
            ftd_rt::Region::H3,
            ftd_rt::Region::H2,
            ftd_rt::Region::H1,
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
    pub fn from(l: Option<String>) -> ftd_rt::Result<Option<ftd_rt::Overflow>> {
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
    derive(Debug, PartialEq, Clone, serde::Serialize)
)]
#[serde(tag = "type")]
pub enum GradientDirection {
    BottomToTop,
    TopToBottom,
    RightToLeft,
    LeftToRight,
    BottomRightToTopLeft,
    BottomLeftToTopRight,
    TopRightToBottomLeft,
    TopLeftBottomRight,
    Center,
    Angle { value: i64 },
}

impl GradientDirection {
    pub fn from(l: Option<String>) -> ftd_rt::Result<Option<ftd_rt::GradientDirection>> {
        let l = match l {
            Some(l) => l,
            None => return Ok(None),
        };

        if l == "bottom to top" {
            return Ok(Some(GradientDirection::BottomToTop));
        }
        if l == "top to bottom" {
            return Ok(Some(GradientDirection::TopToBottom));
        }
        if l == "right to left" {
            return Ok(Some(GradientDirection::RightToLeft));
        }
        if l == "left to right" {
            return Ok(Some(GradientDirection::LeftToRight));
        }
        if l == "bottom-left to top-right" {
            return Ok(Some(GradientDirection::BottomLeftToTopRight));
        }
        if l == "bottom-right to top-left" {
            return Ok(Some(GradientDirection::BottomRightToTopLeft));
        }
        if l == "top-right to bottom-left" {
            return Ok(Some(GradientDirection::TopRightToBottomLeft));
        }
        if l == "top-left to bottom-right" {
            return Ok(Some(GradientDirection::TopLeftBottomRight));
        }
        if l == "center" {
            return Ok(Some(GradientDirection::Center));
        }
        if l.starts_with("angle ") {
            let v = crate::get_name("angle", l.as_str())?;
            return match v.parse() {
                Ok(v) => Ok(Some(GradientDirection::Angle { value: v })),
                Err(_) => crate::e(format!("{} is not a valid integer", v)),
            };
        }
        Ok(None)
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
    pub is_not_visible: bool,
    pub events: Vec<ftd_rt::Event>,
    pub reference: Option<String>,
    pub region: Option<Region>,
    pub padding: Option<i64>,
    pub padding_vertical: Option<i64>,
    pub padding_horizontal: Option<i64>,
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
    pub shadow_offset_x: Option<i64>,
    pub shadow_offset_y: Option<i64>,
    pub shadow_size: Option<i64>,
    pub shadow_blur: Option<i64>,
    pub shadow_color: Option<Color>,
    pub gradient_direction: Option<GradientDirection>,
    pub gradient_colors: Vec<Color>,
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
        ftd_rt::TextAlign::Left
    }
}

impl TextAlign {
    pub fn from(l: Option<String>) -> ftd_rt::Result<ftd_rt::TextAlign> {
        Ok(match l.as_deref() {
            Some("center") => ftd_rt::TextAlign::Center,
            Some("left") => ftd_rt::TextAlign::Left,
            Some("right") => ftd_rt::TextAlign::Right,
            Some("justify") => ftd_rt::TextAlign::Justify,
            Some(t) => return crate::e(format!("{} is not a valid alignment", t)),
            None => return Ok(ftd_rt::TextAlign::Left),
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
impl Default for ftd_rt::FontDisplay {
    fn default() -> Self {
        ftd_rt::FontDisplay::Block
    }
}

impl FontDisplay {
    pub fn from(l: Option<String>) -> ftd_rt::Result<ftd_rt::FontDisplay> {
        Ok(match l.as_deref() {
            Some("swap") => ftd_rt::FontDisplay::Swap,
            Some("block") => ftd_rt::FontDisplay::Block,
            Some(t) => return crate::e(format!("{} is not a valid alignment", t)),
            None => return Ok(ftd_rt::FontDisplay::Block),
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
    pub fn from(l: Option<String>) -> ftd_rt::Result<ftd_rt::NamedFont> {
        Ok(match l.as_deref() {
            Some("monospace") => ftd_rt::NamedFont::Monospace,
            Some("serif") => ftd_rt::NamedFont::Serif,
            Some("sansSerif") => ftd_rt::NamedFont::SansSerif,
            Some(t) => ftd_rt::NamedFont::Named {
                value: t.to_string(),
            },
            None => return Ok(ftd_rt::NamedFont::Serif),
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
        ftd_rt::Weight::Regular
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
    pub weight: ftd_rt::Weight,
}

impl Style {
    pub fn from(l: Option<String>) -> ftd_rt::Result<ftd_rt::Style> {
        let mut s = Style {
            italic: false,
            underline: false,
            strike: false,
            weight: Default::default(),
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
                "heavy" => s.weight = ftd_rt::Weight::Heavy,
                "extra-bold" => s.weight = ftd_rt::Weight::ExtraBold,
                "bold" => s.weight = ftd_rt::Weight::Bold,
                "semi-bold" => s.weight = ftd_rt::Weight::SemiBold,
                "medium" => s.weight = ftd_rt::Weight::Medium,
                "regular" => s.weight = ftd_rt::Weight::Regular,
                "light" => s.weight = ftd_rt::Weight::Light,
                "extra-light" => s.weight = ftd_rt::Weight::ExtraLight,
                "hairline" => s.weight = ftd_rt::Weight::HairLine,
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
    Text,
}

impl Default for ftd_rt::TextFormat {
    fn default() -> ftd_rt::TextFormat {
        ftd_rt::TextFormat::Markdown
    }
}

impl TextFormat {
    pub fn from(l: Option<String>, lang: Option<String>) -> ftd_rt::Result<ftd_rt::TextFormat> {
        Ok(match l.as_deref() {
            Some("markdown") => ftd_rt::TextFormat::Markdown,
            Some("latex") => ftd_rt::TextFormat::Latex,
            Some("code") => ftd_rt::TextFormat::Code {
                lang: lang.unwrap_or_else(|| "txt".to_string()),
            },
            Some("text") => ftd_rt::TextFormat::Text,
            Some(t) => return ftd_rt::e(format!("{} is not a valid format", t)),
            None => return Ok(ftd_rt::TextFormat::Markdown),
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
    pub text: ftd_rt::Rendered,
    pub line: bool,
    pub common: Common,
    pub align: TextAlign,
    pub style: Style,
    pub format: TextFormat,
    pub size: Option<i64>,
    pub font: Vec<NamedFont>,
    pub external_font: Option<ExternalFont>,
    pub line_height: Option<i64>,
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
