pub async fn mark(who: &str, whom: &str) -> fpm::Result<()> {
    let config = fpm::Config::read().await?;
    let who = format!("{}.ftd", who);
    let whom = format!("{}.ftd", whom);

    let snapshots = fpm::snaphot::get_latest_snapshots(config.root.as_str())?;

    check(&snapshots, &who, &whom, config.root.as_str()).await?;

    Ok(())
}

async fn check(
    snapshots: &std::collections::BTreeMap<String, String>,
    who: &str,
    whom: &str,
    base_path: &str,
) -> fpm::Result<()> {
    let timestamp = snapshots.get(whom).unwrap();

    let file_path = format!("{}/.tracks/{}", base_path, who.replace(".ftd", ".track"));
    let mut tracks = get_track(base_path, &file_path)?;
    if let Some(track) = tracks.get_mut(whom) {
        track.other_timestamp = Some(timestamp.to_string());
    }
    write(&file_path, &tracks).await?;
    Ok(())
}
async fn write(
    file_path: &str,
    tracks: &std::collections::BTreeMap<String, Tracks>,
) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;

    let mut f = tokio::fs::File::create(file_path).await?;
    let mut string = "-- import: fpm".to_string();

    for track in tracks.values() {
        string = format!(
            "{}\n\n-- fpm.track: {}\nself-timestamp: {}",
            string, track.document_name, track.self_timestamp
        );
        if let Some(ref other_timestamp) = track.other_timestamp {
            string = format!("{}\nother-timestamp: {}", string, other_timestamp);
        }
    }
    f.write_all(string.as_bytes()).await?;
    Ok(())
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Tracks {
    #[serde(rename = "document-name")]
    pub document_name: String,
    pub package: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "other-timestamp")]
    pub other_timestamp: Option<String>,
    #[serde(rename = "self-timestamp")]
    pub self_timestamp: String,
}

impl Tracks {
    pub fn parse(b: &ftd::p2::Document) -> fpm::Result<Vec<Tracks>> {
        Ok(b.to_owned().instances::<Tracks>("fpm#track")?)
    }
}

fn get_track(
    base_path: &str,
    path: &str,
) -> fpm::Result<std::collections::BTreeMap<String, Tracks>> {
    let mut tracks = std::collections::BTreeMap::new();
    if std::fs::metadata(&path).is_err() {
        return Ok(tracks);
    }

    let lib = fpm::Library {};
    let doc = std::fs::read_to_string(&path)?;
    let b = match ftd::p2::Document::from(base_path, doc.as_str(), &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", base_path, &e);
            todo!();
        }
    };
    let track_list = Tracks::parse(&b)?;
    for track in track_list {
        tracks.insert(track.document_name.to_string(), track);
    }
    Ok(tracks)
}
