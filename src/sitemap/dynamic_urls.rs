#[derive(Debug, serde::Deserialize, Clone)]
pub struct DynamicUrlsTemp {
    #[serde(rename = "dynamic-urls-body")]
    pub body: Option<String>,
}

pub struct DynamicUrls {
    pub urls: Vec<DynamicUrl>,
    // Todo: Inherit from sitemap if present else person can
    // because if a person did not defined sitemap, sitemap is optional
    // pub readers: Vec<String>,
    // pub writers: Vec<String>,
}

pub struct DynamicUrl {
    pub id: String,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
    /// In FPM.ftd sitemap, we can use `document` for section, subsection and toc.
    /// # Section: /books/
    ///   document: /books/python/
    pub document: Option<String>,
    /// If we can define dynamic `url` in section, subsection and toc of a sitemap.
    /// `url: /books/<string:book_name>/<integer:price>/`
    /// here book_name and price are path parameters
    /// path_parameters: [(string, book_name), (integer, price)]
    pub path_parameters: Vec<(String, String)>,
}
