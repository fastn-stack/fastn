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
        translated_latest: u128,
    },
    UptoDate {
        translated: fpm::File,
    },
}

impl TranslatedDocument {
    pub async fn html(
        &self,
        config: &fpm::Config,
        base_path: &camino::Utf8PathBuf,
    ) -> fpm::Result<()> {
        // handle the message
        // render with-fallback or with-message
        let message = fpm::get_messages(self);
        let (main, fallback) = match self {
            TranslatedDocument::Missing { original } => (original, None),
            TranslatedDocument::NeverMarked {
                original,
                translated,
            } => (original, Some(translated)),
            TranslatedDocument::Outdated {
                original,
                translated,
                ..
            } => (translated, Some(original)),
            TranslatedDocument::UptoDate { translated, .. } => (translated, None),
        };
        fpm::process_file(config, base_path, main, fallback, Some(message)).await?;
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
