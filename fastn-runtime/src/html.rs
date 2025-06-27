use fastn_runtime::extensions::*;

pub struct HtmlData {
    pub package: Package,
    pub js: String,
    pub css_files: Vec<String>,
    pub js_files: Vec<String>,
    pub doc: Box<dyn fastn_resolved::tdoc::TDoc>,
    pub has_rive_component: bool,
}
const EMPTY_HTML_BODY: &str = "<body></body><style id=\"styles\"></style>";

impl HtmlData {
    pub fn from_cd(o: fastn_resolved::CompiledDocument) -> fastn_runtime::HtmlData {
        let doc = fastn_runtime::TDoc {
            name: "foo", // Todo: Package name
            definitions: o.definitions,
        };

        let output = fastn_runtime::get_all_asts(&doc, &o.content);

        let js_document_script = fastn_js::to_js(output.ast.as_slice(), "foo");
        let js_ftd_script = fastn_js::to_js(
            fastn_runtime::default_bag_into_js_ast(&doc).as_slice(),
            "foo",
        );
        let js = format!("{js_ftd_script}\n{js_document_script}");
        fastn_runtime::HtmlData {
            package: fastn_runtime::Package::new_name("foo"), // Todo
            js,
            css_files: vec![],
            js_files: vec![],
            doc: Box::new(doc),
            has_rive_component: output.has_rive_components,
        }
    }

    pub fn to_html(&self) -> String {
        self.to_html_(false)
    }

    pub fn to_test_html(&self) -> String {
        self.to_html_(true)
    }

    fn to_html_(&self, test: bool) -> String {
        let script_file = if test {
            self.get_test_script_file()
        } else {
            self.get_script_file()
        };

        format!(
            include_str!("../../ftd/ftd-js.html"),
            // NOTE: meta_tags is only used in edition 2023 where we get this by rendering js on
            // the server (ssr)
            // In edition 2022, the executor extracts meta tags and handle it separately
            meta_tags = "",
            fastn_package = self.get_fastn_package_data(),
            base_url_tag = self
                .package
                .base_url
                .as_ref()
                .map(|v| format!("<base href=\"{v}\">"))
                .unwrap_or_default(),
            favicon_html_tag = self
                .package
                .favicon
                .as_ref()
                .map(|v| v.to_html())
                .unwrap_or_default()
                .as_str(),
            js_script = format!("{}{}", self.js, available_code_themes()).as_str(),
            script_file = script_file.as_str(),
            extra_js = "", // Todo
            default_css = fastn_js::ftd_js_css(),
            html_body = EMPTY_HTML_BODY // Todo: format!("{}{}", EMPTY_HTML_BODY, font_style)
        )
    }

    fn get_test_script_file(&self) -> String {
        format!(
            r#"
                <script src="../fastn-js/prism/prism.js"></script>
                <script src="../fastn-js/prism/prism-line-highlight.js"></script>
                <script src="../fastn-js/prism/prism-line-numbers.js"></script>
                <script src="../fastn-js/prism/prism-rust.js"></script>
                <script src="../fastn-js/prism/prism-json.js"></script>
                <script src="../fastn-js/prism/prism-python.js"></script>
                <script src="../fastn-js/prism/prism-markdown.js"></script>
                <script src="../fastn-js/prism/prism-bash.js"></script>
                <script src="../fastn-js/prism/prism-sql.js"></script>
                <script src="../fastn-js/prism/prism-javascript.js"></script>
                <link rel="stylesheet" href="../fastn-js/prism/prism-line-highlight.css">
                <link rel="stylesheet" href="../fastn-js/prism/prism-line-numbers.css">
                <script>{}</script>
            "#,
            all_js_without_test(self.package.name.as_str(), self.doc.as_ref())
        )
    }

    fn get_script_file(&self) -> String {
        let mut scripts = fastn_runtime::utils::get_external_scripts(self.has_rive_component);
        scripts.push(fastn_runtime::utils::get_js_html(self.js_files.as_slice()));
        scripts.push(fastn_runtime::utils::get_css_html(
            self.css_files.as_slice(),
        ));

        format!(
            r#"
                <script src="{}"></script>
                <script src="{}"></script>
                <script src="{}"></script>
                <link rel="stylesheet" href="{}">
                {}
            "#,
            hashed_markdown_js(),
            hashed_prism_js(),
            hashed_default_ftd_js(self.package.name.as_str(), self.doc.as_ref()),
            hashed_prism_css(),
            scripts.join("").as_str()
        )
    }

    pub fn get_fastn_package_data(&self) -> String {
        format!(
            indoc::indoc! {"
        let __fastn_package_name__ = \"{package_name}\";
    "},
            package_name = self.package.name
        )
    }
}

fn generate_hash(content: impl AsRef<[u8]>) -> String {
    use sha2::Digest;
    use sha2::digest::FixedOutput;
    let mut hasher = sha2::Sha256::new();
    hasher.update(content);
    format!("{:X}", hasher.finalize_fixed())
}

static PRISM_JS_HASH: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    format!("prism-{}.js", generate_hash(fastn_js::prism_js().as_str()),)
});

fn hashed_prism_js() -> &'static str {
    &PRISM_JS_HASH
}

static MARKDOWN_HASH: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    format!("markdown-{}.js", generate_hash(fastn_js::markdown_js()),)
});

fn hashed_markdown_js() -> &'static str {
    &MARKDOWN_HASH
}

static PRISM_CSS_HASH: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    format!(
        "prism-{}.css",
        generate_hash(fastn_js::prism_css().as_str()),
    )
});

fn hashed_prism_css() -> &'static str {
    &PRISM_CSS_HASH
}

static FTD_JS_HASH: once_cell::sync::OnceCell<String> = once_cell::sync::OnceCell::new();

fn hashed_default_ftd_js(package_name: &str, doc: &dyn fastn_resolved::tdoc::TDoc) -> &'static str {
    FTD_JS_HASH.get_or_init(|| {
        format!(
            "default-{}.js",
            generate_hash(all_js_without_test(package_name, doc).as_str())
        )
    })
}

fn all_js_without_test(package_name: &str, doc: &dyn fastn_resolved::tdoc::TDoc) -> String {
    let all_js = fastn_js::all_js_without_test();
    let default_bag_js = fastn_js::to_js(default_bag_into_js_ast(doc).as_slice(), package_name);
    format!("{all_js}\n{default_bag_js}")
}

fn default_bag_into_js_ast(doc: &dyn fastn_resolved::tdoc::TDoc) -> Vec<fastn_js::Ast> {
    let mut ftd_asts = vec![];
    let mut export_asts = vec![];
    for thing in fastn_builtins::builtins().values() {
        match thing {
            fastn_resolved::Definition::Variable(v) => {
                ftd_asts.push(v.to_ast(doc, None, &mut false));
            }
            fastn_resolved::Definition::Function(f) if !f.external_implementation => {
                ftd_asts.push(f.to_ast(doc));
            }
            fastn_resolved::Definition::Export { from, to, .. } => {
                export_asts.push(fastn_js::Ast::Export {
                    from: from.to_string(),
                    to: to.to_string(),
                })
            }
            _ => continue,
        }
    }

    // Global default inherited variable
    ftd_asts.push(fastn_js::Ast::StaticVariable(fastn_js::StaticVariable {
        name: "inherited".to_string(),
        value: fastn_js::SetPropertyValue::Value(fastn_js::Value::Record {
            fields: vec![
                (
                    "colors".to_string(),
                    fastn_js::SetPropertyValue::Reference(
                        "ftd#default-colors__DOT__getClone()__DOT__setAndReturn\
                        (\"is_root\"__COMMA__\
                         true)"
                            .to_string(),
                    ),
                ),
                (
                    "types".to_string(),
                    fastn_js::SetPropertyValue::Reference(
                        "ftd#default-types__DOT__getClone()__DOT__setAndReturn\
                        (\"is_root\"__COMMA__\
                         true)"
                            .to_string(),
                    ),
                ),
            ],
            other_references: vec![],
        }),
        prefix: None,
    }));

    ftd_asts.extend(export_asts);
    ftd_asts
}

#[derive(Debug, Default)]
pub struct Package {
    name: String,
    base_url: Option<String>,
    favicon: Option<Favicon>,
}

impl Package {
    pub fn new_name(name: &str) -> Package {
        Package {
            name: name.to_string(),
            base_url: None,
            favicon: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct Favicon {
    path: String,
    content_type: String,
}

impl Favicon {
    fn to_html(&self) -> String {
        let favicon_html = format!(
            "\n<link rel=\"shortcut icon\" href=\"{}\" type=\"{}\">",
            self.path, self.content_type
        );
        favicon_html
    }
}

fn available_code_themes() -> String {
    // TODO Move code from fastn_core::utils::available_code_themes()
    "".to_string()
}
