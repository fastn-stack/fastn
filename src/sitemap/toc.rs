#[derive(Debug, Clone, Default)]
pub struct TocItem {
    pub id: String,
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

    /// path: /foo/demo/
    /// path: /
    pub fn resolve_document(&self, path: &str) -> fpm::Result<fpm::sitemap::ResolveDocOutput> {
        if !self.path_parameters.is_empty() {
            // path: /arpita/foo/28/
            // request: arpita foo 28
            // sitemap: [string,integer]
            // Mapping: arpita -> string, foo -> foo, 28 -> integer
            let params = fpm::sitemap::utils::parse_named_params(
                path,
                self.id.as_str(),
                self.path_parameters.as_slice(),
            );

            if params.is_ok() {
                return Ok((self.document.clone(), params?));
            }
        } else if fpm::utils::ids_matches(self.id.as_str(), path) {
            return Ok((self.document.clone(), vec![]));
        }

        for child in self.children.iter() {
            let (document, path_prams) = child.resolve_document(path)?;
            if document.is_some() {
                return Ok((document, path_prams));
            }
        }
        Ok((None, vec![]))
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
pub struct TocItemCompat {
    pub url: Option<String>,
    pub number: Option<String>,
    pub title: Option<String>,
    pub path: Option<String>,
    #[serde(rename = "is-heading")]
    pub is_heading: bool,
    // TODO: Font icon mapping to html?
    #[serde(rename = "font-icon")]
    pub font_icon: Option<String>,
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
    ) -> TocItemCompat {
        TocItemCompat {
            url,
            number: None,
            title,
            path: None,
            is_heading: false,
            font_icon: None,
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
