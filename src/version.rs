use itertools::Itertools;
use std::cmp::Ordering;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Version {
    pub major: u64,
    pub minor: Option<u64>,
    pub original: String,
}

impl Version {
    pub(crate) fn base() -> fpm::Version {
        fpm::Version {
            major: 0,
            minor: None,
            original: "BASE_VERSION".to_string(),
        }
    }

    pub(crate) fn parse(s: &str) -> fpm::Result<fpm::Version> {
        let v = s.strip_prefix(&['v', 'V']).unwrap_or_else(|| s);
        let mut minor = None;
        let major = if let Some((major, minor_)) = v.split_once('.') {
            if minor_.contains('.') {
                return Err(fpm::Error::UsageError {
                    message: format!("Cannot have more than one dots `.`, found: `{}`", s),
                });
            }
            let minor_ = minor_.parse::<u64>().map_err(|e| fpm::Error::UsageError {
                message: format!("Invalid minor for `{}`: `{:?}`", s, e),
            })?;
            minor = Some(minor_);
            major.parse::<u64>().map_err(|e| fpm::Error::UsageError {
                message: format!("Invalid major for `{}`: `{:?}`", s, e),
            })?
        } else {
            v.parse::<u64>().map_err(|e| fpm::Error::UsageError {
                message: format!("Invalid major for `{}`: `{:?}`", s, e),
            })?
        };
        Ok(fpm::Version {
            major,
            minor,
            original: s.to_string(),
        })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, rhs))
    }
}

impl Ord for Version {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        if self.major.eq(&rhs.major) {
            let lhs_minor = self.minor.unwrap_or(0);
            let rhs_minor = rhs.minor.unwrap_or(0);
            return lhs_minor.cmp(&rhs_minor);
        }
        self.major.cmp(&rhs.major)
    }
}

pub(crate) async fn build_version(
    config: &fpm::Config,
    _file: Option<&str>,
    base_url: &str,
    skip_failed: bool,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    let versioned_documents = config.get_versions(&config.package).await?;
    let mut documents = std::collections::BTreeMap::new();
    for key in versioned_documents.keys().sorted() {
        let doc = versioned_documents[key].to_owned();
        documents.extend(doc.iter().map(|v| (v.get_id(), v.to_owned())));
        if key.eq(&fpm::Version::base()) {
            continue;
        }
        for doc in documents.values() {
            let mut doc = doc.clone();
            let id = doc.get_id();
            if id.eq("FPM.ftd") {
                continue;
            }
            doc.set_id(format!("{}/{}", key.original, id).as_str());
            fpm::process_file(
                config,
                &config.package,
                &doc,
                None,
                None,
                Default::default(),
                base_url,
                skip_failed,
                asset_documents,
                Some(id),
            )
            .await?;
        }
    }

    for doc in documents.values() {
        fpm::process_file(
            config,
            &config.package,
            doc,
            None,
            None,
            Default::default(),
            base_url,
            skip_failed,
            asset_documents,
            None,
        )
        .await?;
    }
    Ok(())
}
