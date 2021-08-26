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
    pub fn is_open_container(&self) -> bool {
        match self {
            Self::Text(_) => false,
            Self::Input(_) => false,
            Self::Image(_) => false,
            Self::IFrame(_) => false,
            Self::Integer(_) => false,
            Self::Boolean(_) => false,
            Self::Decimal(_) => false,
            Self::Null => false,
            Self::Column(e) => e.container.is_open(),
            Self::Row(e) => e.container.is_open(),
        }
    }
    pub fn container_id(&self) -> Option<String> {
        match self {
            Self::Text(_) => None,
            Self::Input(_) => None,
            Self::Image(_) => None,
            Self::IFrame(_) => None,
            Self::Integer(_) => None,
            Self::Boolean(_) => None,
            Self::Decimal(_) => None,
            Self::Null => None,
            Self::Column(e) => e.common.id.clone(),
            Self::Row(e) => e.common.id.clone(),
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
    Px { value: i64 },
    Portion { value: i64 },
    Max { value: i64 },
    Min { value: i64 },
    Percent { value: i64 },
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

        if l.starts_with("maximum ") {
            let v = crate::get_name("maximum", l.as_str())?;
            return match v.parse() {
                Ok(v) => Ok(Some(Length::Max { value: v })),
                Err(_) => crate::e(format!("{} is not a valid integer", v)),
            };
        }

        if l.starts_with("minimum ") {
            let v = crate::get_name("minimum", l.as_str())?;
            return match v.parse() {
                Ok(v) => Ok(Some(Length::Min { value: v })),
                Err(_) => crate::e(format!("{} is not a valid integer", v)),
            };
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
    MainContent,
    Navigation,
    Aside,
    Footer,
    Description,
    Announce,
    AnnounceUrgently,
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
    pub condition: Option<ftd_rt::Condition>,
    pub region: Option<Region>,
    pub padding: Option<i64>,
    pub padding_left: Option<i64>,
    pub padding_right: Option<i64>,
    pub padding_top: Option<i64>,
    pub padding_bottom: Option<i64>,
    pub border_top_radius: Option<i64>,
    pub border_bottom_radius: Option<i64>,
    pub width: Option<Length>,
    pub height: Option<Length>,
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
    pub open: Option<bool>,
    pub spacing: Option<i64>,
    pub align: Align,
    pub wrap: bool,
}

impl Container {
    pub fn is_open(&self) -> bool {
        self.open.unwrap_or_else(|| self.children.is_empty())
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
