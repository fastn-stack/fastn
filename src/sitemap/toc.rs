#[derive(Debug, Clone, Default, PartialEq)]
pub struct TocItem {
    pub id: String,
    pub icon: Option<String>,
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
    /// /books/<string:book_name>/
    /// here book_name is path parameter
    pub path_parameters: Vec<(String, String)>,
}

impl TocItem {
    /// path: /foo/demo/
    /// path: /
    pub fn path_exists(&self, path: &str) -> bool {
        if fpm::utils::ids_matches(self.id.as_str(), path) {
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
    #[serde(rename = "is-heading")]
    pub is_heading: bool,
    // TODO: Font icon mapping to html?
    #[serde(rename = "font-icon")]
    pub font_icon: Option<ImageSrc>,
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
}

impl TocItemCompat {
    pub(crate) fn new(
        url: Option<String>,
        title: Option<String>,
        is_active: bool,
        is_open: bool,
        readers: Vec<String>,
        writers: Vec<String>,
        icon: Option<String>,
    ) -> TocItemCompat {
        TocItemCompat {
            url,
            number: None,
            title,
            path: None,
            is_heading: false,
            font_icon: icon.map(Into::into),
            is_disabled: false,
            is_active,
            is_open,
            image_src: None,
            children: vec![],
            readers,
            writers,
            document: None,
        }
    }

    pub(crate) fn add_path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }
}
