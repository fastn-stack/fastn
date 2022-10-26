#[derive(Debug, serde::Deserialize, Clone)]
pub struct DynamicUrlsTemp {
    #[serde(rename = "dynamic-urls-body")]
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct DynamicUrls {
    pub sections: Vec<fpm::sitemap::section::Section>,
}

impl DynamicUrls {
    pub fn parse(
        config: &mut fpm::Config,
        package: &fpm::Package,
        s: &str,
    ) -> Result<Self, fpm::sitemap::ParseError> {
        // Note: Using Sitemap Parser, because format of dynamic-urls is same as sitemap
        let mut parser = fpm::sitemap::SitemapParser {
            state: fpm::sitemap::ParsingState::WaitingForSection,
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

        Ok(DynamicUrls {
            sections: fpm::sitemap::construct_tree_util(parser.finalize()?),
        })
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
