/// `Sitemap` stores the sitemap for the fpm package defines in the FPM.ftd
///
/// ```ftd
/// -- fpm.sitemap:
///
/// # foo/
/// ## bar/
/// - doc-1/
///   - childdoc-1/
/// - doc-2/
/// ```
///
/// In above example, the id starts with `#` becomes the section. Similarly the id
/// starts with `##` becomes the subsection and then the id starts with `-` becomes
/// the table od content (TOC).
pub mod dynamic_urls;
pub mod section;
pub mod toc;
pub mod utils;

pub use dynamic_urls::{DynamicUrls, DynamicUrlsTemp};

#[derive(Debug, Clone, Default)]
pub struct Sitemap {
    pub sections: Vec<section::Section>,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct SiteMapCompat {
    pub sections: Vec<toc::TocItemCompat>,
    pub subsections: Vec<toc::TocItemCompat>,
    pub toc: Vec<toc::TocItemCompat>,
    #[serde(rename = "current-section")]
    pub current_section: Option<toc::TocItemCompat>,
    #[serde(rename = "current-subsection")]
    pub current_subsection: Option<toc::TocItemCompat>,
    #[serde(rename = "current-page")]
    pub current_page: Option<toc::TocItemCompat>,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SitemapElement {
    Section(section::Section),
    Subsection(section::Subsection),
    TocItem(toc::TocItem),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathParams {
    NamedParm {
        index: usize,
        name: String,
        param_type: String,
    },
    ValueParam {
        index: usize,
        value: String,
    },
}

impl PathParams {
    pub fn named(index: usize, name: String, param_type: String) -> Self {
        PathParams::NamedParm {
            index,
            name,
            param_type,
        }
    }

    pub fn value(index: usize, value: String) -> Self {
        PathParams::ValueParam { index, value }
    }

    pub fn is_named_param(&self) -> bool {
        matches!(self, Self::NamedParm { .. })
    }
}

impl SitemapElement {
    pub(crate) fn insert_key_value(&mut self, key: &str, value: &str) {
        let element_title = match self {
            SitemapElement::Section(s) => &mut s.extra_data,
            SitemapElement::Subsection(s) => &mut s.extra_data,
            SitemapElement::TocItem(s) => &mut s.extra_data,
        };
        element_title.insert(key.to_string(), value.trim().to_string());
    }

    pub(crate) fn set_title(&mut self, title: Option<String>) {
        let element_title = match self {
            SitemapElement::Section(s) => &mut s.title,
            SitemapElement::Subsection(s) => &mut s.title,
            SitemapElement::TocItem(s) => &mut s.title,
        };
        *element_title = title;
    }

    pub(crate) fn set_icon(&mut self, path: Option<String>) {
        let element_icon = match self {
            SitemapElement::Section(s) => &mut s.icon,
            SitemapElement::Subsection(s) => &mut s.icon,
            SitemapElement::TocItem(s) => &mut s.icon,
        };
        *element_icon = path;
    }

    pub(crate) fn set_bury(&mut self, value: bool) {
        let element_bury = match self {
            SitemapElement::Section(s) => &mut s.bury,
            SitemapElement::Subsection(s) => &mut s.bury,
            SitemapElement::TocItem(s) => &mut s.bury,
        };
        *element_bury = value;
    }

    pub(crate) fn set_id(&mut self, id: Option<String>) {
        let id = if let Some(id) = id {
            id
        } else {
            return;
        };
        match self {
            SitemapElement::Section(s) => {
                s.id = id;
            }
            SitemapElement::Subsection(s) => {
                s.id = Some(id);
            }
            SitemapElement::TocItem(s) => {
                s.id = id;
            }
        };
    }

    pub(crate) fn set_nav_title(&mut self, nav_title: Option<String>) {
        let nav = match self {
            SitemapElement::Section(s) => &mut s.nav_title,
            SitemapElement::Subsection(s) => &mut s.nav_title,
            SitemapElement::TocItem(s) => &mut s.nav_title,
        };
        *nav = nav_title;
    }

    pub(crate) fn set_skip(&mut self, flag: bool) {
        let skip = match self {
            SitemapElement::Section(s) => &mut s.skip,
            SitemapElement::Subsection(s) => &mut s.skip,
            SitemapElement::TocItem(s) => &mut s.skip,
        };
        *skip = flag;
    }

    pub(crate) fn set_confidential(&mut self, flag: bool) {
        let skip = match self {
            SitemapElement::Section(s) => &mut s.confidential,
            SitemapElement::Subsection(s) => &mut s.confidential,
            SitemapElement::TocItem(s) => &mut s.confidential,
        };
        *skip = flag;
    }

    pub(crate) fn set_readers(&mut self, group: &str) {
        let readers = match self {
            SitemapElement::Section(s) => &mut s.readers,
            SitemapElement::Subsection(s) => &mut s.readers,
            SitemapElement::TocItem(s) => &mut s.readers,
        };
        readers.push(group.to_string());
    }

    pub(crate) fn set_writers(&mut self, group: &str) {
        let writers = match self {
            SitemapElement::Section(s) => &mut s.writers,
            SitemapElement::Subsection(s) => &mut s.writers,
            SitemapElement::TocItem(s) => &mut s.writers,
        };
        writers.push(group.to_string());
    }

    pub(crate) fn set_document(&mut self, doc: &str) {
        let document = match self {
            SitemapElement::Section(s) => &mut s.document,
            SitemapElement::Subsection(s) => &mut s.document,
            SitemapElement::TocItem(s) => &mut s.document,
        };
        *document = Some(doc.to_string());
    }

    pub(crate) fn get_title(&self) -> Option<String> {
        match self {
            SitemapElement::Section(s) => &s.title,
            SitemapElement::Subsection(s) => &s.title,
            SitemapElement::TocItem(s) => &s.title,
        }
        .clone()
    }

    pub(crate) fn get_id(&self) -> Option<String> {
        match self {
            SitemapElement::Section(s) => Some(s.id.clone()),
            SitemapElement::Subsection(s) => s.id.clone(),
            SitemapElement::TocItem(s) => Some(s.id.clone()),
        }
    }

    // If url contains path parameters so it will set those parameters
    // /person/<string:username>/<integer:age>
    // In that case it will parse and set parameters `username` and `age`
    pub(crate) fn set_path_params(&mut self, url: &str) -> Result<(), ParseError> {
        let params = utils::parse_named_params(url)?;

        if !params.is_empty() {
            self.set_skip(true);
        }

        match self {
            SitemapElement::Section(s) => {
                s.path_parameters = params;
            }
            SitemapElement::Subsection(s) => {
                s.path_parameters = params;
            }
            SitemapElement::TocItem(t) => {
                t.path_parameters = params;
            }
        }
        Ok(())
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ParseError {
    #[error("{doc_id} -> {message} -> Row Content: {row_content}")]
    InvalidTOCItem {
        doc_id: String,
        message: String,
        row_content: String,
    },
    #[error("InvalidUserGroup: {doc_id} -> {message} -> Row Content: {row_content}")]
    InvalidUserGroup {
        doc_id: String,
        message: String,
        row_content: String,
    },
    #[error("id: {id} not found while linking in sitemap, doc: {doc_id}")]
    InvalidID { doc_id: String, id: String },
    #[error("message: {message} ")]
    InvalidSitemap { message: String },
    #[error("message: {message} ")]
    InvalidDynamicUrls { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsingState {
    WaitingForSection,
    ParsingSection,
    ParsingSubsection,
    ParsingTOC,
}
#[derive(Debug)]
pub struct SitemapParser {
    state: ParsingState,
    sections: Vec<(SitemapElement, usize)>,
    temp_item: Option<(SitemapElement, usize)>,
    doc_name: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct SitemapTemp {
    #[serde(rename = "sitemap-body")]
    pub body: String,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
}

impl SitemapParser {
    pub fn read_line(
        &mut self,
        line: &str,
        global_ids: &std::collections::HashMap<String, String>,
    ) -> Result<(), ParseError> {
        // The row could be one of the 4 things:

        // - Heading
        // - Prefix/suffix item
        // - Separator
        // - ToC item

        if line.trim().is_empty() {
            return Ok(());
        }

        let mut iter = line.chars();
        let mut depth = 0;
        let mut rest = "".to_string();
        loop {
            match iter.next() {
                Some(' ') => {
                    depth += 1;
                    iter.next();
                }
                Some('-') => {
                    rest = iter.collect::<String>();
                    if ![
                        ParsingState::ParsingSection,
                        ParsingState::ParsingSubsection,
                        ParsingState::ParsingTOC,
                    ]
                    .contains(&self.state)
                    {
                        return Err(ParseError::InvalidTOCItem {
                            doc_id: self.doc_name.clone(),
                            message: "Ambiguous <title>: <URL> evaluation. TOC is found before section or subsection".to_string(),
                            row_content: rest.as_str().to_string(),
                        });
                    }
                    self.state = ParsingState::ParsingTOC;
                    break;
                }
                Some('#') => {
                    // Heading can not have any attributes. Append the item and look for the next input
                    rest = iter.collect::<String>();
                    self.state = ParsingState::ParsingSection;
                    if let Some(content) = rest.strip_prefix('#') {
                        if !ParsingState::ParsingSection.eq(&self.state) {
                            return Err(ParseError::InvalidTOCItem {
                                doc_id: self.doc_name.clone(),
                                message: "Ambiguous <title>: <URL> evaluation. Subsection is called before subsection".to_string(),
                                row_content: rest.as_str().to_string(),
                            });
                        }
                        rest = content.to_string();
                        self.state = ParsingState::ParsingSubsection;
                    }
                    break;
                }
                Some(k) => {
                    let l = format!("{}{}", k, iter.collect::<String>());
                    self.parse_attrs(l.as_str(), global_ids)?;
                    return Ok(());
                    // panic!()
                }
                None => {
                    break;
                }
            }
        }
        self.eval_temp_item(global_ids)?;

        // Stop eager checking, Instead of split and evaluate URL/title, first push
        // The complete string, postprocess if url doesn't exist
        let sitemapelement = match self.state {
            ParsingState::WaitingForSection => SitemapElement::Section(section::Section {
                id: rest.as_str().trim().to_string(),
                ..Default::default()
            }),
            ParsingState::ParsingSection => SitemapElement::Section(section::Section {
                id: rest.as_str().trim().to_string(),
                ..Default::default()
            }),
            ParsingState::ParsingSubsection => SitemapElement::Subsection(section::Subsection {
                id: Some(rest.as_str().trim().to_string()),
                ..Default::default()
            }),
            ParsingState::ParsingTOC => SitemapElement::TocItem(toc::TocItem {
                id: rest.as_str().trim().to_string(),
                ..Default::default()
            }),
        };
        self.temp_item = Some((sitemapelement, depth));
        Ok(())
    }

    fn eval_temp_item(
        &mut self,
        global_ids: &std::collections::HashMap<String, String>,
    ) -> Result<(), ParseError> {
        if let Some((ref toc_item, depth)) = self.temp_item {
            // Split the line by `:`. title = 0, url = Option<1>
            let resp_item = if toc_item.get_title().is_none() && toc_item.get_id().is_some() {
                // URL not defined, Try splitting the title to evaluate the URL
                let current_title = toc_item.get_id().unwrap();
                let (title, url) = match current_title.as_str().matches(':').count() {
                    1 | 0 => {
                        if let Some((first, second)) = current_title.rsplit_once(':') {
                            // Case 1: first = <Title>: second = <url>
                            // Case 2: first = <Title>: second = <id> (<url> = link to <id>)

                            match second.trim().is_empty()
                                || second.trim_end().ends_with(".html")
                                || second.contains('/')
                            {
                                // Treat second as url if it contains '/'
                                true => (
                                    Some(first.trim().to_string()),
                                    Some(second.trim().to_string()),
                                ),
                                // otherwise treat second as <id>
                                false => {
                                    let link = global_ids.get(second.trim()).ok_or_else(|| {
                                        ParseError::InvalidID {
                                            doc_id: self.doc_name.clone(),
                                            id: second.trim().to_string(),
                                        }
                                    })?;
                                    (Some(first.trim().to_string()), Some(link.to_string()))
                                }
                            }
                        } else {
                            // Case 1: current_title = <title>, <url> = None
                            // Case 2: current_title = <id>, <url> = link to <id>

                            // Try finding for link if found assign that link
                            let possible_link = global_ids.get(current_title.trim());
                            match possible_link {
                                Some(link) => (Some(current_title), Some(link.to_string())),
                                None => (Some(current_title), None),
                            }
                        }
                    }
                    _ => {
                        // The URL can have its own colons. So match the URL first
                        let url_regex = crate::http::url_regex();
                        if let Some(regex_match) = url_regex.find(current_title.as_str()) {
                            let curr_title = current_title.as_str();
                            (
                                Some(curr_title[..regex_match.start()].trim().to_string()),
                                Some(
                                    curr_title[regex_match.start()..regex_match.end()]
                                        .trim_start_matches(':')
                                        .trim()
                                        .to_string(),
                                ),
                            )
                        } else {
                            return Err(ParseError::InvalidTOCItem {
                                doc_id: self.doc_name.clone(),
                                message: "Ambiguous <title>: <URL> evaluation. Multiple colons found. Either specify the complete URL or specify the url as an attribute".to_string(),
                                row_content: current_title.as_str().to_string(),
                            });
                        }
                    }
                };

                {
                    let mut toc_item = toc_item.clone();
                    toc_item.set_id(url);
                    toc_item.set_title(title);
                    toc_item
                }
            } else {
                let id = toc_item.get_id();
                let mut toc_item = toc_item.clone();
                toc_item.set_id(id);
                toc_item
            };
            self.sections.push((resp_item, depth))
        }
        self.temp_item = None;
        Ok(())
    }

    fn parse_attrs(
        &mut self,
        line: &str,
        global_ids: &std::collections::HashMap<String, String>,
    ) -> Result<(), ParseError> {
        if line.trim().is_empty() {
            // Empty line found. Process the temp_item
            self.eval_temp_item(global_ids)?;
        } else {
            let doc_id = self.doc_name.to_string();
            match &mut self.temp_item {
                Some((i, _)) => match line.split_once(':') {
                    Some((k, v)) => {
                        let v = v.trim();
                        let id = i.get_id();
                        // TODO: Later use match
                        if k.eq("url") {
                            i.set_id(Some(v.to_string()));
                            if i.get_title().is_none() {
                                i.set_title(id);
                            }
                            i.set_path_params(v)?;
                        } else if k.eq("id") {
                            // Fetch link corresponding to the id from global_ids map
                            let link = global_ids.get(v).ok_or_else(|| ParseError::InvalidID {
                                id: v.to_string(),
                                doc_id: self.doc_name.clone(),
                            })?;
                            i.set_id(Some(link.clone()));
                            if i.get_title().is_none() {
                                i.set_title(id);
                            }
                        } else if k.eq("nav-title") {
                            i.set_nav_title(Some(v.to_string()));
                        } else if k.eq("skip") {
                            i.set_skip(v.parse::<bool>().map_err(|e| {
                                ParseError::InvalidTOCItem {
                                    doc_id,
                                    message: e.to_string(),
                                    row_content: line.to_string(),
                                }
                            })?);
                        } else if k.eq("icon") {
                            i.set_icon(Some(v.to_string()));
                        } else if k.eq("bury") {
                            i.set_bury(v.parse::<bool>().map_err(|e| {
                                ParseError::InvalidTOCItem {
                                    doc_id,
                                    message: e.to_string(),
                                    row_content: line.to_string(),
                                }
                            })?);
                        } else if k.eq("readers") {
                            i.set_readers(v);
                        } else if k.eq("writers") {
                            i.set_writers(v);
                        } else if k.eq("document") {
                            i.set_document(v);
                        } else if k.eq("confidential") {
                            i.set_confidential(v.parse::<bool>().map_err(|e| {
                                ParseError::InvalidTOCItem {
                                    doc_id,
                                    message: e.to_string(),
                                    row_content: line.to_string(),
                                }
                            })?);
                        }
                        i.insert_key_value(k, v);
                    }
                    _ => todo!(),
                },
                _ => panic!("State mismatch"),
            };
        };
        Ok(())
    }

    fn finalize(self) -> Result<Vec<(SitemapElement, usize)>, ParseError> {
        Ok(self.sections)
    }
}

impl Sitemap {
    pub async fn parse(
        s: &str,
        package: &fpm::Package,
        config: &mut fpm::Config,
        resolve_sitemap: bool,
    ) -> Result<Self, ParseError> {
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
        let mut sitemap = Sitemap {
            sections: construct_tree_util(parser.finalize()?),
            readers: vec![],
            writers: vec![],
        };

        // TODO: Need to fix it later
        // sitemap should not contain the dynamic parameters
        if sitemap.has_path_params() {
            return Err(ParseError::InvalidSitemap {
                message: "Sitemap must not contain urls with named params".to_string(),
            });
        }

        if resolve_sitemap {
            sitemap
                .resolve(package, config)
                .await
                .map_err(|e| ParseError::InvalidTOCItem {
                    doc_id: package.name.to_string(),
                    message: e.to_string(),
                    row_content: "".to_string(),
                })?;
        }
        Ok(sitemap)
    }

    async fn resolve(
        &mut self,
        package: &fpm::Package,
        config: &mut fpm::Config,
    ) -> fpm::Result<()> {
        let package_root = config.get_root_for_package(package);
        let current_package_root = config.root.to_owned();
        for section in self.sections.iter_mut() {
            resolve_section(section, &package_root, &current_package_root, config).await?;
        }
        return Ok(());

        async fn resolve_section(
            section: &mut section::Section,
            package_root: &camino::Utf8PathBuf,
            current_package_root: &camino::Utf8PathBuf,
            config: &mut fpm::Config,
        ) -> fpm::Result<()> {
            let (file_location, translation_file_location) = if let Ok(file_name) = config
                .get_file_path_and_resolve(&section.get_file_id())
                .await
            {
                (
                    Some(config.root.join(file_name.as_str())),
                    Some(config.root.join(file_name.as_str())),
                )
            } else if crate::http::url_regex()
                .find(section.get_file_id().as_str())
                .is_some()
            {
                (None, None)
            } else {
                match fpm::Config::get_file_name(
                    current_package_root,
                    section.get_file_id().as_str(),
                ) {
                    Ok(name) => {
                        if current_package_root.eq(package_root) {
                            (Some(current_package_root.join(name)), None)
                        } else {
                            (
                                Some(package_root.join(name.as_str())),
                                Some(current_package_root.join(name)),
                            )
                        }
                    }
                    Err(_) => (
                        Some(
                            package_root.join(
                                fpm::Config::get_file_name(
                                    package_root,
                                    section.get_file_id().as_str(),
                                )
                                    .map_err(|e| {
                                        fpm::Error::UsageError {
                                            message: format!(
                                                "`{}` not found, fix fpm.sitemap in FPM.ftd. Error: {:?}",
                                                section.get_file_id(), e
                                            ),
                                        }
                                    })?,
                            ),
                        ),
                        None,
                    ),
                }
            };
            section.file_location = file_location;
            section.translation_file_location = translation_file_location;

            for subsection in section.subsections.iter_mut() {
                resolve_subsection(subsection, package_root, current_package_root, config).await?;
            }
            Ok(())
        }

        async fn resolve_subsection(
            subsection: &mut section::Subsection,
            package_root: &camino::Utf8PathBuf,
            current_package_root: &camino::Utf8PathBuf,
            config: &mut fpm::Config,
        ) -> fpm::Result<()> {
            if let Some(ref id) = subsection.get_file_id() {
                let (file_location, translation_file_location) = if let Ok(file_name) =
                    config.get_file_path_and_resolve(id).await
                {
                    (
                        Some(config.root.join(file_name.as_str())),
                        Some(config.root.join(file_name.as_str())),
                    )
                } else if crate::http::url_regex().find(id.as_str()).is_some() {
                    (None, None)
                } else {
                    match fpm::Config::get_file_name(current_package_root, id.as_str()) {
                        Ok(name) => {
                            if current_package_root.eq(package_root) {
                                (Some(current_package_root.join(name)), None)
                            } else {
                                (
                                    Some(package_root.join(name.as_str())),
                                    Some(current_package_root.join(name)),
                                )
                            }
                        }
                        Err(_) => (
                            Some(package_root.join(
                                fpm::Config::get_file_name(package_root, id.as_str()).map_err(
                                    |e| fpm::Error::UsageError {
                                        message: format!(
                                            "`{}` not found, fix fpm.sitemap in FPM.ftd. Error: {:?}",
                                            id, e
                                        ),
                                    },
                                )?,
                            )),
                            None,
                        ),
                    }
                };
                subsection.file_location = file_location;
                subsection.translation_file_location = translation_file_location;
            }

            for toc in subsection.toc.iter_mut() {
                resolve_toc(toc, package_root, current_package_root, config).await?;
            }
            Ok(())
        }

        #[async_recursion::async_recursion(?Send)]
        async fn resolve_toc(
            toc: &mut toc::TocItem,
            package_root: &camino::Utf8PathBuf,
            current_package_root: &camino::Utf8PathBuf,
            config: &mut fpm::Config,
        ) -> fpm::Result<()> {
            let (file_location, translation_file_location) = if let Ok(file_name) =
                config.get_file_path_and_resolve(&toc.get_file_id()).await
            {
                (
                    Some(config.root.join(file_name.as_str())),
                    Some(config.root.join(file_name.as_str())),
                )
            } else if toc.get_file_id().trim().is_empty()
                || crate::http::url_regex()
                    .find(toc.get_file_id().as_str())
                    .is_some()
            {
                (None, None)
            } else {
                match fpm::Config::get_file_name(current_package_root, toc.get_file_id().as_str()) {
                    Ok(name) => {
                        if current_package_root.eq(package_root) {
                            (Some(current_package_root.join(name)), None)
                        } else {
                            (
                                Some(package_root.join(name.as_str())),
                                Some(current_package_root.join(name)),
                            )
                        }
                    }
                    Err(_) => (
                        Some(
                            package_root.join(
                                fpm::Config::get_file_name(
                                    package_root,
                                    toc.get_file_id().as_str(),
                                )
                                    .map_err(|e| {
                                        fpm::Error::UsageError {
                                            message: format!(
                                                "`{}` not found, fix fpm.sitemap in FPM.ftd. Error: {:?}",
                                                toc.get_file_id(), e
                                            ),
                                        }
                                    })?,
                            ),
                        ),
                        None,
                    ),
                }
            };
            toc.file_location = file_location;
            toc.translation_file_location = translation_file_location;

            for toc in toc.children.iter_mut() {
                resolve_toc(toc, package_root, current_package_root, config).await?;
            }
            Ok(())
        }
    }

    /// `get_all_locations` returns the list of tuple containing the following values:
    /// (
    ///     file_location: &camino::Utf8PathBuf, // The location of the document in the file system.
    ///                     In case of translation package, the location in the original package
    ///     translation_file_location: &Option<camino::Utf8PathBuf> // In case of the translation package,
    ///                         The location of the document in the current/translation package
    ///     url: &Option<String> // expected url for the document.
    /// )
    pub(crate) fn get_all_locations(
        &self,
    ) -> Vec<(
        &camino::Utf8PathBuf,
        &Option<camino::Utf8PathBuf>,
        Option<String>,
    )> {
        let mut locations = vec![];
        for section in self.sections.iter() {
            if let Some(ref file_location) = section.file_location {
                locations.push((
                    file_location,
                    &section.translation_file_location,
                    get_id(section.id.as_str()),
                ));
            }
            for subsection in section.subsections.iter() {
                if subsection.visible {
                    if let Some(ref file_location) = subsection.file_location {
                        locations.push((
                            file_location,
                            &subsection.translation_file_location,
                            subsection.id.as_ref().and_then(|v| get_id(v.as_str())),
                        ));
                    }
                }
                for toc in subsection.toc.iter() {
                    if let Some(ref file_location) = toc.file_location {
                        locations.push((
                            file_location,
                            &toc.translation_file_location,
                            get_id(toc.id.as_str()),
                        ));
                    }
                    locations.extend(get_toc_locations(toc));
                }
            }
        }
        return locations;

        fn get_id(id: &str) -> Option<String> {
            if id.contains("-/") {
                return Some(id.to_string());
            }
            None
        }

        fn get_toc_locations(
            toc: &toc::TocItem,
        ) -> Vec<(
            &camino::Utf8PathBuf,
            &Option<camino::Utf8PathBuf>,
            Option<String>,
        )> {
            let mut locations = vec![];
            for child in toc.children.iter() {
                if let Some(ref file_location) = child.file_location {
                    locations.push((
                        file_location,
                        &child.translation_file_location,
                        get_id(child.id.as_str()),
                    ));
                }
                locations.extend(get_toc_locations(child));
            }
            locations
        }
    }

    pub(crate) fn get_sitemap_by_id(&self, id: &str) -> Option<SiteMapCompat> {
        use itertools::Itertools;

        let mut sections = vec![];
        let mut subsections = vec![];
        let mut toc = vec![];
        let mut index = 0;
        let mut current_section = None;
        let mut current_subsection = None;
        let mut current_page = None;
        for (idx, section) in self.sections.iter().enumerate() {
            index = idx;

            if fpm::utils::ids_matches(section.id.as_str(), id) {
                subsections = section
                    .subsections
                    .iter()
                    .filter(|v| v.visible)
                    .filter(|v| {
                        let active = v
                            .get_file_id()
                            .as_ref()
                            .map(|v| fpm::utils::ids_matches(v, id))
                            .unwrap_or(false);
                        active || !v.skip
                    })
                    .map(|v| {
                        let active = v
                            .get_file_id()
                            .as_ref()
                            .map(|v| fpm::utils::ids_matches(v, id))
                            .unwrap_or(false);
                        let toc = toc::TocItemCompat::new(
                            v.id.clone(),
                            v.title.clone(),
                            active,
                            active,
                            v.readers.clone(),
                            v.writers.clone(),
                            v.icon.clone(),
                            v.bury,
                        );
                        if active {
                            let mut curr_subsection = toc.clone();
                            if let Some(ref title) = v.nav_title {
                                curr_subsection.title = Some(title.to_string());
                            }
                            current_subsection = Some(curr_subsection);
                        }
                        toc
                    })
                    .collect();

                if let Some(sub) = section
                    .subsections
                    .iter()
                    .filter(|s| !s.skip)
                    .find_or_first(|v| {
                        v.get_file_id()
                            .as_ref()
                            .map(|v| fpm::utils::ids_matches(v, id))
                            .unwrap_or(false)
                    })
                    .or_else(|| section.subsections.first())
                {
                    let (toc_list, current_toc) = get_all_toc(sub.toc.as_slice(), id);
                    toc.extend(toc_list);
                    current_page = current_toc;
                }
                let mut section_toc = toc::TocItemCompat::new(
                    get_url(section.id.as_str()),
                    section.title.clone(),
                    true,
                    true,
                    section.readers.clone(),
                    section.writers.clone(),
                    section.icon.clone(),
                    section.bury,
                );
                sections.push(section_toc.clone());
                if let Some(ref title) = section.nav_title {
                    section_toc.title = Some(title.to_string());
                }
                current_section = Some(section_toc);
                break;
            }

            if let Some((subsection_list, toc_list, curr_subsection, curr_toc)) =
                get_subsection_by_id(id, section.subsections.as_slice())
            {
                subsections.extend(subsection_list);
                toc.extend(toc_list);
                current_subsection = curr_subsection;
                current_page = curr_toc;
                let mut section_toc = toc::TocItemCompat::new(
                    get_url(section.id.as_str()),
                    section.title.clone(),
                    true,
                    true,
                    section.readers.clone(),
                    section.writers.clone(),
                    section.icon.clone(),
                    section.bury,
                );
                sections.push(section_toc.clone());
                if let Some(ref title) = section.nav_title {
                    section_toc.title = Some(title.to_string());
                }
                current_section = Some(section_toc);
                break;
            }

            if !section.skip {
                sections.push(toc::TocItemCompat::new(
                    get_url(section.id.as_str()),
                    section.title.clone(),
                    false,
                    false,
                    section.readers.clone(),
                    section.writers.clone(),
                    section.icon.clone(),
                    section.bury,
                ));
            }
        }
        sections.extend(
            self.sections[index + 1..]
                .iter()
                .filter(|s| !s.skip)
                .map(|v| {
                    toc::TocItemCompat::new(
                        get_url(v.id.as_str()),
                        v.title.clone(),
                        false,
                        false,
                        v.readers.clone(),
                        v.writers.clone(),
                        v.icon.clone(),
                        v.bury,
                    )
                }),
        );
        return Some(SiteMapCompat {
            sections,
            subsections,
            toc,
            current_section,
            current_subsection,
            current_page,
            readers: self.readers.clone(),
            writers: self.writers.clone(),
        });

        #[allow(clippy::type_complexity)]
        fn get_subsection_by_id(
            id: &str,
            subsections: &[section::Subsection],
        ) -> Option<(
            Vec<toc::TocItemCompat>,
            Vec<toc::TocItemCompat>,
            Option<toc::TocItemCompat>,
            Option<toc::TocItemCompat>,
        )> {
            let mut subsection_list = vec![];
            let mut toc = vec![];
            let mut index = 0;
            let mut found = false;
            let mut current_subsection = None;
            let mut current_page = None;

            for (idx, subsection) in subsections.iter().enumerate() {
                index = idx;
                if subsection.visible
                    && subsection
                        .id
                        .as_ref()
                        .map(|v| fpm::utils::ids_matches(v, id))
                        .unwrap_or(false)
                {
                    let (toc_list, current_toc) = get_all_toc(subsection.toc.as_slice(), id);
                    toc.extend(toc_list);
                    current_page = current_toc;
                    let mut subsection_toc = toc::TocItemCompat::new(
                        subsection.id.as_ref().and_then(|v| get_url(v.as_str())),
                        subsection.title.clone(),
                        true,
                        true,
                        subsection.readers.clone(),
                        subsection.writers.clone(),
                        subsection.icon.clone(),
                        subsection.bury,
                    );
                    subsection_list.push(subsection_toc.clone());
                    if let Some(ref title) = subsection.nav_title {
                        subsection_toc.title = Some(title.to_string());
                    }
                    current_subsection = Some(subsection_toc);
                    found = true;
                    break;
                }

                if let Some((toc_list, current_toc)) = get_toc_by_id(id, subsection.toc.as_slice())
                {
                    toc.extend(toc_list);
                    current_page = Some(current_toc);
                    if subsection.visible {
                        let mut subsection_toc = toc::TocItemCompat::new(
                            subsection.id.as_ref().and_then(|v| get_url(v.as_str())),
                            subsection.title.clone(),
                            true,
                            true,
                            subsection.readers.clone(),
                            subsection.writers.clone(),
                            subsection.icon.clone(),
                            subsection.bury,
                        );
                        subsection_list.push(subsection_toc.clone());
                        if let Some(ref title) = subsection.nav_title {
                            subsection_toc.title = Some(title.to_string());
                        }
                        current_subsection = Some(subsection_toc);
                    }
                    found = true;
                    break;
                }

                if !subsection.skip {
                    subsection_list.push(toc::TocItemCompat::new(
                        subsection.id.as_ref().and_then(|v| get_url(v.as_str())),
                        subsection.title.clone(),
                        false,
                        false,
                        subsection.readers.clone(),
                        subsection.writers.clone(),
                        subsection.icon.clone(),
                        subsection.bury,
                    ));
                }
            }

            if found {
                subsection_list.extend(subsections[index + 1..].iter().filter(|s| !s.skip).map(
                    |v| {
                        toc::TocItemCompat::new(
                            v.id.clone(),
                            v.title.clone(),
                            false,
                            false,
                            v.readers.clone(),
                            v.writers.clone(),
                            v.icon.clone(),
                            v.bury,
                        )
                    },
                ));
                return Some((subsection_list, toc, current_subsection, current_page));
            }
            None
        }

        fn get_all_toc(
            toc: &[toc::TocItem],
            id: &str,
        ) -> (Vec<toc::TocItemCompat>, Option<toc::TocItemCompat>) {
            let mut current_page = None;
            let toc = get_toc_by_id_(id, toc, &mut current_page).1;
            (toc, current_page)
        }

        fn get_toc_by_id(
            id: &str,
            toc: &[toc::TocItem],
        ) -> Option<(Vec<toc::TocItemCompat>, toc::TocItemCompat)> {
            let mut current_page = None;
            let toc_list = get_toc_by_id_(id, toc, &mut current_page).1;
            if let Some(current_page) = current_page {
                return Some((toc_list, current_page));
            }
            None
        }

        fn get_toc_by_id_(
            id: &str,
            toc: &[toc::TocItem],
            current_page: &mut Option<toc::TocItemCompat>,
        ) -> (bool, Vec<toc::TocItemCompat>) {
            let mut toc_list = vec![];
            let mut found_here = false;

            for toc_item in toc.iter() {
                let (is_open, children) =
                    get_toc_by_id_(id, toc_item.children.as_slice(), current_page);
                let is_active = fpm::utils::ids_matches(toc_item.get_file_id().as_str(), id);
                let current_toc = {
                    let mut current_toc = toc::TocItemCompat::new(
                        get_url(toc_item.id.as_str()),
                        toc_item.title.clone(),
                        is_active,
                        is_active || is_open,
                        toc_item.readers.clone(),
                        toc_item.writers.clone(),
                        toc_item.icon.clone(),
                        toc_item.bury,
                    );
                    current_toc.children = children;
                    if is_open {
                        found_here = true;
                    }
                    current_toc
                };

                if current_page.is_none() {
                    found_here = fpm::utils::ids_matches(toc_item.get_file_id().as_str(), id);
                    if found_here {
                        let mut current_toc = current_toc.clone();
                        if let Some(ref title) = toc_item.nav_title {
                            current_toc.title = Some(title.to_string());
                        }
                        *current_page = Some(current_toc);
                    }
                }

                if is_open || is_active || !toc_item.skip {
                    toc_list.push(current_toc);
                }
            }
            (found_here, toc_list)
        }

        fn get_url(id: &str) -> Option<String> {
            if id.trim().is_empty() {
                return None;
            }
            if id.eq("/") {
                return Some(id.to_string());
            }
            let id = id.trim_start_matches('/');
            if id.contains('#') {
                return Some(id.trim_end_matches('/').to_string());
            }
            if id.ends_with('/') || id.ends_with("index.html") {
                return Some(id.to_string());
            }
            Some(format!("{}/", id))
        }
    }

    pub(crate) fn get_extra_data_by_id(
        &self,
        id: &str,
    ) -> Option<std::collections::BTreeMap<String, String>> {
        for section in self.sections.iter() {
            if fpm::utils::ids_matches(section.id.as_str(), id) {
                return Some(section.extra_data.to_owned());
            }
            if let Some(data) = get_extra_data_from_subsections(id, section.subsections.as_slice())
            {
                let mut all_data = section.extra_data.clone();
                all_data.extend(data);
                return Some(all_data);
            }
        }
        return None;

        fn get_extra_data_from_subsections(
            id: &str,
            subsections: &[section::Subsection],
        ) -> Option<std::collections::BTreeMap<String, String>> {
            for subsection in subsections {
                if subsection.visible
                    && fpm::utils::ids_matches(
                        subsection.id.as_ref().unwrap_or(&"".to_string()),
                        id,
                    )
                {
                    return Some(subsection.extra_data.to_owned());
                }
                if let Some(data) = get_extra_data_from_toc(id, subsection.toc.as_slice()) {
                    let mut all_data = subsection.extra_data.clone();
                    all_data.extend(data);
                    return Some(all_data);
                }
            }
            None
        }

        fn get_extra_data_from_toc(
            id: &str,
            toc: &[toc::TocItem],
        ) -> Option<std::collections::BTreeMap<String, String>> {
            for toc_item in toc {
                if fpm::utils::ids_matches(toc_item.id.as_str(), id) {
                    return Some(toc_item.extra_data.to_owned());
                }
                if let Some(data) = get_extra_data_from_toc(id, toc_item.children.as_slice()) {
                    let mut all_data = toc_item.extra_data.clone();
                    all_data.extend(data);
                    return Some(all_data);
                }
            }
            None
        }
    }

    /// This function will return all the readers and readers which are inherited from parent

    // TODO: need to handle special reader: everyone, writer: everyone
    // readers function should return Vec<UserGroup> or Everyone
    // it return readers and `confidential`
    pub fn readers<'a>(
        &self,
        doc_path: &str,
        groups: &'a std::collections::BTreeMap<String, fpm::user_group::UserGroup>,
    ) -> (Vec<&'a fpm::user_group::UserGroup>, bool) {
        use itertools::Itertools;

        dbg!(doc_path);
        dbg!(groups);
        dbg!(&self.sections);

        for section in self.sections.iter() {
            let (readers, confidential) = find_section(section, doc_path);
            if readers.is_empty() {
                continue;
            }
            let readers: Vec<String> = self.readers.iter().cloned().chain(readers).collect();
            return (
                readers
                    .iter()
                    .unique()
                    .filter_map(|g| groups.get(g))
                    .chain(self.writers(doc_path, groups))
                    .collect(),
                confidential,
            );
        }

        return (vec![], true);

        fn find_toc(toc: &toc::TocItem, doc_path: &str) -> (Vec<String>, bool) {
            if toc.id.eq(doc_path) {
                return (toc.readers.clone(), toc.confidential);
            }

            for child in toc.children.iter() {
                let (readers, mut confidential) = find_toc(child, doc_path);
                if readers.is_empty() {
                    continue;
                }
                // Inherit from the parent, if parent is not confidential assuming all the children
                // are not confidential
                if !toc.confidential {
                    confidential = false;
                }

                return (
                    readers
                        .into_iter()
                        .chain(toc.readers.iter().cloned())
                        .collect(),
                    confidential,
                );
            }
            (vec![], true)
        }

        fn find_subsection(
            subsection: &section::Subsection,
            doc_path: &str,
        ) -> (Vec<String>, bool) {
            if subsection
                .id
                .as_ref()
                .map(|id| id.eq(doc_path))
                .unwrap_or(false)
            {
                return (subsection.readers.clone(), subsection.confidential);
            }

            for toc in subsection.toc.iter() {
                let (readers, mut confidential) = find_toc(toc, doc_path);
                if readers.is_empty() {
                    continue;
                }

                // Inherit from the parent, if parent is not confidential assuming all the children
                // are not confidential
                if !subsection.confidential {
                    confidential = false;
                }

                return (
                    readers
                        .into_iter()
                        .chain(subsection.readers.iter().cloned())
                        .collect(),
                    confidential,
                );
            }
            (vec![], true)
        }

        fn find_section(section: &section::Section, doc_path: &str) -> (Vec<String>, bool) {
            if section.id.eq(doc_path) {
                return (section.readers.clone(), section.confidential);
            }

            for subsection in section.subsections.iter() {
                let (readers, mut confidential) = find_subsection(subsection, doc_path);
                if readers.is_empty() {
                    continue;
                }
                // Inherit from the parent, if parent is not confidential assuming all the children
                // are not confidential
                if !section.confidential {
                    confidential = false;
                }
                return (
                    readers
                        .into_iter()
                        .chain(section.readers.iter().cloned())
                        .collect(),
                    confidential,
                );
            }
            (vec![], true)
        }
    }

    pub fn writers<'a>(
        &self,
        doc_path: &str,
        groups: &'a std::collections::BTreeMap<String, fpm::user_group::UserGroup>,
    ) -> Vec<&'a fpm::user_group::UserGroup> {
        use itertools::Itertools;

        for section in self.sections.iter() {
            let writers = find_section(section, doc_path);
            if writers.is_empty() {
                continue;
            }
            let writers: Vec<String> = self.writers.iter().cloned().chain(writers).collect();
            return writers
                .iter()
                .unique()
                .filter_map(|g| groups.get(g))
                .collect();
        }

        return vec![];

        fn find_toc(toc: &toc::TocItem, doc_path: &str) -> Vec<String> {
            if toc.id.eq(doc_path) {
                return toc.writers.clone();
            }

            for child in toc.children.iter() {
                let writers = find_toc(child, doc_path);
                if writers.is_empty() {
                    continue;
                }
                return writers
                    .into_iter()
                    .chain(toc.writers.iter().cloned())
                    .collect();
            }
            vec![]
        }

        fn find_subsection(subsection: &section::Subsection, doc_path: &str) -> Vec<String> {
            if subsection
                .id
                .as_ref()
                .map(|id| id.eq(doc_path))
                .unwrap_or(false)
            {
                return subsection.writers.clone();
            }

            for toc in subsection.toc.iter() {
                let writers = find_toc(toc, doc_path);
                if writers.is_empty() {
                    continue;
                }
                return writers
                    .into_iter()
                    .chain(subsection.writers.iter().cloned())
                    .collect();
            }
            vec![]
        }

        fn find_section(section: &section::Section, doc_path: &str) -> Vec<String> {
            if section.id.eq(doc_path) {
                return section.writers.clone();
            }

            for subsection in section.subsections.iter() {
                let writers = find_subsection(subsection, doc_path);
                if writers.is_empty() {
                    continue;
                }
                return writers
                    .into_iter()
                    .chain(section.writers.iter().cloned())
                    .collect();
            }
            vec![]
        }
    }

    /// path: foo/temp/
    /// path: /
    /// This function can be used for if path exists in sitemap or not
    #[tracing::instrument(name = "sitemap-resolve-document", skip_all)]
    pub fn resolve_document(&self, path: &str) -> Option<String> {
        tracing::info!(path = path);
        fn resolve_in_toc(toc: &toc::TocItem, path: &str) -> Option<String> {
            if fpm::utils::ids_matches(toc.id.as_str(), path) {
                return toc.document.clone();
            }

            for child in toc.children.iter() {
                let document = resolve_in_toc(child, path);
                if document.is_some() {
                    return document;
                }
            }
            None
        }

        fn resolve_in_sub_section(sub_section: &section::Subsection, path: &str) -> Option<String> {
            if let Some(id) = sub_section.id.as_ref() {
                if fpm::utils::ids_matches(path, id.as_str()) {
                    return sub_section.document.clone();
                }
            }

            for toc in sub_section.toc.iter() {
                let document = resolve_in_toc(toc, path);
                if document.is_some() {
                    return document;
                }
            }

            None
        }

        fn resolve_in_section(section: &section::Section, path: &str) -> Option<String> {
            if fpm::utils::ids_matches(section.id.as_str(), path) {
                return section.document.clone();
            }

            for subsection in section.subsections.iter() {
                let document = resolve_in_sub_section(subsection, path);
                if document.is_some() {
                    return document;
                }
            }
            None
        }

        for section in self.sections.iter() {
            let document = resolve_in_section(section, path);
            if document.is_some() {
                tracing::info!(msg = "return: document found", path = path);
                return document;
            }
        }

        tracing::info!(msg = "return: document not found", path = path);

        None
    }

    pub fn has_path_params(&self) -> bool {
        section::Section::contains_named_params(&self.sections)
    }
}

#[derive(Debug)]
struct LevelTree {
    level: usize,
    item: toc::TocItem,
}

impl LevelTree {
    fn new(level: usize, item: toc::TocItem) -> Self {
        Self { level, item }
    }
}

fn construct_tree_util(mut elements: Vec<(SitemapElement, usize)>) -> Vec<section::Section> {
    let mut sections = vec![];
    elements.reverse();
    construct_tree_util_(elements, &mut sections);
    return sections;

    fn construct_tree_util_(
        mut elements: Vec<(SitemapElement, usize)>,
        sections: &mut Vec<section::Section>,
    ) {
        if elements.is_empty() {
            return;
        }
        let smallest_level = elements.last().unwrap().1;
        while let Some((SitemapElement::Section(section), _)) = elements.last() {
            sections.push(section.to_owned());
            elements.pop();
        }

        let last_section = if let Some(section) = sections.last_mut() {
            section
        } else {
            // todo: return an error
            return;
        };
        while let Some((SitemapElement::Subsection(subsection), _)) = elements.last() {
            last_section.subsections.push(subsection.to_owned());
            elements.pop();
        }

        let last_subsection = if let Some(subsection) = last_section.subsections.last_mut() {
            subsection
        } else {
            last_section.subsections.push(section::Subsection {
                visible: false,
                ..Default::default()
            });
            last_section.subsections.last_mut().unwrap()
        };

        let mut toc_items: Vec<(toc::TocItem, usize)> = vec![];
        while let Some((SitemapElement::TocItem(toc), level)) = elements.last() {
            toc_items.push((toc.to_owned(), level.to_owned()));
            elements.pop();
        }
        toc_items.push((toc::TocItem::default(), smallest_level));
        // println!("Elements: {:#?}", elements);
        let mut tree = construct_tree(toc_items, smallest_level);
        let _garbage = tree.pop();
        last_subsection.toc.extend(
            tree.into_iter()
                .map(|x| x.item)
                .collect::<Vec<toc::TocItem>>(),
        );

        construct_tree_util_(elements, sections);
    }
}

fn get_top_level(stack: &[LevelTree]) -> usize {
    stack.last().map(|x| x.level).unwrap()
}

fn construct_tree(elements: Vec<(toc::TocItem, usize)>, smallest_level: usize) -> Vec<LevelTree> {
    let mut stack_tree = vec![];
    for (toc_item, level) in elements.into_iter() {
        if level < smallest_level {
            panic!("Level should not be lesser than smallest level");
        }
        if !(stack_tree.is_empty() || get_top_level(&stack_tree) <= level) {
            let top = stack_tree.pop().unwrap();
            let mut top_level = top.level;
            let mut children = vec![top];
            while level < top_level {
                loop {
                    if stack_tree.is_empty() {
                        panic!("Tree should not be empty here")
                    }
                    let mut cur_element = stack_tree.pop().unwrap();
                    if stack_tree.is_empty() || cur_element.level < top_level {
                        // Means found children's parent, needs to append children to its parents
                        // and update top level accordingly
                        // parent level should equal to top_level - 1
                        assert_eq!(cur_element.level as i32, (top_level as i32) - 1);
                        cur_element
                            .item
                            .children
                            .append(&mut children.into_iter().rev().map(|x| x.item).collect());
                        top_level = cur_element.level;
                        children = vec![];
                        stack_tree.push(cur_element);
                        break;
                    } else if cur_element.level == top_level {
                        // if popped element is same as already popped element it is adjacent
                        // element, needs to push into children and find parent in stack
                        children.push(cur_element);
                    } else {
                        panic!(
                            "Stacked elements level should never be greater than top element level"
                        );
                    }
                }
            }
            assert!(level >= top_level);
        }
        let node = LevelTree::new(level, toc_item);

        stack_tree.push(node);
    }
    stack_tree
}

pub fn resolve(
    package: &fpm::Package,
    path: &str,
) -> fpm::Result<fpm::sitemap::dynamic_urls::ResolveDocOutput> {
    // resolve in sitemap
    if let Some(sitemap) = package.sitemap.as_ref() {
        if let Some(document) = sitemap.resolve_document(path) {
            return Ok((Some(document), vec![]));
        }
    };

    // resolve in dynamic-urls
    if let Some(dynamic_urls) = package.dynamic_urls.as_ref() {
        return dynamic_urls.resolve_document(path);
    };

    Ok((None, vec![]))
}
