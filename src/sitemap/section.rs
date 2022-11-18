#[derive(Debug, Clone, Default, PartialEq)]
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
    // TODO: It should be ftd::ImageSrc
    pub icon: Option<String>,

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
    ///
    /// In case of translation package, it stores the location in original
    /// package
    /// It is an optional field as the id provided could be an url to a website.
    /// Eg:
    /// ```ftd
    /// # Fifthtry: https://fifthtry.com/
    /// ````
    /// In that case it store `None`
    pub file_location: Option<camino::Utf8PathBuf>,

    /// `translation_file_location` has value in case of translation package.
    /// It stores the location of the document in the
    /// file system in the translation package.
    pub translation_file_location: Option<camino::Utf8PathBuf>,

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
    //    pub extra_data: Vec<(String, String)>,
    pub extra_data: std::collections::BTreeMap<String, String>,
    pub is_active: bool,
    pub nav_title: Option<String>,
    pub subsections: Vec<fpm::sitemap::section::Subsection>,

    /// `skip` is used for skipping the section from sitemap processor
    /// Example:
    ///
    /// ```ftd
    ///
    /// # foo: /
    /// skip: true
    ///
    /// ```
    /// default value will be `false`
    pub skip: bool,
    /// if provided `document` is confidential or not.
    /// `confidential:true` means totally confidential
    /// `confidential:false` can be seen some it's data
    pub confidential: bool,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
    /// In FPM.ftd sitemap, we can use `document` for section, subsection and toc.
    /// # Section: /books/
    ///   document: /books/python/
    pub document: Option<String>,
    /// If we can define dynamic `url` in section, subsection and toc in `dynamic-urls`.
    /// `url: /books/<string:book_name>/<integer:price>/`
    /// here book_name and price are path parameters
    /// path_parameters: [(string, book_name), (integer, price)]
    pub path_parameters: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Subsection {
    pub id: Option<String>,
    pub icon: Option<String>,
    pub title: Option<String>,
    pub file_location: Option<camino::Utf8PathBuf>,
    pub translation_file_location: Option<camino::Utf8PathBuf>,
    pub visible: bool,
    pub extra_data: std::collections::BTreeMap<String, String>,
    pub is_active: bool,
    pub nav_title: Option<String>,
    pub toc: Vec<fpm::sitemap::toc::TocItem>,
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
    pub path_parameters: Vec<(String, String)>,
}

impl Section {
    pub fn path_exists(&self, path: &str) -> bool {
        if fpm::utils::ids_matches(self.id.as_str(), path) {
            return true;
        }

        for subsection in self.subsections.iter() {
            if subsection.path_exists(path) {
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

    // return true if any item in sitemap does contain path_params
    pub fn contains_path_params(sections: &[Section]) -> bool {
        fn check_toc(toc: &fpm::sitemap::toc::TocItem) -> bool {
            if !toc.path_parameters.is_empty() {
                return true;
            }

            for toc in toc.children.iter() {
                if check_toc(toc) {
                    return true;
                }
            }
            false
        }

        fn check_sub_section(sub_section: &Subsection) -> bool {
            if !sub_section.path_parameters.is_empty() {
                return true;
            }

            for toc in sub_section.toc.iter() {
                if check_toc(toc) {
                    return true;
                }
            }
            false
        }

        fn check_section(section: &Section) -> bool {
            if !section.path_parameters.is_empty() {
                return true;
            }

            for sub_section in section.subsections.iter() {
                if check_sub_section(sub_section) {
                    return true;
                }
            }
            false
        }

        for section in sections.iter() {
            if check_section(section) {
                return true;
            }
        }
        false
    }
}

impl Default for Subsection {
    fn default() -> Self {
        Subsection {
            id: None,
            title: None,
            icon: None,
            file_location: Default::default(),
            translation_file_location: None,
            visible: true,
            extra_data: Default::default(),
            is_active: false,
            nav_title: None,
            toc: vec![],
            skip: false,
            readers: vec![],
            writers: vec![],
            document: None,
            path_parameters: vec![],
            confidential: false,
        }
    }
}

impl Subsection {
    /// path: /foo/demo/
    /// path: /
    fn path_exists(&self, path: &str) -> bool {
        if let Some(id) = self.id.as_ref() {
            if fpm::utils::ids_matches(path, id.as_str()) {
                return true;
            }
        }

        for toc in self.toc.iter() {
            if toc.path_exists(path) {
                return true;
            }
        }

        false
    }

    /// returns the file id portion of the url only in case
    /// any component id is referred in the url itself
    pub fn get_file_id(&self) -> Option<String> {
        self.id
            .as_ref()
            .map(|id| id.rsplit_once('#').map(|s| s.0).unwrap_or(id).to_string())
    }
}
