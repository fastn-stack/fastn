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
#[derive(Debug, Clone, Default)]
pub struct Sitemap {
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Default)]
pub struct Section {
    /// `id` is the document id (or url) provided in the section
    /// Example:
    ///
    /// ```ftd
    ///
    /// # foo/
    ///
    /// ```
    ///
    /// Here foo/ is store as `id`
    pub id: String,

    /// `title` contains the title of the document. This can be specified inside
    /// document itself.
    ///
    /// Example: In the foo.ftd document
    ///
    /// ```ftd
    /// -- fpm.info DOCUMENT_INFO:
    /// title: Foo Title
    /// ```
    ///
    /// In above example the `title` stores `Foo Title`.
    ///
    /// In the case where the title is not defined as above, the title would be
    /// according to heading priority
    ///
    /// Example: In the foo.ftd document
    ///
    /// ```ftd
    ///
    /// -- ft.h0: Foo Heading Title
    /// ```
    /// In above example, the `title` stores `Foo Heading Title`.
    pub title: Option<String>,

    /// `file_location` stores the location of the document in the
    /// file system
    pub file_location: camino::Utf8PathBuf,

    /// `base` stores the location of the package in which the document
    /// exists
    pub base: camino::Utf8PathBuf,

    /// `extra_data` stores the key value data provided in the section.
    /// This is passed as context and consumes by processors like `get-data`.
    ///
    /// Example:
    ///
    /// In `FPM.ftd`
    ///
    /// ```fpm
    /// -- fpm.sitemap:
    ///
    /// \# foo/
    /// show: true
    /// message: Hello World
    /// ```
    ///
    /// In `foo.ftd`
    ///
    /// ```ftd
    ///
    /// -- boolean show:
    /// $processor$: get-data
    ///
    /// -- string message:
    /// $processor$: get-data
    /// ```
    ///
    /// The above example injects the value `true` and `Hello World`
    /// to the variables `show` and `message` respectively in foo.ftd
    /// and then renders it.
    pub extra_data: std::collections::BTreeMap<String, String>,
    pub is_active: bool,
    pub subsections: Vec<Subsection>,
}

#[derive(Debug, Clone)]
pub struct Subsection {
    pub id: Option<String>,
    pub title: Option<String>,
    pub file_location: camino::Utf8PathBuf,
    pub base: camino::Utf8PathBuf,
    pub visible: bool,
    pub extra_data: std::collections::BTreeMap<String, String>,
    pub is_active: bool,
    pub toc: Vec<TocItem>,
}

impl Default for Subsection {
    fn default() -> Self {
        Subsection {
            id: None,
            title: None,
            file_location: Default::default(),
            base: Default::default(),
            visible: true,
            extra_data: Default::default(),
            is_active: false,
            toc: vec![],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TocItem {
    pub id: String,
    pub title: Option<String>,
    pub file_location: camino::Utf8PathBuf,
    pub base: camino::Utf8PathBuf,
    pub extra_data: std::collections::BTreeMap<String, String>,
    pub is_active: bool,
    pub children: Vec<TocItem>,
}
#[derive(Debug, Default, serde::Serialize)]
pub struct SiteMapCompat {
    pub sections: Vec<TocItemCompat>,
    pub subsections: Vec<TocItemCompat>,
    pub toc: Vec<TocItemCompat>,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct TocItemCompat {
    pub url: Option<String>,
    pub number: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "is-heading")]
    pub is_heading: bool,
    // TODO: Font icon mapping to html?
    #[serde(rename = "font-icon")]
    pub font_icon: Option<String>,
    #[serde(rename = "is-disabled")]
    pub is_disabled: bool,
    #[serde(rename = "is-active")]
    pub is_active: bool,
    #[serde(rename = "img-src")]
    pub image_src: Option<String>,
    pub children: Vec<TocItemCompat>,
}

impl TocItemCompat {
    fn new(url: Option<String>, title: Option<String>, is_active: bool) -> TocItemCompat {
        TocItemCompat {
            url,
            number: None,
            title,
            is_heading: false,
            font_icon: None,
            is_disabled: false,
            is_active,
            image_src: None,
            children: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum SitemapElement {
    Section(Section),
    Subsection(Subsection),
    TocItem(TocItem),
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
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("{doc_id} -> {message} -> Row Content: {row_content}")]
    InvalidTOCItem {
        doc_id: String,
        message: String,
        row_content: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
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

impl SitemapParser {
    pub fn read_line(&mut self, line: &str) -> Result<(), ParseError> {
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
                    self.read_attrs(l.as_str())?;
                    return Ok(());
                    // panic!()
                }
                None => {
                    break;
                }
            }
        }
        self.eval_temp_item()?;

        // Stop eager checking, Instead of split and evaluate URL/title, first push
        // The complete string, postprocess if url doesn't exist
        let sitemapelement = match self.state {
            ParsingState::WaitingForSection => SitemapElement::Section(Section {
                id: rest.as_str().trim().to_string(),
                ..Default::default()
            }),
            ParsingState::ParsingSection => SitemapElement::Section(Section {
                id: rest.as_str().trim().to_string(),
                ..Default::default()
            }),
            ParsingState::ParsingSubsection => SitemapElement::Subsection(Subsection {
                id: Some(rest.as_str().trim().to_string()),
                ..Default::default()
            }),
            ParsingState::ParsingTOC => SitemapElement::TocItem(TocItem {
                id: rest.as_str().trim().to_string(),
                ..Default::default()
            }),
        };
        self.temp_item = Some((sitemapelement, depth));
        Ok(())
    }

    fn eval_temp_item(&mut self) -> Result<(), ParseError> {
        if let Some((ref toc_item, depth)) = self.temp_item {
            // Split the line by `:`. title = 0, url = Option<1>
            let resp_item = if toc_item.get_title().is_none() && toc_item.get_id().is_some() {
                // URL not defined, Try splitting the title to evaluate the URL
                let current_title = toc_item.get_id().clone().unwrap();
                let (title, url) = match current_title.as_str().matches(':').count() {
                    1 | 0 => {
                        if let Some((first, second)) = current_title.rsplit_once(":") {
                            (
                                Some(first.trim().to_string()),
                                Some(second.trim().to_string()),
                            )
                        } else {
                            // No matches, i.e. return the current string as title, url as none
                            (Some(current_title), None)
                        }
                    }
                    _ => {
                        // The URL can have its own colons. So match the URL first
                        let url_regex = regex::Regex::new(
                            r#":[ ]?(?P<url>(?:https?)?://(?:[a-zA-Z0-9]+\.)?(?:[A-z0-9]+\.)(?:[A-z0-9]+)(?:[/A-Za-z0-9\?:\&%]+))"#
                        ).unwrap();
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
                toc_item.clone()
            };
            self.sections.push((resp_item, depth))
        }
        self.temp_item = None;
        Ok(())
    }
    fn read_attrs(&mut self, line: &str) -> Result<(), ParseError> {
        if line.trim().is_empty() {
            // Empty line found. Process the temp_item
            self.eval_temp_item()?;
        } else {
            // let id = self.temp_item.unwrap().0.get_id();
            match &mut self.temp_item {
                Some((i, _)) => match line.split_once(":") {
                    Some((k, v)) => {
                        let id = i.get_id();
                        if k.eq("url") {
                            i.set_id(Some(v.to_string()));
                            if i.get_title().is_none() {
                                i.set_title(id);
                            }
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
    pub fn parse(
        s: &str,
        package: &fpm::Package,
        config: &fpm::Config,
    ) -> Result<Self, ParseError> {
        let mut parser = SitemapParser {
            state: ParsingState::WaitingForSection,
            sections: vec![],
            temp_item: None,
            doc_name: package.name.to_string(),
        };
        for line in s.split('\n') {
            parser.read_line(line)?;
        }
        if parser.temp_item.is_some() {
            parser.eval_temp_item()?;
        }
        let mut sitemap = Sitemap {
            sections: construct_tree_util(parser.finalize()?),
        };

        sitemap
            .resolve(package, config)
            .map_err(|e| ParseError::InvalidTOCItem {
                doc_id: package.name.to_string(),
                message: e.to_string(),
                row_content: "".to_string(),
            })?;

        Ok(sitemap)
    }

    fn resolve(&mut self, package: &fpm::Package, config: &fpm::Config) -> fpm::Result<()> {
        let package_root = config.get_root_for_package(package);
        let current_package_root = config.root.to_owned();
        for section in self.sections.iter_mut() {
            resolve_section(section, &package_root, &current_package_root)?;
        }
        return Ok(());

        fn resolve_section(
            section: &mut fpm::sitemap::Section,
            package_root: &camino::Utf8PathBuf,
            current_package_root: &camino::Utf8PathBuf,
        ) -> fpm::Result<()> {
            let file_path =
                match fpm::Config::get_file_name(current_package_root, section.id.as_str()) {
                    Ok(name) => current_package_root.join(name),
                    Err(_) => package_root.join(
                        fpm::Config::get_file_name(package_root, section.id.as_str()).map_err(
                            |e| fpm::Error::UsageError {
                                message: format!(
                                    "`{}` not found, fix fpm.sitemap in FPM.ftd. Error: {:?}",
                                    section.id, e
                                ),
                            },
                        )?,
                    ),
                };
            section.file_location = file_path;

            for subsection in section.subsections.iter_mut() {
                resolve_subsection(subsection, package_root, current_package_root)?;
            }
            Ok(())
        }

        fn resolve_subsection(
            subsection: &mut fpm::sitemap::Subsection,
            package_root: &camino::Utf8PathBuf,
            current_package_root: &camino::Utf8PathBuf,
        ) -> fpm::Result<()> {
            if let Some(ref id) = subsection.id {
                let file_path = match fpm::Config::get_file_name(current_package_root, id) {
                    Ok(name) => current_package_root.join(name),
                    Err(_) => {
                        package_root.join(fpm::Config::get_file_name(package_root, id).map_err(
                            |e| fpm::Error::UsageError {
                                message: format!(
                                    "`{}` not found, fix fpm.sitemap in FPM.ftd. Error: {:?}",
                                    id, e
                                ),
                            },
                        )?)
                    }
                };
                subsection.file_location = file_path;
            }

            for toc in subsection.toc.iter_mut() {
                resolve_toc(toc, package_root, current_package_root)?;
            }
            Ok(())
        }

        fn resolve_toc(
            toc: &mut fpm::sitemap::TocItem,
            package_root: &camino::Utf8PathBuf,
            current_package_root: &camino::Utf8PathBuf,
        ) -> fpm::Result<()> {
            let file_path = match fpm::Config::get_file_name(current_package_root, toc.id.as_str())
            {
                Ok(name) => current_package_root.join(name),
                Err(_) => package_root.join(
                    fpm::Config::get_file_name(package_root, toc.id.as_str()).map_err(|e| {
                        fpm::Error::UsageError {
                            message: format!(
                                "`{}` not found, fix fpm.sitemap in FPM.ftd. Error: {:?}",
                                toc.id, e
                            ),
                        }
                    })?,
                ),
            };
            toc.file_location = file_path;

            for toc in toc.children.iter_mut() {
                resolve_toc(toc, package_root, current_package_root)?;
            }
            Ok(())
        }
    }

    /// `get_all_locations` returns the list of tuple containing the following values:
    /// (
    ///     file_location: &camino::Utf8PathBuf, // The location of the document in the file system
    ///     base: &camino::Utf8PathBuf // The base location of the fpm package where the document exists
    /// )
    pub(crate) fn get_all_locations<'a>(
        &'a self,
    ) -> Vec<(
        &'a camino::Utf8PathBuf,
        &'a camino::Utf8PathBuf,
        Option<String>,
    )> {
        let mut locations = vec![];
        for section in self.sections.iter() {
            locations.push((
                &section.file_location,
                &section.base,
                get_id(section.id.as_str()),
            ));
            for subsection in section.subsections.iter() {
                if subsection.visible {
                    locations.push((
                        &subsection.file_location,
                        &subsection.base,
                        subsection.id.as_ref().map(|v| get_id(v.as_str())).flatten(),
                    ));
                }
                for toc in subsection.toc.iter() {
                    locations.push((&toc.file_location, &toc.base, get_id(toc.id.as_str())));
                    locations.extend(get_toc_locations(&toc));
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
            toc: &fpm::sitemap::TocItem,
        ) -> Vec<(&camino::Utf8PathBuf, &camino::Utf8PathBuf, Option<String>)> {
            let mut locations = vec![];
            for child in toc.children.iter() {
                locations.push((&child.file_location, &child.base, get_id(child.id.as_str())));
                locations.extend(get_toc_locations(&child));
            }
            locations
        }
    }

    pub(crate) fn get_sitemap_by_id(&self, id: &str) -> Option<SiteMapCompat> {
        let mut sections = vec![];
        let mut subsections = vec![];
        let mut toc = vec![];
        let mut index = 0;
        let mut found = false;
        for (idx, section) in self.sections.iter().enumerate() {
            index = idx;
            if section.id.as_str().eq(id) {
                subsections = section
                    .subsections
                    .iter()
                    .filter(|v| v.visible)
                    .map(|v| TocItemCompat::new(v.id.clone(), v.title.clone(), false))
                    .collect();
                if let Some(sub) = section.subsections.first() {
                    toc = get_all_toc(&sub.toc);
                }
                sections.push(TocItemCompat::new(
                    Some(section.id.to_string()),
                    section.title.clone(),
                    true,
                ));
                found = true;
                break;
            }

            if let Some((subsection_list, toc_list)) =
                get_subsection_by_id(id, &section.subsections)
            {
                subsections.extend(subsection_list);
                toc.extend(toc_list);
                sections.push(TocItemCompat::new(
                    Some(section.id.to_string()),
                    section.title.clone(),
                    true,
                ));
                found = true;
                break;
            }

            sections.push(TocItemCompat::new(
                Some(section.id.to_string()),
                section.title.clone(),
                false,
            ));
        }
        if found {
            sections.extend(
                self.sections[index + 1..]
                    .iter()
                    .map(|v| TocItemCompat::new(Some(v.id.to_string()), v.title.clone(), false)),
            );
            return Some(SiteMapCompat {
                sections,
                subsections,
                toc,
            });
        }

        return None;

        fn get_all_toc(toc: &Vec<TocItem>) -> Vec<TocItemCompat> {
            toc.iter()
                .map(|v| {
                    let mut toc = TocItemCompat::new(Some(v.id.clone()), v.title.clone(), false);
                    toc.children = get_all_toc(&v.children);
                    toc
                })
                .collect()
        }

        fn get_subsection_by_id(
            id: &str,
            subsections: &Vec<Subsection>,
        ) -> Option<(Vec<TocItemCompat>, Vec<TocItemCompat>)> {
            let mut subsection_list = vec![];
            let mut toc = vec![];
            let mut index = 0;
            let mut found = false;
            for (idx, subsection) in subsections.iter().enumerate() {
                index = idx;
                if subsection.visible && subsection.id.as_ref().map(|v| v.eq(id)).unwrap_or(false) {
                    toc = get_all_toc(&subsection.toc);
                    subsection_list.push(TocItemCompat::new(
                        subsection.id.clone(),
                        subsection.title.clone(),
                        true,
                    ));
                    found = true;
                    break;
                }

                if let Some(toc_list) = get_toc_by_id(id, &subsection.toc) {
                    toc.extend(toc_list);
                    subsection_list.push(TocItemCompat::new(
                        subsection.id.clone(),
                        subsection.title.clone(),
                        true,
                    ));
                    found = true;
                    break;
                }

                subsection_list.push(TocItemCompat::new(
                    subsection.id.clone(),
                    subsection.title.clone(),
                    false,
                ));
            }
            if found {
                subsection_list.extend(
                    subsections[index + 1..]
                        .iter()
                        .map(|v| TocItemCompat::new(v.id.clone(), v.title.clone(), false)),
                );
                return Some((subsection_list, toc));
            }
            None
        }

        fn get_toc_by_id(id: &str, toc: &Vec<TocItem>) -> Option<Vec<TocItemCompat>> {
            let mut found = false;
            let toc_list = get_toc_by_id_(id, toc, &mut found).1;
            if found {
                return Some(toc_list);
            }
            return None;

            fn get_toc_by_id_(
                id: &str,
                toc: &Vec<TocItem>,
                found: &mut bool,
            ) -> (bool, Vec<TocItemCompat>) {
                let mut toc_list = vec![];
                let mut found_here = false;
                for toc_item in toc.iter() {
                    toc_list.push({
                        let (is_active, children) = get_toc_by_id_(id, &toc_item.children, found);
                        let mut current_toc = TocItemCompat::new(
                            Some(toc_item.id.to_string()),
                            toc_item.title.clone(),
                            toc_item.id.as_str().eq(id) || is_active,
                        );
                        current_toc.children = children;
                        if is_active {
                            found_here = true;
                        }
                        current_toc
                    });
                    if !*found {
                        *found = toc_item.id.as_str().eq(id);
                        found_here = *found;
                    }
                }
                (found_here, toc_list)
            }
        }
    }

    pub(crate) fn get_extra_data_by_id(
        &self,
        id: &str,
    ) -> Option<std::collections::BTreeMap<String, String>> {
        for section in self.sections.iter() {
            if section.id.as_str().eq(id) {
                return Some(section.extra_data.to_owned());
            }
            if let Some(data) = get_extra_data_from_subsections(id, &section.subsections) {
                let mut all_data = section.extra_data.clone();
                all_data.extend(data);
                return Some(all_data);
            }
        }
        return None;

        fn get_extra_data_from_subsections(
            id: &str,
            subsections: &Vec<Subsection>,
        ) -> Option<std::collections::BTreeMap<String, String>> {
            for subsection in subsections {
                if subsection.visible && subsection.id.as_ref().unwrap_or(&"".to_string()).eq(id) {
                    return Some(subsection.extra_data.to_owned());
                }
                if let Some(data) = get_extra_data_from_toc(id, &subsection.toc) {
                    let mut all_data = subsection.extra_data.clone();
                    all_data.extend(data);
                    return Some(all_data);
                }
            }
            None
        }

        fn get_extra_data_from_toc(
            id: &str,
            toc: &Vec<TocItem>,
        ) -> Option<std::collections::BTreeMap<String, String>> {
            for toc_item in toc {
                if toc_item.id.as_str().eq(id) {
                    return Some(toc_item.extra_data.to_owned());
                }
                if let Some(data) = get_extra_data_from_toc(id, &toc_item.children) {
                    let mut all_data = toc_item.extra_data.clone();
                    all_data.extend(data);
                    return Some(all_data);
                }
            }
            None
        }
    }
}

#[derive(Debug)]
struct LevelTree {
    level: usize,
    item: TocItem,
}

impl LevelTree {
    fn new(level: usize, item: TocItem) -> Self {
        Self { level, item }
    }
}

fn construct_tree_util(mut elements: Vec<(SitemapElement, usize)>) -> Vec<Section> {
    let mut sections = vec![];
    elements.reverse();
    construct_tree_util_(elements, &mut sections);
    return sections;

    fn construct_tree_util_(
        mut elements: Vec<(SitemapElement, usize)>,
        sections: &mut Vec<Section>,
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
            last_section.subsections.push(Subsection {
                visible: false,
                ..Default::default()
            });
            last_section.subsections.last_mut().unwrap()
        };

        let mut toc_items: Vec<(TocItem, usize)> = vec![];
        while let Some((SitemapElement::TocItem(toc), level)) = elements.last() {
            toc_items.push((toc.to_owned(), level.to_owned()));
            elements.pop();
        }
        toc_items.push((TocItem::default(), smallest_level));
        // println!("Elements: {:#?}", elements);
        let mut tree = construct_tree(toc_items, smallest_level);
        let _garbage = tree.pop();
        last_subsection
            .toc
            .extend(tree.into_iter().map(|x| x.item).collect::<Vec<TocItem>>());

        construct_tree_util_(elements, sections);
    }
}

fn get_top_level(stack: &[LevelTree]) -> usize {
    stack.last().map(|x| x.level).unwrap()
}

fn construct_tree(elements: Vec<(TocItem, usize)>, smallest_level: usize) -> Vec<LevelTree> {
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
