pub(crate) enum TranslatedDocument {
    Missing {
        original: fpm::File,
    },
    NeverMarked {
        original: fpm::File,   // main
        translated: fpm::File, // fallback
    },
    Outdated {
        original: fpm::File,   // fallback
        translated: fpm::File, // main
        last_marked_on: u128,
        original_latest: u128,
        #[allow(dead_code)]
        translated_latest: u128,
    },
    UptoDate {
        translated: fpm::File,
    },
}

impl TranslatedDocument {
    pub async fn html(&self, config: &fpm::Config) -> fpm::Result<()> {
        // handle the message
        // render with-fallback or with-message
        let message = fpm::get_messages(self).to_string();
        let (main, fallback, diff) = match self {
            TranslatedDocument::Missing { original } => (original, None, None),
            TranslatedDocument::NeverMarked {
                original,
                translated,
            } => (original, Some(translated), None),
            TranslatedDocument::Outdated {
                original,
                translated,
                last_marked_on,
                original_latest,
                ..
            } => {
                // This adds the diff on original file between last_marked_on and original_latest timestamp
                let last_marked_on_path = fpm::utils::history_path(
                    original.get_id().as_str(),
                    config.original_path()?.as_str(),
                    last_marked_on,
                );
                let last_marked_on_data = tokio::fs::read_to_string(last_marked_on_path).await?;
                let original_latest_path = fpm::utils::history_path(
                    original.get_id().as_str(),
                    config.original_path()?.as_str(),
                    original_latest,
                );
                let original_latest_data = tokio::fs::read_to_string(original_latest_path).await?;

                let patch = diffy::create_patch(&last_marked_on_data, &original_latest_data);
                let diff = patch
                    .to_string()
                    .replace("\n", "\n\n")
                    .replace("---", "\\---");

                (translated, Some(original), Some(diff))
            }
            TranslatedDocument::UptoDate { translated, .. } => (translated, None, None),
        };
        fpm::process_file(config, main, fallback, Some(message.as_str()), diff).await?;
        Ok(())
    }

    pub async fn get_translated_document(
        config: &fpm::Config,
        original_documents: std::collections::BTreeMap<String, fpm::File>,
        translated_documents: std::collections::BTreeMap<String, fpm::File>,
    ) -> fpm::Result<std::collections::BTreeMap<String, TranslatedDocument>> {
        let original_snapshots =
            fpm::snapshot::get_latest_snapshots(&config.original_path()?).await?;
        let mut translation_status = std::collections::BTreeMap::new();
        for (file, timestamp) in original_snapshots {
            let original_document = original_documents.get(file.as_str()).unwrap();
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
            let track_path = fpm::utils::track_path(file.as_str(), config.root.as_str());
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
            let tracks = fpm::tracker::get_tracks(config.root.as_str(), &track_path)?;
            if let Some(fpm::Track {
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
