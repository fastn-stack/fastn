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
    pub width: Option<Length>,
    pub max_width: Option<Length>,
    pub min_width: Option<Length>,
    pub height: Option<Length>,
    pub min_height: Option<Length>,
    pub max_height: Option<Length>,
    pub explain: bool,
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
