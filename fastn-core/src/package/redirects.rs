#[derive(Debug, serde::Deserialize, Clone)]
pub struct RedirectsTemp {
    #[serde(rename = "redirects-body")]
    pub body: String,
}

impl RedirectsTemp {
    pub(crate) fn redirects_from_body(&self) -> fastn_core::Result<ftd::Map<String>> {
        let body = self.body.as_str();
        let mut redirects: ftd::Map<String> = ftd::Map::new();
        for line in body.lines() {
            if line.is_empty() || line.trim_start().starts_with(';') {
                continue;
            }
            // Supported Redirects Syntax under fastn.redirects
            // <some link>: <link to redirect>
            // <some link> -> <link to redirect>

            if let Some((key, value)) = line.split_once("->") {
                Self::assert_and_insert_redirect(key, value, &mut redirects)?;
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                fastn_core::warning!(
                    "Redirect syntax: '{key}: {value}' will be deprecated\nPlease use the '{key} \
                    -> {value}' redirect syntax instead."
                );
                Self::assert_and_insert_redirect(key, value, &mut redirects)?;
            }
        }
        Ok(redirects)
    }

    // Assert checks on redirects
    // - All redirects should be A -> B where A != B (Self loop)
    // - If A -> B exists then there can’t be A -> C where B != C
    //   (No duplicated values starting with the same A)
    fn assert_and_insert_redirect(
        from: &str,
        to: &str,
        redirects: &mut ftd::Map<String>,
    ) -> fastn_core::Result<()> {
        let from = from.trim().to_owned();
        let to = to.trim().to_owned();

        assert!(
            !from.eq(to.as_str()),
            "Redirect {} -> {} is invalid",
            from,
            to
        );
        assert!(
            !redirects.contains_key(from.as_str()),
            "Redirect {} -> {} is invalid, since {} -> {} already exists",
            from.as_str(),
            to.as_str(),
            from.as_str(),
            redirects.get(from.as_str()).unwrap(),
        );

        redirects.insert(from, to);
        Ok(())
    }
}

pub fn find_redirect<'a>(redirects: &'a ftd::Map<String>, path: &str) -> Option<&'a String> {
    let original = path;
    let fixed = format!(
        "/{}/",
        path.trim_matches('/')
            .trim_end_matches("index.ftd")
            .trim_end_matches(".ftd")
    );

    return if redirects.contains_key(original) {
        redirects.get(original)
    } else if redirects.contains_key(fixed.as_str()) {
        redirects.get(fixed.as_str())
    } else {
        None
    };
}
