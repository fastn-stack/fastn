#[derive(serde::Deserialize, Debug, Clone)]
pub struct Track {
    #[serde(rename = "document-name")]
    pub document_name: String,
    pub package: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "other-timestamp")]
    pub other_timestamp: Option<String>,
    #[serde(rename = "self-timestamp")]
    pub self_timestamp: String,
}

pub(crate) fn get_tracks(
    base_path: &str,
    path: &str,
) -> fpm::Result<std::collections::BTreeMap<String, Track>> {
    let mut tracks = std::collections::BTreeMap::new();
    if std::fs::metadata(&path).is_err() {
        return Ok(tracks);
    }

    let lib = fpm::Library::default();
    let doc = std::fs::read_to_string(&path)?;
    let b = match ftd::p2::Document::from(base_path, doc.as_str(), &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", base_path, &e);
            todo!();
        }
    };
    let track_list: Vec<Track> = b.get("fpm#track")?;
    for track in track_list {
        tracks.insert(track.document_name.to_string(), track);
    }
    Ok(tracks)
}
