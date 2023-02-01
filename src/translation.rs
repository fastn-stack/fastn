#![allow(dead_code, unused, unused_variables)]

#[derive(Debug)]
pub(crate) enum TranslatedDocument {
    Missing {
        original: fastn::File,
    },
    NeverMarked {
        original: fastn::File,   // main
        translated: fastn::File, // fallback
    },
    Outdated {
        original: fastn::File,   // fallback
        translated: fastn::File, // main
        last_marked_on: u128,
        original_latest: u128,
        translated_latest: u128,
    },
    UptoDate {
        translated: fastn::File,
    },
}

#[derive(Debug, Default, Clone)]
pub struct TranslationData {
    pub diff: Option<String>,
    pub last_marked_on: Option<u128>,
    pub original_latest: Option<u128>,
    pub translated_latest: Option<u128>,
    pub status: Option<String>,
}

impl TranslationData {
    fn new(status: &str) -> TranslationData {
        TranslationData {
            diff: None,
            last_marked_on: None,
            original_latest: None,
            translated_latest: None,
            status: Some(status.to_string()),
        }
    }
}

impl TranslatedDocument {
    pub async fn html(
        &self,
        config: &fastn::Config,
        _base_url: &str,
        _skip_failed: bool,
        _asset_documents: &std::collections::HashMap<String, String>,
    ) -> fastn::Result<()> {
        // handle the message
        // render with-fallback or with-message
        let _message = fastn::get_messages(self, config)?;
        let (_main, _fallback, _translated_data) = match self {
            TranslatedDocument::Missing { original } => {
                (original, None, TranslationData::new("Missing"))
            }
            TranslatedDocument::NeverMarked {
                original,
                translated,
            } => (
                original,
                Some(translated),
                TranslationData::new("NeverMarked"),
            ),
            TranslatedDocument::Outdated {
                original,
                translated,
                last_marked_on,
                original_latest,
                translated_latest,
            } => {
                // Gets the diff on original file between last_marked_on and original_latest timestamp
                let diff = get_diff(config, original, last_marked_on, original_latest).await?;
                let translated_data = TranslationData {
                    diff: Some(diff),
                    last_marked_on: Some(*last_marked_on),
                    original_latest: Some(*original_latest),
                    translated_latest: Some(*translated_latest),
                    status: Some("Outdated".to_string()),
                };

                (translated, Some(original), translated_data)
            }
            TranslatedDocument::UptoDate { translated, .. } => {
                (translated, None, TranslationData::new("UptoDate"))
            }
        };

        todo!();
        // fastn::process_file(
        //     config,
        //     &config.package,
        //     main,
        //     fallback,
        //     Some(message.as_str()),
        //     translated_data,
        //     base_url,
        //     skip_failed,
        //     asset_documents,
        //     None,
        //     false,
        // )
        // .await?;
        // return Ok(());

        /// Gets the diff on original file between last_marked_on and original_latest timestamp
        async fn get_diff(
            config: &fastn::Config,
            original: &fastn::File,
            last_marked_on: &u128,
            original_latest: &u128,
        ) -> fastn::Result<String> {
            let last_marked_on_path = fastn::utils::history_path(
                original.get_id().as_str(),
                config.original_path()?.as_str(),
                last_marked_on,
            );
            let last_marked_on_data = tokio::fs::read_to_string(last_marked_on_path).await?;
            let original_latest_path = fastn::utils::history_path(
                original.get_id().as_str(),
                config.original_path()?.as_str(),
                original_latest,
            );
            let original_latest_data = tokio::fs::read_to_string(original_latest_path).await?;

            let patch = diffy::create_patch(&last_marked_on_data, &original_latest_data);
            Ok(patch.to_string().replace("---", "\\---"))
        }
    }

    pub async fn get_translated_document(
        config: &fastn::Config,
        original_documents: std::collections::BTreeMap<String, fastn::File>,
        translated_documents: std::collections::BTreeMap<String, fastn::File>,
    ) -> fastn::Result<std::collections::BTreeMap<String, TranslatedDocument>> {
        let original_snapshots =
            fastn::snapshot::get_latest_snapshots(&config.original_path()?).await?;
        let mut translation_status = std::collections::BTreeMap::new();
        for (file, timestamp) in original_snapshots {
            let original_document =
                if let Some(original_document) = original_documents.get(file.as_str()) {
                    original_document
                } else {
                    return Err(fastn::Error::PackageError {
                        message: format!("Could not find `{}` in original package", file),
                    });
                };
            if !translated_documents.contains_key(&file) {
                translation_status.insert(
                    file,
                    TranslatedDocument::Missing {
                        original: original_document.clone(),
                    },
                );
                continue;
            }
            let translated_document = translated_documents.get(file.as_str()).unwrap();
            let track_path = fastn::utils::track_path(file.as_str(), config.root.as_str());
            if !track_path.exists() {
                translation_status.insert(
                    file,
                    TranslatedDocument::NeverMarked {
                        original: original_document.clone(),
                        translated: translated_document.clone(),
                    },
                );
                continue;
            }
            let tracks = fastn::tracker::get_tracks(config.root.as_str(), &track_path)?;
            if let Some(fastn::Track {
                last_merged_version: Some(last_merged_version),
                self_timestamp,
                ..
            }) = tracks.get(&file)
            {
                if last_merged_version < &timestamp {
                    translation_status.insert(
                        file,
                        TranslatedDocument::Outdated {
                            original: original_document.clone(),
                            translated: translated_document.clone(),
                            last_marked_on: *last_merged_version,
                            original_latest: timestamp,
                            translated_latest: *self_timestamp,
                        },
                    );
                    continue;
                }
                translation_status.insert(
                    file,
                    TranslatedDocument::UptoDate {
                        translated: translated_document.clone(),
                    },
                );
            } else {
                translation_status.insert(
                    file,
                    TranslatedDocument::NeverMarked {
                        original: original_document.clone(),
                        translated: translated_document.clone(),
                    },
                );
            }
        }
        Ok(translation_status)
    }
}

pub(crate) fn get_translation_status_counts(
    snapshots: &std::collections::BTreeMap<String, u128>,
    path: &camino::Utf8PathBuf,
) -> fastn::Result<TranslationStatusSummary> {
    let mut translation_status_count = TranslationStatusSummary {
        never_marked: 0,
        missing: 0,
        out_dated: 0,
        upto_date: 0,
        last_modified_on: None,
    };
    for (file, timestamp) in snapshots {
        if !path.join(file).exists() {
            translation_status_count.missing += 1;
            continue;
        }
        let track_path = fastn::utils::track_path(file.as_str(), path.as_str());
        if !track_path.exists() {
            translation_status_count.never_marked += 1;
            continue;
        }
        let tracks = fastn::tracker::get_tracks(path.as_str(), &track_path)?;
        if let Some(fastn::Track {
            last_merged_version: Some(last_merged_version),
            ..
        }) = tracks.get(file)
        {
            if last_merged_version < timestamp {
                translation_status_count.out_dated += 1;
                continue;
            }
            translation_status_count.upto_date += 1;
        } else {
            translation_status_count.never_marked += 1;
        }
    }
    translation_status_count.last_modified_on =
        futures::executor::block_on(fastn::utils::get_last_modified_on(path));
    Ok(translation_status_count)
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct TranslationStatusSummary {
    #[serde(rename = "never-marked")]
    pub never_marked: i32,
    pub missing: i32,
    #[serde(rename = "out-dated")]
    pub out_dated: i32,
    #[serde(rename = "upto-date")]
    pub upto_date: i32,
    #[serde(rename = "last-modified-on")]
    pub last_modified_on: Option<String>,
}

impl ToString for TranslationStatusSummary {
    fn to_string(&self) -> String {
        format!(
            indoc::indoc! {"
                Never marked: {never_marked}
                Missing: {missing}
                Out-dated: {out_dated}
                Up to date: {upto_date}

                "},
            never_marked = self.never_marked,
            missing = self.missing,
            out_dated = self.out_dated,
            upto_date = self.upto_date
        )
    }
}
