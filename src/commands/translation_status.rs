pub async fn translation_status(config: &fpm::Config) -> fpm::Result<()> {
    // it can be original package or translation
    let translation_status = if let Some(ref original) = config.package.translation_of.as_ref() {
        translation_package_status(config, original).await?
    } else if !config.package.translations.is_empty() {
        original_package_status().await?
    } else {
        return Err(fpm::Error::UsageError {
            message: "neither translation_of nor translations is set".to_string(),
        });
    };

    print_translation_status(&translation_status);
    Ok(())
}

async fn translation_package_status(
    config: &fpm::Config,
    original: &fpm::Package,
) -> fpm::Result<std::collections::BTreeMap<String, TranslationStatus>> {
    let original_path = config.root.join(".packages").join(original.name.as_str());
    let snapshots = fpm::snapshot::get_latest_snapshots(&original_path).await?;
    let mut translation_status = std::collections::BTreeMap::new();
    for (file, timestamp) in snapshots {
        if !config.root.join(&file).exists() {
            translation_status.insert(file, TranslationStatus::Missing);
            continue;
        }
        let track_path = fpm::utils::track_path(file.as_str(), config.root.as_str());
        if !track_path.exists() {
            translation_status.insert(file, TranslationStatus::NeverMarked);
            continue;
        }
        let tracks = fpm::tracker::get_tracks(config.root.as_str(), &track_path)?;
        if let Some(fpm::Track {
            last_merged_version: Some(last_merged_version),
            ..
        }) = tracks.get(&file)
        {
            if last_merged_version < &timestamp {
                translation_status.insert(file, TranslationStatus::Outdated);
                continue;
            }
            translation_status.insert(file, TranslationStatus::UptoDate);
        } else {
            translation_status.insert(file, TranslationStatus::NeverMarked);
        }
    }
    Ok(translation_status)
}

async fn original_package_status(
) -> fpm::Result<std::collections::BTreeMap<String, TranslationStatus>> {
    Ok(std::collections::BTreeMap::new())
}

fn print_translation_status(
    translation_status: &std::collections::BTreeMap<String, TranslationStatus>,
) {
    for (file, status) in translation_status {
        println!("{}: {}", status.to_string(), file);
    }
}

enum TranslationStatus {
    Missing,
    NeverMarked,
    Outdated,
    UptoDate,
}

impl ToString for TranslationStatus {
    fn to_string(&self) -> String {
        match self {
            TranslationStatus::Missing => "Missing".to_string(),
            TranslationStatus::NeverMarked => "Never marked".to_string(),
            TranslationStatus::Outdated => "Out-dated".to_string(),
            TranslationStatus::UptoDate => "Up to date".to_string(),
        }
    }
}
