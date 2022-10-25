pub struct DynamicUrls {
    pub urls: Vec<DynamicUrl>,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
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
