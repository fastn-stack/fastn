use itertools::Itertools;

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq, Clone)]
pub struct FileHistory {
    pub filename: String,
    #[serde(rename = "file-edit")]
    pub file_edit: Vec<FileEdit>,
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq, Clone)]
pub struct FileEdit {
    pub message: Option<String>,
    pub timestamp: u128,
    pub version: i32,
    pub author: Option<String>,
    #[serde(rename = "src-cr")]
    pub src_cr: Option<i32>,
    pub operation: FileOperation,
}

impl FileEdit {
    pub(crate) fn to_workspace(&self, file_name: &str) -> fpm::workspace::WorkspaceEntry {
        fpm::workspace::WorkspaceEntry {
            filename: file_name.to_string(),
            deleted: Some(self.operation.eq(&FileOperation::Deleted)),
            version: Some(self.version),
        }
    }

    pub(crate) fn is_deleted(&self) -> bool {
        self.operation.eq(&FileOperation::Deleted)
    }
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq, Clone)]
pub struct FileEditTemp {
    pub message: Option<String>,
    pub author: Option<String>,
    pub src_cr: Option<i32>,
    pub operation: FileOperation,
}

impl FileEditTemp {
    fn to_file_edit(&self, timestamp: u128, version: i32) -> FileEdit {
        FileEdit {
            message: self.message.clone(),
            timestamp,
            version,
            author: self.author.clone(),
            src_cr: self.src_cr,
            operation: self.operation.clone(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq, Clone)]
pub enum FileOperation {
    Added,
    Updated,
    Deleted,
    Merged,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum FileOperation1 {
    Added,
    Updated,
    Deleted,
    Merged,
}

impl fpm::Config {
    pub async fn get_history(&self) -> fpm::Result<Vec<FileHistory>> {
        let history_file_path = self.history_file();
        let history_content = tokio::fs::read_to_string(history_file_path).await?;
        FileHistory::from_ftd(history_content.as_str())
    }

    pub async fn get_latest_file_edits_with_deleted(
        &self,
    ) -> fpm::Result<std::collections::BTreeMap<String, FileEdit>> {
        let history_list = self.get_history().await?;
        fpm::history::FileHistory::get_latest_file_edits(history_list.as_slice())
    }

    pub async fn get_latest_file_edits(
        &self,
    ) -> fpm::Result<std::collections::BTreeMap<String, FileEdit>> {
        let history_list = self.get_history().await?;
        Ok(
            fpm::history::FileHistory::get_latest_file_edits(history_list.as_slice())?
                .into_iter()
                .filter(|(_, v)| !v.is_deleted())
                .collect(),
        )
    }

    pub async fn get_latest_file_paths(&self) -> fpm::Result<Vec<(String, camino::Utf8PathBuf)>> {
        Ok(self
            .get_latest_file_edits()
            .await?
            .iter()
            .map(|(file_name, file_edit)| {
                (
                    file_name.to_string(),
                    self.history_path(file_name, file_edit.version),
                )
            })
            .collect_vec())
    }
}

impl FileHistory {
    pub(crate) fn get_latest_file_edits(
        list: &[FileHistory],
    ) -> fpm::Result<std::collections::BTreeMap<String, FileEdit>> {
        Ok(list
            .iter()
            .filter_map(|v| {
                v.get_latest_file_edit()
                    .map(|file_edit| (v.filename.to_string(), file_edit))
            })
            .collect())
    }

    fn get_latest_file_edit(&self) -> Option<FileEdit> {
        for file_edit in self.file_edit.iter() {
            if file_edit.operation.eq(&FileOperation::Merged) {
                return Some(file_edit.clone());
            }
            if file_edit.src_cr.is_none() {
                return Some(file_edit.clone());
            }
        }
        None
    }

    pub(crate) fn to_ftd(history: &[&fpm::history::FileHistory]) -> String {
        let mut files_history = vec!["-- import: fpm".to_string()];
        for file_history in history {
            let mut file_history_data = format!("-- fpm.history: {}\n", file_history.filename);
            for file_edit in &file_history.file_edit {
                let author = file_edit
                    .author
                    .as_ref()
                    .map(|v| format!("author: {}\n", v))
                    .unwrap_or_else(|| "".to_string());
                let src_cr = file_edit
                    .src_cr
                    .map(|v| format!("src-cr: {}\n", v))
                    .unwrap_or_else(|| "".to_string());
                file_history_data = format!(
                    "{}\n--- file-edit:\ntimestamp: {}\noperation: {:?}\nversion: {}\n{}{}\n{}\n",
                    file_history_data,
                    file_edit.timestamp,
                    file_edit.operation,
                    file_edit.version,
                    author,
                    src_cr,
                    file_edit.message.as_ref().unwrap_or(&"".to_string())
                );
            }
            files_history.push(file_history_data);
        }
        files_history.join("\n\n\n")
    }

    pub(crate) fn from_ftd(file: &str) -> fpm::Result<Vec<FileHistory>> {
        let doc = {
            let lib = fpm::FPMLibrary::default();
            fpm::doc::parse_ftd("history.ftd", file, &lib)?
        };
        Ok(doc.get("fpm#history")?)
    }
}

pub(crate) async fn insert_into_history(
    root: &camino::Utf8PathBuf,
    file_list: &std::collections::BTreeMap<String, fpm::history::FileEditTemp>,
    history: &mut Vec<fpm::history::FileHistory>,
) -> fpm::Result<()> {
    let mut file_history: std::collections::BTreeMap<String, fpm::history::FileHistory> = history
        .iter_mut()
        .map(|v| (v.filename.to_string(), v.clone()))
        .collect();
    insert_into_history_(root, file_list, &mut file_history).await?;
    *history = file_history.into_values().collect_vec();
    Ok(())
}

pub(crate) async fn insert_into_history_(
    root: &camino::Utf8PathBuf,
    file_list: &std::collections::BTreeMap<String, fpm::history::FileEditTemp>,
    file_history: &mut std::collections::BTreeMap<String, fpm::history::FileHistory>,
) -> fpm::Result<()> {
    let timestamp = fpm::timestamp_nanosecond();
    for (file, file_op) in file_list {
        let version =
            fpm::snapshot::get_new_version(file_history.values().collect_vec().as_slice(), file);
        if let Some(file_history) = file_history.get_mut(file) {
            file_history
                .file_edit
                .insert(0, file_op.to_file_edit(timestamp, version))
        } else {
            file_history.insert(
                file.to_string(),
                FileHistory {
                    filename: file.to_string(),
                    file_edit: vec![file_op.to_file_edit(timestamp, version)],
                },
            );
        }
        let server_state = root.join(".server-state").join("history");

        if !server_state.exists() {
            tokio::fs::create_dir_all(&server_state).await?;
        }

        if !file_op.operation.eq(&FileOperation::Deleted) {
            let new_file_path =
                server_state.join(fpm::utils::snapshot_id(file, &(version as u128)));
            tokio::fs::copy(root.join(file), new_file_path).await?;
        }
    }

    let history_ftd = FileHistory::to_ftd(file_history.values().collect_vec().as_slice());
    tokio::fs::write(
        root.join(".server-state").join("history.ftd"),
        history_ftd.as_str(),
    )
    .await?;

    Ok(())
}
