/*
document filename
foo/abc.ftd

document id
/foo/abc/
/foo/abc/-/x/y/ --> full id
/x/y/ - suffix
*/

pub mod processor {

    pub fn document_id<'a>(
        _section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc<'a>,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let doc_id = config.doc_id().unwrap_or_else(|| {
            doc.name
                .to_string()
                .replace(config.package.name.as_str(), "")
        });

        let document_id = doc_id
            .split_once("/-/")
            .map(|x| x.0)
            .unwrap_or_else(|| &doc_id)
            .trim_matches('/');

        Ok(ftd::Value::String {
            text: format!("/{}/", document_id),
            source: ftd::TextSource::Default,
        })
    }
    pub fn document_full_id<'a>(
        _section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc<'a>,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let full_document_id = config.doc_id().unwrap_or_else(|| {
            doc.name
                .to_string()
                .replace(config.package.name.as_str(), "")
        });

        Ok(ftd::Value::String {
            text: format!("/{}/", full_document_id.trim_matches('/')),
            source: ftd::TextSource::Default,
        })
    }
    pub async fn document_filename<'a>(
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc<'a>,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let doc_id = config.doc_id().unwrap_or_else(|| {
            doc.name
                .to_string()
                .replace(config.package.name.as_str(), "")
        });

        let file_path =
            config
                .get_file_path(&doc_id)
                .await
                .map_err(|e| ftd::p1::Error::ParseError {
                    message: e.to_string(),
                    doc_id: doc.name.to_string(),
                    line_number: section.line_number,
                })?;

        Ok(ftd::Value::String {
            text: file_path.trim().to_string(),
            source: ftd::TextSource::Default,
        })
    }

    pub fn document_suffix<'a>(
        _section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc<'a>,
        config: &fpm::Config,
    ) -> ftd::p1::Result<ftd::Value> {
        let doc_id = config.doc_id().unwrap_or_else(|| {
            doc.name
                .to_string()
                .replace(config.package.name.as_str(), "")
        });

        let value = doc_id
            .split_once("/-/")
            .map(|(_, y)| y.trim().to_string())
            .map(|suffix| ftd::Value::String {
                text: suffix,
                source: ftd::TextSource::Default,
            });

        Ok(ftd::Value::Optional {
            data: Box::new(value),
            kind: ftd::p2::Kind::String {
                caption: false,
                body: false,
                default: None,
                is_reference: false,
            },
        })
    }
}
