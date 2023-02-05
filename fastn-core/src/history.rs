#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq, Eq, Clone)]
pub struct FileHistory {
    pub filename: String,
    #[serde(rename = "file-edit")]
    pub file_edit: Vec<FileEdit>,
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq, Eq, Clone)]
pub struct FileEdit {
    pub message: Option<String>,
    pub timestamp: u128,
    pub version: i32,
    pub author: Option<String>,
    #[serde(rename = "src-cr")]
    pub src_cr: Option<usize>,
    pub operation: FileOperation,
}

impl FileEdit {
    pub(crate) fn into_workspace(self, file_name: &str) -> fastn_core::workspace::WorkspaceEntry {
        fastn_core::workspace::WorkspaceEntry {
            filename: file_name.to_string(),
            deleted: Some(self.operation.eq(&FileOperation::Deleted)),
            version: Some(self.version),
            cr: self.src_cr,
        }
    }

    pub(crate) fn is_deleted(&self) -> bool {
        self.operation.is_deleted()
    }
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq, Eq, Clone)]
pub struct FileEditTemp {
    pub message: Option<String>,
    pub author: Option<String>,
    pub src_cr: Option<usize>,
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

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq, Eq, Clone)]
pub enum FileOperation {
    Added,
    Updated,
    Deleted,
    Merged,
}

impl FileOperation {
    pub(crate) fn is_deleted(&self) -> bool {
        self.eq(&FileOperation::Deleted)
    }
}

impl fastn_core::Config {
    pub async fn get_history(&self) -> fastn_core::Result<Vec<FileHistory>> {
        let history_file_path = self.history_file();
        let history_content = tokio::fs::read_to_string(history_file_path).await?;
        FileHistory::from_ftd(history_content.as_str())
    }

    pub async fn get_remote_manifest(
        &self,
        with_deleted: bool,
    ) -> fastn_core::Result<std::collections::BTreeMap<String, fastn_core::history::FileEdit>> {
        let history_list = self.get_history().await?;
        if with_deleted {
            fastn_core::history::FileHistory::get_remote_manifest(history_list.as_slice(), with_deleted)
        } else {
            Ok(fastn_core::history::FileHistory::get_remote_manifest(
                history_list.as_slice(),
                with_deleted,
            )?
            .into_iter()
            .filter(|(_, v)| !v.is_deleted())
            .collect())
        }
    }

    pub async fn get_cr_manifest(
        &self,
        cr_number: usize,
    ) -> fastn_core::Result<std::collections::BTreeMap<String, fastn_core::history::FileEdit>> {
        let history_list = self.get_history().await?;
        let cr_path_prefix = fastn_core::cr::cr_path(cr_number);
        Ok(
            fastn_core::history::FileHistory::get_remote_manifest(history_list.as_slice(), true)?
                .into_iter()
                .filter(|(k, _)| {
                    k.starts_with(cr_path_prefix.as_str())
                        || k.starts_with(format!(".tracks/{}", cr_path_prefix).as_str())
                })
                .collect(),
        )
    }

    pub async fn get_non_deleted_latest_file_paths(
        &self,
    ) -> fastn_core::Result<Vec<(String, camino::Utf8PathBuf)>> {
        use itertools::Itertools;

        Ok(self
            .get_remote_manifest(false)
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
    pub(crate) fn get_remote_manifest(
        list: &[FileHistory],
        with_deleted: bool,
    ) -> fastn_core::Result<std::collections::BTreeMap<String, FileEdit>> {
        Ok(list
            .iter()
            .filter_map(|v| {
                v.get_latest_file_edit(with_deleted)
                    .map(|file_edit| (v.filename.to_string(), file_edit))
            })
            .collect())
    }

    fn get_latest_file_edit(&self, with_deleted: bool) -> Option<FileEdit> {
        for file_edit in self.file_edit.iter() {
            if file_edit.is_deleted() && !with_deleted {
                return None;
            }
            if file_edit.operation.eq(&FileOperation::Merged) {
                return Some(file_edit.clone());
            }
            if file_edit.src_cr.is_none() {
                return Some(file_edit.clone());
            }
        }
        None
    }

    pub(crate) fn to_ftd(history: &[&fastn_core::history::FileHistory]) -> String {
        let mut files_history = vec!["-- import: fastn".to_string()];
        for file_history in history {
            let mut file_history_data = format!("-- fastn.history: {}\n", file_history.filename);
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

    pub(crate) fn from_ftd(file: &str) -> fastn_core::Result<Vec<FileHistory>> {
        let doc = {
            let lib = fastn_core::FastnLibrary::default();
            fastn_core::doc::parse_ftd("history.ftd", file, &lib)?
        };
        Ok(doc.get("fastn#history")?)
    }
}

pub(crate) async fn insert_into_history(
    root: &camino::Utf8PathBuf,
    file_list: &std::collections::BTreeMap<String, fastn_core::history::FileEditTemp>,
    history: &mut Vec<fastn_core::history::FileHistory>,
) -> fastn_core::Result<()> {
    use itertools::Itertools;

    let mut file_history: std::collections::BTreeMap<String, fastn_core::history::FileHistory> = history
        .iter_mut()
        .map(|v| (v.filename.to_string(), v.clone()))
        .collect();
    insert_into_history_(root, file_list, &mut file_history).await?;
    *history = file_history.into_values().collect_vec();
    Ok(())
}

pub(crate) async fn insert_into_history_(
    root: &camino::Utf8PathBuf,
    file_list: &std::collections::BTreeMap<String, fastn_core::history::FileEditTemp>,
    file_history: &mut std::collections::BTreeMap<String, fastn_core::history::FileHistory>,
) -> fastn_core::Result<()> {
    use itertools::Itertools;

    let timestamp = fastn_core::timestamp_nanosecond();
    for (file, file_op) in file_list {
        let version =
            fastn_core::snapshot::get_new_version(file_history.values().collect_vec().as_slice(), file);
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
        let remote_state = root.join(".remote-state").join("history");

        if !remote_state.exists() {
            tokio::fs::create_dir_all(&remote_state).await?;
        }

        if !file_op.operation.eq(&FileOperation::Deleted) {
            let new_file_path =
                remote_state.join(fastn_core::utils::snapshot_id(file, &(version as u128)));
            let content = tokio::fs::read(root.join(file)).await?;
            fastn_core::utils::update(&new_file_path, content.as_slice()).await?;
        }
    }

    let history_ftd = FileHistory::to_ftd(file_history.values().collect_vec().as_slice());
    tokio::fs::write(
        root.join(".remote-state").join("history.ftd"),
        history_ftd.as_str(),
    )
    .await?;

    Ok(())
}
