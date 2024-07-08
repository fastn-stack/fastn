#[allow(dead_code)]
#[derive(serde::Deserialize, Debug, Clone)]
pub struct Track {
    pub filename: String,
    pub package: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "other-timestamp")]
    pub other_timestamp: Option<u128>,
    #[serde(rename = "self-timestamp")]
    pub self_timestamp: u128,
    #[serde(rename = "last-merged-version")]
    pub last_merged_version: Option<u128>,
}

pub(crate) async fn get_tracks(
    config: &fastn_core::Config,
    base_path: &fastn_ds::Path,
    path: &fastn_ds::Path,
    session_id: &Option<String>,
) -> fastn_core::Result<std::collections::BTreeMap<String, Track>> {
    let mut tracks = std::collections::BTreeMap::new();
    if !config.ds.exists(path).await {
        return Ok(tracks);
    }

    let lib = fastn_core::FastnLibrary::default();
    let doc = config.ds.read_to_string(path, session_id).await?;
    let b = match fastn_core::doc::parse_ftd(base_path.to_string().as_str(), doc.as_str(), &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", base_path, &e);
            todo!();
        }
    };
    let track_list: Vec<Track> = b.get("fastn#track")?;
    for track in track_list {
        tracks.insert(track.filename.to_string(), track);
    }
    Ok(tracks)
}
