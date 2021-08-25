#[derive(PartialEq, Debug, Clone, serde_derive::Serialize, Default)]
pub struct Meta {
    pub surfers: Vec<Surfer>,
    pub readers: Vec<Reader>,
    pub writers: Vec<Writer>,
    pub admins: Vec<Admin>,
    pub cr: Option<CR>,
    pub design: Option<Design>,
    pub minimum_cr_approvals: i32,
    pub no_index: bool,
    pub(crate) lang: crate::ValueWithDefault<realm_lang::Language>,
    translation: Translation,
    pub git_repo: Option<String>,
    pub deleted: bool,
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub enum Translation {
    NoTranslation,
    ItIsTranslationOf { id: String },
    ItHasTranslations { translations: Vec<String> },
}

impl Default for Translation {
    fn default() -> Self {
        Self::NoTranslation
    }
}

impl Translation {
    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, crate::document::ParseError> {
        let mut of: Option<String> = None;
        let mut translations = vec![];

        for (k, v) in p1.header.0.iter() {
            if k == "translation-of" {
                if of.is_some() {
                    return Err(crate::document::ParseError::ValidationError(
                        "translation-of: specified more than once".to_string(),
                    ));
                }
                of = Some(v.to_string())
            } else if k == "translation" {
                translations.push(v.to_string())
            }
        }

        match of {
            Some(v) => {
                if translations.is_empty() {
                    Ok(Translation::ItIsTranslationOf { id: v })
                } else {
                    Err(crate::document::ParseError::ValidationError(
                        "both translation-of: and translation: headers can't be specified"
                            .to_string(),
                    ))
                }
            }
            None => {
                if translations.is_empty() {
                    Ok(Translation::NoTranslation)
                } else {
                    Ok(Translation::ItHasTranslations { translations })
                }
            }
        }
    }

    pub fn to_p1(&self, p1: &mut crate::p1::Section) {
        match self {
            Self::NoTranslation => {}
            Self::ItHasTranslations { translations } => {
                for t in translations.iter() {
                    p1.header.0.push(("translation".to_string(), t.clone()))
                }
            }
            Self::ItIsTranslationOf { id } => {
                p1.header.0.push(("translation-of".to_string(), id.clone()))
            }
        }
    }

    pub fn is_translation_of(&self) -> bool {
        matches!(self, Self::ItIsTranslationOf { .. })
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub struct WidgetColors {
    background: Color,
    text: Color,
    text_primary: Color,
    separator: Color,
    hover: Color,
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub struct ColumnColors {
    background: Color,
    text: Color,
    active_background: Color,
    active_text: Color,
    separator: Color,
    secondary: Color,
    hover_text: Color,
    hover_secondary: Color,
    widget: WidgetColors,
    heading: Color,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Theme {
    Darkula,
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub struct ThemeConfig {
    brand: Color,
    accent: Color,
    link: Color,
    success: Color,
    danger: Color,
    informational: Color,
    warning: Color,
    background: Color,
    separator: Color,
    first_header: ColumnColors,
    second_header: ColumnColors,
    left_sidebar: ColumnColors,
    right_sidebar: ColumnColors,
    body: ColumnColors,
}

trait ThemeMode {
    fn light_mode(&self) -> ThemeConfig;
    fn dark_mode(&self) -> ThemeConfig;
}

impl ThemeMode for Theme {
    fn light_mode(&self) -> ThemeConfig {
        match self {
            Theme::Darkula => ThemeConfig {
                brand: "#ffffff".parse().unwrap(),
                link: "#0047b4".parse().unwrap(),
                accent: "#D37455".parse().unwrap(),
                success: "#1A936F".parse().unwrap(),
                danger: "#e87f85".parse().unwrap(),
                informational: "#1B51A8".parse().unwrap(),
                warning: "#f5c94f".parse().unwrap(),
                background: "#ffffff".parse().unwrap(),
                separator: "#f5f5f5".parse().unwrap(),
                first_header: ColumnColors {
                    background: "#1b2e47".parse().unwrap(),
                    text: "#F5F5F5".parse().unwrap(),
                    active_background: "#1b2e47".parse().unwrap(),
                    active_text: "#F5F5F5".parse().unwrap(),
                    separator: "#eaecef".parse().unwrap(),
                    secondary: "#F5F5F5".parse().unwrap(),
                    hover_text: "#000000".parse().unwrap(),
                    hover_secondary: "#fbfbff".parse().unwrap(),
                    widget: WidgetColors {
                        background: "#F5F5F5".parse().unwrap(),
                        text: "#282828".parse().unwrap(),
                        text_primary: "#000000".parse().unwrap(),
                        separator: "#eaecef".parse().unwrap(),
                        hover: "#fbfbff".parse().unwrap(),
                    },
                    heading: "#FFFFFF".parse().unwrap(),
                },
                second_header: ColumnColors {
                    background: "#6a7c95".parse().unwrap(),
                    text: "#F5F5F5".parse().unwrap(),
                    active_background: "#1b2e47".parse().unwrap(),
                    active_text: "#F5F5F5".parse().unwrap(),
                    separator: "#eaecef".parse().unwrap(),
                    secondary: "#F5F5F5".parse().unwrap(),
                    hover_text: "#000000".parse().unwrap(),
                    hover_secondary: "#fbfbff".parse().unwrap(),
                    widget: WidgetColors {
                        background: "#F5F5F5".parse().unwrap(),
                        text: "#282828".parse().unwrap(),
                        text_primary: "#000000".parse().unwrap(),
                        separator: "#eaecef".parse().unwrap(),
                        hover: "#fbfbff".parse().unwrap(),
                    },
                    heading: "#FFFFFF".parse().unwrap(),
                },
                left_sidebar: ColumnColors {
                    background: "#FFFFFF".parse().unwrap(),
                    text: "#4D4D4D".parse().unwrap(),
                    active_background: "#6a7c95".parse().unwrap(),
                    active_text: "#FFFFFF".parse().unwrap(),
                    separator: "#eaecef".parse().unwrap(),
                    secondary: "#F5F5F5".parse().unwrap(),
                    hover_text: "#000000".parse().unwrap(),
                    hover_secondary: "#fbfbff".parse().unwrap(),
                    widget: WidgetColors {
                        background: "#FFFFFF".parse().unwrap(),
                        text: "#282828".parse().unwrap(),
                        text_primary: "#000000".parse().unwrap(),
                        separator: "#eaecef".parse().unwrap(),
                        hover: "#F5F5F5".parse().unwrap(),
                    },
                    heading: "#000000".parse().unwrap(),
                },
                right_sidebar: ColumnColors {
                    background: "#FFFFFF".parse().unwrap(),
                    text: "#4D4D4D".parse().unwrap(),
                    active_background: "#1b2E47".parse().unwrap(),
                    active_text: "#FFFFFF".parse().unwrap(),
                    separator: "#eaecef".parse().unwrap(),
                    secondary: "#F5F5F5".parse().unwrap(),
                    hover_text: "#000000".parse().unwrap(),
                    hover_secondary: "#fbfbff".parse().unwrap(),
                    widget: WidgetColors {
                        background: "#FFFFFF".parse().unwrap(),
                        text: "#282828".parse().unwrap(),
                        text_primary: "#000000".parse().unwrap(),
                        separator: "#eaecef".parse().unwrap(),
                        hover: "#F5F5F5".parse().unwrap(),
                    },
                    heading: "#000000".parse().unwrap(),
                },
                body: ColumnColors {
                    background: "#FFFFFF".parse().unwrap(),
                    text: "#4D4D4D".parse().unwrap(),
                    active_background: "#1b2E47".parse().unwrap(),
                    active_text: "#FFFFFF".parse().unwrap(),
                    separator: "#eaecef".parse().unwrap(),
                    secondary: "#F5F5F5".parse().unwrap(),
                    hover_text: "#000000".parse().unwrap(),
                    hover_secondary: "#fbfbff".parse().unwrap(),
                    widget: WidgetColors {
                        background: "#FFFFFF".parse().unwrap(),
                        text: "#282828".parse().unwrap(),
                        text_primary: "#000000".parse().unwrap(),
                        separator: "#eaecef".parse().unwrap(),
                        hover: "#F5F5F5".parse().unwrap(),
                    },
                    heading: "#000000".parse().unwrap(),
                },
            },
        }
    }

    fn dark_mode(&self) -> ThemeConfig {
        match self {
            Theme::Darkula => ThemeConfig {
                brand: "#FFFFFF".parse().unwrap(),
                link: "#6c8fef".parse().unwrap(),
                accent: "#D37455".parse().unwrap(),
                success: "#3b7a5c".parse().unwrap(),
                danger: "#e87f85".parse().unwrap(),
                informational: "#5e7cec".parse().unwrap(),
                warning: "#f5c94f".parse().unwrap(),
                background: "#1a1f35".parse().unwrap(),
                separator: "#3c4257".parse().unwrap(),
                first_header: ColumnColors {
                    background: "#1a1f35".parse().unwrap(),
                    text: "#6c8fef".parse().unwrap(),
                    active_background: "#3c4257".parse().unwrap(),
                    active_text: "#FFFFFF".parse().unwrap(),
                    separator: "#3c4257".parse().unwrap(),
                    secondary: "#2a2f45".parse().unwrap(),
                    hover_text: "#FFFFFF".parse().unwrap(),
                    hover_secondary: "#2a2f45".parse().unwrap(),
                    widget: WidgetColors {
                        background: "#2a2f45".parse().unwrap(),
                        text: "#c1c9d2".parse().unwrap(),
                        text_primary: "#FFFFFF".parse().unwrap(),
                        separator: "#3c4257".parse().unwrap(),
                        hover: "#2a2f45".parse().unwrap(),
                    },
                    heading: "#FFFFFF".parse().unwrap(),
                },
                second_header: ColumnColors {
                    background: "#1a1f35".parse().unwrap(),
                    text: "#6c8fef".parse().unwrap(),
                    active_background: "#3c4257".parse().unwrap(),
                    active_text: "#FFFFFF".parse().unwrap(),
                    separator: "#3c4257".parse().unwrap(),
                    secondary: "#2a2f45".parse().unwrap(),
                    hover_text: "#FFFFFF".parse().unwrap(),
                    hover_secondary: "#2a2f45".parse().unwrap(),
                    widget: WidgetColors {
                        background: "#2a2f45".parse().unwrap(),
                        text: "#c1c9d2".parse().unwrap(),
                        text_primary: "#FFFFFF".parse().unwrap(),
                        separator: "#3c4257".parse().unwrap(),
                        hover: "#2a2f45".parse().unwrap(),
                    },
                    heading: "#FFFFFF".parse().unwrap(),
                },
                left_sidebar: ColumnColors {
                    background: "#1a1f35".parse().unwrap(),
                    text: "#c1c9d2".parse().unwrap(),
                    active_background: "#3c4257".parse().unwrap(),
                    active_text: "#FFFFFF".parse().unwrap(),
                    separator: "#3c4257".parse().unwrap(),
                    secondary: "#2a2f45".parse().unwrap(),
                    hover_text: "#FFFFFF".parse().unwrap(),
                    hover_secondary: "#161a2e".parse().unwrap(),
                    widget: WidgetColors {
                        background: "#2a2f45".parse().unwrap(),
                        text: "#c1c9d2".parse().unwrap(),
                        text_primary: "#FFFFFF".parse().unwrap(),
                        separator: "#3c4257".parse().unwrap(),
                        hover: "#161a2e".parse().unwrap(),
                    },
                    heading: "#FFFFFF".parse().unwrap(),
                },
                right_sidebar: ColumnColors {
                    background: "#1a1f35".parse().unwrap(),
                    text: "#c1c9d2".parse().unwrap(),
                    active_background: "#3c4257".parse().unwrap(),
                    active_text: "#FFFFFF".parse().unwrap(),
                    separator: "#3c4257".parse().unwrap(),
                    secondary: "#2a2f45".parse().unwrap(),
                    hover_text: "#FFFFFF".parse().unwrap(),
                    hover_secondary: "#161a2e".parse().unwrap(),
                    widget: WidgetColors {
                        background: "#2a2f45".parse().unwrap(),
                        text: "#c1c9d2".parse().unwrap(),
                        text_primary: "#FFFFFF".parse().unwrap(),
                        separator: "#3c4257".parse().unwrap(),
                        hover: "#161a2e".parse().unwrap(),
                    },
                    heading: "#FFFFFF".parse().unwrap(),
                },
                body: ColumnColors {
                    background: "#1a1f35".parse().unwrap(),
                    text: "#c1c9d2".parse().unwrap(),
                    active_background: "#3c4257".parse().unwrap(),
                    active_text: "#FFFFFF".parse().unwrap(),
                    separator: "#3c4257".parse().unwrap(),
                    secondary: "#2a2f45".parse().unwrap(),
                    hover_text: "#FFFFFF".parse().unwrap(),
                    hover_secondary: "#161a2e".parse().unwrap(),
                    widget: WidgetColors {
                        background: "#2a2f45".parse().unwrap(),
                        text: "#c1c9d2".parse().unwrap(),
                        text_primary: "#FFFFFF".parse().unwrap(),
                        separator: "#3c4257".parse().unwrap(),
                        hover: "#161a2e".parse().unwrap(),
                    },
                    heading: "#FFFFFF".parse().unwrap(),
                },
            },
        }
    }
}

impl std::str::FromStr for Theme {
    type Err = crate::document::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "darkula" => Theme::Darkula,
            _ => crate::document::err(format!("unknown theme: {}", s).as_str())?,
        })
    }
}

impl Theme {
    pub fn to_string(&self) -> &'static str {
        match self {
            Theme::Darkula => "darkula",
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Color {
    color: css_color_parser::Color,
    original: String,
}

impl serde::ser::Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Color", 4)?;
        state.serialize_field("r", &self.color.r)?;
        state.serialize_field("g", &self.color.g)?;
        state.serialize_field("b", &self.color.b)?;
        state.serialize_field("a", &self.color.a)?;
        state.end()
    }
}

impl std::str::FromStr for Color {
    type Err = crate::document::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Color {
            color: s.parse()?,
            original: s.to_string(),
        })
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub enum Layout {
    OneColumn,
    TwoColumn,
    ThreeColumn,
    TwoColumnNoHeader,
}

impl std::str::FromStr for Layout {
    type Err = crate::document::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "one_column" => Layout::OneColumn,
            "two_column" => Layout::TwoColumn,
            "three_column" => Layout::ThreeColumn,
            "two_column_no_header" => Layout::TwoColumnNoHeader,
            _ => crate::document::err(format!("unknown layout: {}", s).as_str())?,
        })
    }
}

impl Layout {
    pub fn to_string(&self) -> &'static str {
        match self {
            Layout::OneColumn => "one_column",
            Layout::TwoColumn => "two_column",
            Layout::ThreeColumn => "three_column",
            Layout::TwoColumnNoHeader => "two_column_no_header",
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Design {
    theme: Theme,
    layout: Layout,
    header_fixed: bool,
    left_sidebar_fixed: bool,
    right_sidebar_fixed: bool,
    logo: String,
}

impl Default for Design {
    fn default() -> Self {
        Design {
            theme: Theme::Darkula,
            layout: Layout::ThreeColumn,
            header_fixed: false,
            left_sidebar_fixed: true,
            right_sidebar_fixed: true,
            logo: LOGO.to_string(),
        }
    }
}

impl serde::ser::Serialize for Design {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Design", 5)?;
        state.serialize_field("light_mode", &self.theme.light_mode())?;
        state.serialize_field("dark_mode", &self.theme.dark_mode())?;
        state.serialize_field("layout", &self.layout)?;
        state.serialize_field("header_fixed", &self.header_fixed)?;
        state.serialize_field("left_sidebar_fixed", &self.left_sidebar_fixed)?;
        state.serialize_field("right_sidebar_fixed", &self.right_sidebar_fixed)?;
        state.serialize_field("logo", &self.logo)?;
        state.end()
    }
}

impl Design {
    fn from_p1(p1: &crate::p1::SubSection) -> Result<Self, crate::document::ParseError> {
        Ok(Design {
            theme: p1.header.string_with_default("theme", "darkula")?.parse()?,
            layout: p1
                .header
                .str_with_default("layout", "three_column")?
                .parse()?,
            header_fixed: p1.header.bool_with_default("header-fixed", false)?,
            left_sidebar_fixed: p1.header.bool_with_default("left-sidebar-fixed", true)?,
            right_sidebar_fixed: p1.header.bool_with_default("right-sidebar-fixed", true)?,
            logo: p1.header.string_with_default(LOGO_KEY, LOGO)?,
        })
    }

    fn to_p1(&self) -> crate::p1::SubSection {
        let mut p1 = crate::p1::SubSection::with_name("design");
        p1 = p1.add_header_if_not_equal("theme", self.theme.to_string(), "darkula");
        p1 = p1.add_header_if_not_equal("layout", self.layout.to_string(), "three_column");
        p1 = p1.add_header_if_not_equal("header-fixed", self.header_fixed, false);
        p1 = p1.add_header_if_not_equal("left-sidebar-fixed", self.left_sidebar_fixed, true);
        p1 = p1.add_header_if_not_equal("right-sidebar-fixed", self.right_sidebar_fixed, true);
        p1 = p1.add_header_if_not_equal(LOGO_KEY, self.logo.as_str(), LOGO);
        p1
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct CR {
    pub status: CRStatus,
    pub reviewers: Vec<Reviewer>,
}

impl CR {
    pub fn set_status(&mut self, status: CRStatus) {
        self.status = status
    }

    pub fn from_p1(p1: &crate::p1::SubSection) -> Result<Self, crate::document::ParseError> {
        let mut reviewers = vec![];
        for (k, v) in p1.header.0.iter() {
            match k.as_str() {
                ASSIGNED_TO_KEY => reviewers.push(Reviewer::AssignedTo {
                    username: v.to_string(),
                }),
                APPROVED_BY_KEY => reviewers.push(Reviewer::ApprovedBy {
                    username: v.to_string(),
                }),
                REJECTED_BY_KEY => reviewers.push(Reviewer::RejectedBy {
                    username: v.to_string(),
                }),
                CHANGES_REQUESTED_BY_KEY => reviewers.push(Reviewer::ChangesRequestedBy {
                    username: v.to_string(),
                }),
                STATUS_KEY => {}
                t => {
                    return crate::document::err(
                            format!(
                                "unknown value: {}, allowed values: assigned-to, approved-by, rejected-by, or changes-requested-by",
                                t
                            ).as_str(),
                    )?;
                }
            }
        }
        Ok(CR {
            status: match p1.header.str_optional(STATUS_KEY)? {
                Some(OPEN_KEY) | None => CRStatus::Open,
                Some(CLOSED_KEY) => CRStatus::Closed,
                Some(WIP_KEY) => CRStatus::WIP,
                Some(t) => {
                    return crate::document::err(
                        format!("unknown status: {}, allowed values: open, closed or wip", t)
                            .as_str(),
                    )?;
                }
            },
            reviewers,
        })
    }

    pub fn to_p1(&self) -> crate::p1::SubSection {
        let mut p1 = crate::p1::SubSection::with_name("cr")
            .add_header(STATUS_KEY, self.status.to_string().as_str());
        for reviewer in &self.reviewers {
            p1 = match &reviewer {
                Reviewer::AssignedTo { username } => p1.add_header(ASSIGNED_TO_KEY, username),
                Reviewer::ApprovedBy { username } => p1.add_header(APPROVED_BY_KEY, username),
                Reviewer::RejectedBy { username } => p1.add_header(REJECTED_BY_KEY, username),
                Reviewer::ChangesRequestedBy { username } => {
                    p1.add_header(CHANGES_REQUESTED_BY_KEY, username)
                }
            };
        }
        p1
    }
}

impl std::fmt::Display for CRStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CRStatus::WIP => WIP_KEY,
                CRStatus::Open => OPEN_KEY,
                CRStatus::Closed => CLOSED_KEY,
            }
        )
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum CRStatus {
    Open,
    WIP,
    Closed,
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
#[serde(tag = "type")]
pub enum Reviewer {
    AssignedTo { username: String },
    ApprovedBy { username: String },
    RejectedBy { username: String },
    ChangesRequestedBy { username: String },
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
#[serde(tag = "type")]
pub enum ReviewStatus {
    NotRequired,
    Approved,
    Rejected,
    ChangesRequested,
    ApprovalRequired { required: i32 },
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub enum Delta {
    UserAdded { username: String },
    UserRemoved { username: String },
}

impl Meta {
    pub fn lang(&self) -> realm_lang::Language {
        match self.lang {
            crate::ValueWithDefault::Default(v) => v,
            crate::ValueWithDefault::Found(v) => v,
        }
    }

    pub fn delta(&self, new: &Meta) -> Vec<Delta> {
        let old: Vec<String> = self.usernames().iter().map(|u| u.to_lowercase()).collect();
        let new: Vec<String> = new.usernames().iter().map(|u| u.to_lowercase()).collect();

        let mut d = vec![];
        for item in old.iter() {
            if new.contains(item) {
                continue;
            }

            d.push(Delta::UserRemoved {
                username: item.to_string(),
            })
        }
        for item in new.iter() {
            if old.contains(item) {
                continue;
            }
            d.push(Delta::UserAdded {
                username: item.to_string(),
            })
        }

        d
    }

    pub fn reviewers_delta(&self, new: &Meta) -> Vec<String> {
        let old: Vec<String> = self
            .cr_reviewers()
            .iter()
            .map(|u| u.to_lowercase())
            .collect();
        let new: Vec<String> = new
            .cr_reviewers()
            .iter()
            .map(|u| u.to_lowercase())
            .collect();

        new.into_iter()
            .filter(|x| !old.contains(x))
            .collect::<Vec<String>>()
    }

    pub fn cr_reviewers(&self) -> Vec<String> {
        if let Some(cr) = &self.cr {
            cr.reviewers
                .iter()
                .map(|ref x| match x {
                    Reviewer::AssignedTo { username } => username.to_string(),
                    Reviewer::ApprovedBy { username } => username.to_string(),
                    Reviewer::RejectedBy { username } => username.to_string(),
                    Reviewer::ChangesRequestedBy { username } => username.to_string(),
                })
                .collect::<Vec<String>>()
        } else {
            vec![]
        }
    }

    pub fn usernames(&self) -> Vec<String> {
        let mut d = vec![];
        for s in self.surfers.iter() {
            if let Someone::Username(ref u) = s.who {
                d.push(u.to_string())
            }
        }
        for s in self.readers.iter() {
            if let Someone::Username(ref u) = s.who {
                d.push(u.to_string())
            }
        }
        for s in self.writers.iter() {
            if let Someone::Username(ref u) = s.who {
                d.push(u.to_string())
            }
        }
        for s in self.admins.iter() {
            if let Someone::Username(ref u) = s.who {
                d.push(u.to_string())
            }
        }
        d
    }

    pub fn set_cr_status(&mut self, status: CRStatus) {
        let cr = match self.cr.take() {
            Some(mut cr) => {
                cr.set_status(status);
                Some(cr)
            }
            None => Some(CR {
                status,
                reviewers: vec![],
            }),
        };
        self.cr = cr
    }

    pub fn is_public(&self) -> bool {
        for r in self.readers.iter() {
            if r.who == Someone::Everyone {
                return true;
            }
        }
        false
    }

    pub fn can_read(&self, username: &str) -> bool {
        if self.is_public() {
            return true;
        }

        for r in self.readers.iter() {
            match &r.who {
                Someone::Username(u) if username.to_lowercase() == u.to_lowercase() => return true,
                _ => (),
            };
        }

        if self.can_write(username) {
            return true;
        }

        if self.can_admin(username) {
            return true;
        }

        false
    }

    pub fn can_write(&self, username: &str) -> bool {
        for r in self.writers.iter() {
            match &r.who {
                Someone::Username(u) if username.to_lowercase() == u.to_lowercase() => return true,
                _ => (),
            };
        }

        if self.can_admin(username) {
            return true;
        }

        false
    }

    pub fn can_admin(&self, username: &str) -> bool {
        for r in self.admins.iter() {
            match &r.who {
                Someone::Username(u) if username.to_lowercase() == u.to_lowercase() => return true,
                _ => (),
            };
        }
        false
    }

    pub fn to_p1(&self) -> crate::p1::Section {
        let mut p1 = crate::p1::Section::with_name("meta");

        if let Some(ref cr) = self.cr {
            p1 = p1.add_sub_section(cr.to_p1());
        }
        if let Some(ref v) = self.git_repo {
            p1 = p1.add_header("git-repo", v);
        }
        if self.no_index {
            p1 = p1.add_header("no-index", "true");
        }
        for surfer in self.surfers.iter() {
            p1 = p1.add_sub_section(surfer.to_p1());
        }
        for reader in self.readers.iter() {
            p1 = p1.add_sub_section(reader.to_p1());
        }
        for writer in self.writers.iter() {
            p1 = p1.add_sub_section(writer.to_p1());
        }
        for writer in self.admins.iter() {
            p1 = p1.add_sub_section(writer.to_p1());
        }
        if let Some(ref design) = self.design {
            p1 = p1.add_sub_section(design.to_p1());
        }
        if self.minimum_cr_approvals != MINIMUM_CR_APPROVALS {
            p1 = p1.add_header(
                MINIMUM_CR_APPROVALS_KEY,
                &self.minimum_cr_approvals.to_string(),
            );
        }
        if let crate::ValueWithDefault::Found(lang) = self.lang {
            p1 = p1.add_header(LANG_KEY, lang.id());
        }

        if self.deleted {
            p1 = p1.add_header("deleted", "true");
        }

        self.translation.to_p1(&mut p1);
        p1
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, crate::document::ParseError> {
        let mut meta = Meta::default();
        for sub in p1.sub_sections.0.iter() {
            match sub.name.as_str() {
                "surfer" => meta.surfers.push(Surfer::from_p1(sub)?),
                "reader" => meta.readers.push(Reader::from_p1(sub)?),
                "writer" => meta.writers.push(Writer::from_p1(sub)?),
                "admin" => meta.admins.push(Admin::from_p1(sub)?),
                "cr" => meta.cr = Some(CR::from_p1(sub)?),
                "design" => meta.design = Some(Design::from_p1(sub)?),
                t => crate::document::err(format!("unknown sub-section: {}", t).as_str())?,
            }
        }
        meta.minimum_cr_approvals = p1
            .header
            .i32_with_default(MINIMUM_CR_APPROVALS_KEY, MINIMUM_CR_APPROVALS)?;
        meta.no_index = p1.header.bool_with_default("no-index", false)?;
        if let Some(lang) = p1.header.str_optional(LANG_KEY)? {
            meta.lang = crate::ValueWithDefault::Found(lang.parse()?);
        }
        meta.translation = Translation::from_p1(p1)?;
        meta.git_repo = p1.header.string_optional("git-repo")?;
        meta.deleted = p1.header.bool_with_default("deleted", false)?;
        Ok(meta)
    }

    pub fn get_translation(&self) -> &Translation {
        &self.translation
    }

    pub fn get_lang(&self) -> &crate::ValueWithDefault<realm_lang::Language> {
        &self.lang
    }

    pub fn with_reader(mut self, reader: Reader) -> Self {
        self.readers.push(reader);
        self
    }

    pub fn with_admin(mut self, admin: Admin) -> Self {
        self.admins.push(admin);
        self
    }

    pub fn with_translation(mut self, translation: Translation) -> Self {
        self.translation = translation;
        self
    }

    pub fn with_translation_of(self, id: &str) -> Self {
        self.with_translation(Translation::ItIsTranslationOf { id: id.to_string() })
    }

    pub fn with_lang(mut self, lang: realm_lang::Language) -> Self {
        self.lang = crate::ValueWithDefault::found(lang);
        self
    }

    pub fn git_repo(&self) -> Option<String> {
        self.git_repo.as_ref().map(|x| x.to_string())
    }

    pub fn with_deleted(mut self) -> Self {
        self.deleted = true;
        self
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub struct Surfer {
    who: Someone,
    can_see_toc: bool,
}

impl Surfer {
    fn from_p1(p1: &crate::p1::SubSection) -> Result<Self, crate::document::ParseError> {
        Ok(Surfer {
            who: Someone::from_p1(p1)?,
            can_see_toc: p1.header.bool_with_default("can_see_toc", true)?,
        })
    }

    fn to_p1(&self) -> crate::p1::SubSection {
        self.who.to_p1(
            crate::p1::SubSection::with_name("surfer").add_header_if_not_equal(
                "can_see_toc",
                self.can_see_toc,
                true,
            ),
        )
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub struct Reader {
    who: Someone,
    can_create_cr: bool,
    can_see_history: bool,
}

impl Reader {
    fn from_p1(p1: &crate::p1::SubSection) -> Result<Self, crate::document::ParseError> {
        Ok(Reader {
            who: Someone::from_p1(p1)?,
            can_create_cr: p1.header.bool_with_default("can_create_cr", true)?,
            can_see_history: p1.header.bool_with_default("can_see_history", true)?,
        })
    }

    fn to_p1(&self) -> crate::p1::SubSection {
        self.who.to_p1(
            crate::p1::SubSection::with_name("reader")
                .add_header_if_not_equal("can_create_cr", self.can_create_cr, true)
                .add_header_if_not_equal("can_see_history", self.can_create_cr, true),
        )
    }

    pub fn everyone() -> Reader {
        Self {
            who: Someone::Everyone,
            ..Default::default()
        }
    }
}

impl Default for Reader {
    fn default() -> Self {
        Self {
            who: Someone::Everyone,
            can_create_cr: true,
            can_see_history: true,
        }
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub struct Writer {
    who: Someone,
}

impl Writer {
    fn from_p1(p1: &crate::p1::SubSection) -> Result<Self, crate::document::ParseError> {
        let who = Someone::from_p1(p1)?;
        if who == Someone::Everyone {
            return crate::document::err("you cant make everyone writer");
        }
        Ok(Writer { who })
    }

    fn to_p1(&self) -> crate::p1::SubSection {
        self.who.to_p1(crate::p1::SubSection::with_name("writer"))
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub struct Admin {
    who: Someone,
}

impl Admin {
    fn from_p1(p1: &crate::p1::SubSection) -> Result<Self, crate::document::ParseError> {
        let who = Someone::from_p1(p1)?;
        if who == Someone::Everyone {
            return crate::document::err("you cant make everyone admin");
        }
        Ok(Admin { who })
    }

    fn to_p1(&self) -> crate::p1::SubSection {
        self.who.to_p1(crate::p1::SubSection::with_name("admin"))
    }

    pub fn with_username(username: &str) -> Self {
        Admin {
            who: Someone::Username(username.to_string()),
        }
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub enum Someone {
    Everyone,
    Followers,
    Email(String),
    Username(String),
    Org(String),
    Team { org: String, team: String },
}

impl Someone {
    fn to_p1(&self, p1: crate::p1::SubSection) -> crate::p1::SubSection {
        match self {
            Someone::Everyone => p1.add_header("who", "everyone"),
            Someone::Followers => p1.add_header("who", "followers"),
            Someone::Email(e) => p1.add_header("email", e),
            Someone::Username(u) => p1.add_header("username", u),
            Someone::Org(o) => p1.add_header("org", o),
            Someone::Team { org, team } => {
                p1.add_header("team", format!("{}/{}", org, team).as_str())
            }
        }
    }

    fn from_p1(p1: &crate::p1::SubSection) -> Result<Self, crate::document::ParseError> {
        if let Some(val) = p1.header.str_optional("email")? {
            // TODO: valid email
            return Ok(Someone::Email(val.to_string()));
        }
        if let Some(val) = p1.header.str_optional("username")? {
            // TODO: valid username (regex, not if it exists in db)
            return Ok(Someone::Username(val.to_string()));
        }
        if let Some(val) = p1.header.str_optional("org")? {
            // TODO: valid org (regex, not if it exists in db)
            return Ok(Someone::Org(val.to_string()));
        }
        if let Some(val) = p1.header.str_optional("team")? {
            let mut parts = val.splitn(2, '/');
            return match (parts.next(), parts.next()) {
                (Some(o), Some(t)) => Ok(Someone::Team {
                    org: o.to_string(),
                    team: t.to_string(),
                }),
                _ => crate::document::err(
                    format!("expected a value like ftweb/frontend, found: {}", val).as_str(),
                ),
            };
        }
        if let Some(val) = p1.header.str_optional("who")? {
            return if val == "everyone" {
                Ok(Someone::Everyone)
            } else if val == "followers" {
                Ok(Someone::Followers)
            } else {
                crate::document::err(
                    format!("who can either be everyone or followers, not: {}", val).as_str(),
                )
            };
        }
        crate::document::err("one of email, username, team or org is required")
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn delta() {
        let old = crate::Meta {
            surfers: vec![crate::Surfer {
                who: crate::Someone::Everyone,
                can_see_toc: false,
            }],
            readers: vec![
                crate::Reader {
                    who: crate::Someone::Org("fifthtry".to_string()),
                    can_create_cr: false,
                    can_see_history: true,
                },
                crate::Reader {
                    who: crate::Someone::Team {
                        org: "browserstack".to_string(),
                        team: "leadership".to_string(),
                    },
                    can_create_cr: true,
                    can_see_history: true,
                },
            ],
            writers: vec![
                crate::Writer {
                    who: crate::Someone::Username("amitu".to_string()),
                },
                crate::Writer {
                    who: crate::Someone::Followers,
                },
            ],
            admins: vec![crate::Admin {
                who: crate::Someone::Email("foo@bar.com".to_string()),
            }],
            ..Default::default()
        };

        assert_eq!(old.usernames(), vec!["amitu".to_string()]);

        let new = crate::Meta {
            surfers: vec![crate::Surfer {
                who: crate::Someone::Everyone,
                can_see_toc: false,
            }],
            readers: vec![
                crate::Reader {
                    who: crate::Someone::Org("fifthtry".to_string()),
                    can_create_cr: false,
                    can_see_history: true,
                },
                crate::Reader {
                    who: crate::Someone::Team {
                        org: "browserstack".to_string(),
                        team: "leadership".to_string(),
                    },
                    can_create_cr: true,
                    can_see_history: true,
                },
            ],
            writers: vec![
                crate::Writer {
                    who: crate::Someone::Username("amitu2".to_string()),
                },
                crate::Writer {
                    who: crate::Someone::Followers,
                },
            ],
            admins: vec![crate::Admin {
                who: crate::Someone::Email("foo@bar.com".to_string()),
            }],
            ..Default::default()
        };

        assert_eq!(new.usernames(), vec!["amitu2".to_string()]);

        let delta = old.delta(&new);
        assert_eq!(
            delta,
            vec![
                super::Delta::UserRemoved {
                    username: "amitu".to_string()
                },
                super::Delta::UserAdded {
                    username: "amitu2".to_string()
                }
            ]
        )
    }

    #[test]
    fn basic() {
        p(
            &indoc::indoc!(
                "
            -- meta:

            --- surfer:
            who: everyone
            can_see_toc: false

            --- reader:
            org: fifthtry
            can_create_cr: false
            can_see_history: true

            --- reader:
            team: browserstack/leadership

            --- writer:
            username: amitu

            --- writer:
            who: followers

            --- admin:
            email: foo@bar.com
            "
            ),
            &vec![crate::Section::Meta(crate::Meta {
                cr: None,
                surfers: vec![crate::Surfer {
                    who: crate::Someone::Everyone,
                    can_see_toc: false,
                }],
                readers: vec![
                    crate::Reader {
                        who: crate::Someone::Org("fifthtry".to_string()),
                        can_create_cr: false,
                        can_see_history: true,
                    },
                    crate::Reader {
                        who: crate::Someone::Team {
                            org: "browserstack".to_string(),
                            team: "leadership".to_string(),
                        },
                        can_create_cr: true,
                        can_see_history: true,
                    },
                ],
                writers: vec![
                    crate::Writer {
                        who: crate::Someone::Username("amitu".to_string()),
                    },
                    crate::Writer {
                        who: crate::Someone::Followers,
                    },
                ],
                admins: vec![crate::Admin {
                    who: crate::Someone::Email("foo@bar.com".to_string()),
                }],
                ..Default::default()
            })],
        );
    }

    #[test]
    fn no_index() {
        p(
            &indoc::indoc!(
                "
            -- meta:
            no-index: true
            "
            ),
            &vec![crate::Section::Meta(crate::Meta {
                no_index: true,
                ..Default::default()
            })],
        );
        p(
            &indoc::indoc!(
                "
            -- meta:
            "
            ),
            &vec![crate::Section::Meta(crate::Meta {
                no_index: false,
                ..Default::default()
            })],
        );
    }

    #[test]
    fn cr_reviewers_1() {
        p(
            &indoc::indoc!(
                "
            -- meta:

            --- cr:
            status: open
            assigned-to: wilderbit
            "
            ),
            &vec![crate::Section::Meta(crate::Meta {
                cr: Some(super::CR {
                    status: super::CRStatus::Open,
                    reviewers: vec![super::Reviewer::AssignedTo {
                        username: "wilderbit".to_string(),
                    }],
                }),
                ..Default::default()
            })],
        );
    }

    #[test]
    fn cr_reviewers_2() {
        p(
            &indoc::indoc!(
                "
            -- meta:

            --- cr:
            status: closed
            approved-by: wilderbit
            "
            ),
            &vec![crate::Section::Meta(crate::Meta {
                cr: Some(super::CR {
                    status: super::CRStatus::Closed,
                    reviewers: vec![super::Reviewer::ApprovedBy {
                        username: "wilderbit".to_string(),
                    }],
                }),
                ..Default::default()
            })],
        );
    }

    #[test]
    fn cr_assigned_to_3() {
        p(
            &indoc::indoc!(
                "
            -- meta:

            --- cr:
            status: wip
            "
            ),
            &vec![crate::Section::Meta(crate::Meta {
                cr: Some(super::CR {
                    status: super::CRStatus::WIP,
                    reviewers: vec![],
                }),
                ..Default::default()
            })],
        );
    }

    #[test]
    fn cr_assigned_to_4() {
        p(
            &indoc::indoc!(
                "
            -- meta:

            --- cr:
            status: wip
            rejected-by: wilderbit

            "
            ),
            &vec![crate::Section::Meta(crate::Meta {
                cr: Some(super::CR {
                    status: super::CRStatus::WIP,
                    reviewers: vec![super::Reviewer::RejectedBy {
                        username: "wilderbit".to_string(),
                    }],
                }),
                ..Default::default()
            })],
        );
    }

    #[test]
    fn cr_assigned_to_5() {
        p(
            &indoc::indoc!(
                "
            -- meta:

            --- cr:
            status: wip
            changes-requested-by: wilderbit
            "
            ),
            &vec![crate::Section::Meta(crate::Meta {
                cr: Some(super::CR {
                    status: super::CRStatus::WIP,
                    reviewers: vec![super::Reviewer::ChangesRequestedBy {
                        username: "wilderbit".to_string(),
                    }],
                }),
                ..Default::default()
            })],
        );
    }

    #[test]
    fn lang() {
        p(
            &indoc::indoc!(
                "
            -- meta:
            lang: hi
            "
            ),
            &vec![crate::Section::Meta(crate::Meta {
                lang: crate::ValueWithDefault::Found(realm_lang::Language::Hindi),
                ..Default::default()
            })],
        );
    }

    #[test]
    fn translation() {
        p(
            &indoc::indoc!(
                "
            -- meta:
            translation: foo/bar
            translation: x/y
            "
            ),
            &vec![crate::Section::Meta(crate::Meta {
                translation: super::Translation::ItHasTranslations {
                    translations: vec!["foo/bar".to_string(), "x/y".to_string()],
                },
                ..Default::default()
            })],
        );

        p(
            &indoc::indoc!(
                "
            -- meta:
            translation-of: foo/bar
            "
            ),
            &vec![crate::Section::Meta(crate::Meta {
                translation: super::Translation::ItIsTranslationOf {
                    id: "foo/bar".to_string(),
                },
                ..Default::default()
            })],
        );

        f(
            &indoc::indoc!(
                "
            -- meta:
            translation-of: foo/bar
            translation: x/y
            "
            ),
            "ValidationError: both translation-of: and translation: headers can\'t be specified",
        );
    }
}

pub const MINIMUM_CR_APPROVALS: i32 = 0;
const MINIMUM_CR_APPROVALS_KEY: &str = "minimum-cr-approvals";
// const SIDEBAR_COLOR: &str = "#FDF0E3";
// const SIDEBAR_COLOR_KEY: &str = "sidebar-color";
// const SIDEBAR_TEXT_COLOR: &str = "#935328";
// const SIDEBAR_TEXT_COLOR_KEY: &str = "sidebar-text-color";
// const BUTTON_COLOR: &str = "#F38F32";
// const BUTTON_COLOR_KEY: &str = "button-color";
// const BUTTON_TEXT_COLOR: &str = "#FFFFFF";
// const BUTTON_TEXT_COLOR_KEY: &str = "button-text-color";
// const LINK_COLOR: &str = "#935328";
// const LINK_COLOR_KEY: &str = "link-color";
// const BACKGROUND_COLOR: &str = "#FFFFFF";
// const BACKGROUND_COLOR_KEY: &str = "background-color";
const LOGO: &str =
    "https://res.cloudinary.com/dlgztvq9v/image/upload/v1611131493/fifthtry/fifthtry_bgiic1.svg";
const LOGO_KEY: &str = "logo";
const ASSIGNED_TO_KEY: &str = "assigned-to";
const APPROVED_BY_KEY: &str = "approved-by";
const REJECTED_BY_KEY: &str = "rejected-by";
const CHANGES_REQUESTED_BY_KEY: &str = "changes-requested-by";
const STATUS_KEY: &str = "status";
const WIP_KEY: &str = "wip";
const OPEN_KEY: &str = "open";
const CLOSED_KEY: &str = "closed";
const LANG_KEY: &str = "lang";
