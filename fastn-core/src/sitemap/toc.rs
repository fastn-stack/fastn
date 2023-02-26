#[derive(Debug, Clone, PartialEq)]
pub struct TocItem {
    pub id: String,
    pub icon: Option<String>,
    pub bury: bool,
    pub title: Option<String>,
    pub file_location: Option<camino::Utf8PathBuf>,
    pub translation_file_location: Option<camino::Utf8PathBuf>,
    pub extra_data: std::collections::BTreeMap<String, String>,
    pub is_active: bool,
    pub nav_title: Option<String>,
    pub children: Vec<TocItem>,
    pub skip: bool,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
    pub document: Option<String>,
    /// if provided `document` is confidential or not.
    /// `confidential:true` means totally confidential
    /// `confidential:false` can be seen some it's data
    pub confidential: bool,
    /// /books/<string:book_name>/
    /// here book_name is path parameter
    pub path_parameters: Vec<fastn_core::sitemap::PathParams>,
}

impl Default for TocItem {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            icon: None,
            bury: false,
            title: None,
            file_location: None,
            translation_file_location: None,
            extra_data: Default::default(),
            is_active: false,
            confidential: true,
            children: vec![],
            skip: false,
            readers: vec![],
            writers: vec![],
            nav_title: None,
            document: None,
            path_parameters: vec![],
        }
    }
}

impl TocItem {
    /// path: /foo/demo/
    /// path: /
    pub fn path_exists(&self, path: &str) -> bool {
        if fastn_core::utils::ids_matches(self.id.as_str(), path) {
            return true;
        }

        for child in self.children.iter() {
            if child.path_exists(path) {
                return true;
            }
        }

        false
    }

    /// returns the file id portion of the url only in case
    /// any component id is referred in the url itself
    pub fn get_file_id(&self) -> String {
        self.id
            .rsplit_once('#')
            .map(|s| s.0)
            .unwrap_or(self.id.as_str())
            .to_string()
    }
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct ImageSrc {
    pub light: String,
    pub dark: String,
}

impl From<String> for ImageSrc {
    fn from(path: String) -> Self {
        ImageSrc {
            light: path.clone(),
            dark: path,
        }
    }
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct TocItemCompat {
    pub url: Option<String>,
    pub number: Option<String>,
    pub title: Option<String>,
    pub path: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "is-heading")]
    pub is_heading: bool,
    // TODO: Font icon mapping to html?
    #[serde(rename = "font-icon")]
    pub font_icon: Option<ImageSrc>,
    pub bury: bool,
    #[serde(rename = "is-disabled")]
    pub is_disabled: bool,
    #[serde(rename = "is-active")]
    pub is_active: bool,
    #[serde(rename = "is-open")]
    pub is_open: bool,
    #[serde(rename = "img-src")]
    pub image_src: Option<String>,
    pub children: Vec<TocItemCompat>,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
    pub document: Option<String>,
    pub extra_data: std::collections::BTreeMap<String, String>,
    #[serde(rename = "nav-title")]
    pub nav_title: Option<String>,
}

#[allow(clippy::too_many_arguments)]
impl TocItemCompat {
    pub(crate) fn new(
        url: Option<String>,
        title: Option<String>,
        is_active: bool,
        is_open: bool,
        readers: Vec<String>,
        writers: Vec<String>,
        icon: Option<String>,
        bury: bool,
    ) -> TocItemCompat {
        TocItemCompat {
            url,
            number: None,
            title,
            path: None,
            description: None,
            is_heading: false,
            font_icon: icon.map(Into::into),
            bury,
            is_disabled: false,
            is_active,
            is_open,
            image_src: None,
            children: vec![],
            readers,
            writers,
            document: None,
            extra_data: Default::default(),
            nav_title: None,
        }
    }

    pub(crate) fn add_path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }
}
