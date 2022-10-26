use crate::sitemap::{ParseError, ParsingState, SitemapParser};

#[derive(Debug, serde::Deserialize, Clone)]
pub struct DynamicUrlsTemp {
    #[serde(rename = "dynamic-urls-body")]
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct DynamicUrls {
    pub urls: Vec<DynamicUrl>,
    // Todo: Inherit from sitemap if present else person can
    // because if a person did not defined sitemap, sitemap is optional
    // pub readers: Vec<String>,
    // pub writers: Vec<String>,
}

impl DynamicUrls {
    pub fn parse(
        s: &str,
        package: &fpm::Package,
        config: &mut fpm::Config,
        asset_documents: &std::collections::HashMap<String, String>,
        base_url: &str,
        resolve_sitemap: bool,
    ) -> Result<Self, ParseError> {
        dbg!("Dynamic Urls Body");
        dbg!(s);

        let mut parser = SitemapParser {
            state: ParsingState::WaitingForSection,
            sections: vec![],
            temp_item: None,
            doc_name: package.name.to_string(),
        };

        for line in s.split('\n') {
            parser.read_line(line, &config.global_ids)?;
        }

        if parser.temp_item.is_some() {
            parser.eval_temp_item(&config.global_ids)?;
        }

        dbg!(&parser);

        Ok(DynamicUrls { urls: vec![] })
    }
}

#[derive(Debug, Clone)]
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
