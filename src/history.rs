use itertools::Itertools;

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq)]
pub struct FileHistory {
    pub filename: String,
    pub file_edit: Vec<FileEdit>,
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq, Clone)]
pub struct FileEdit {
    pub message: String,
    pub timestamp: u128,
    pub version: i32,
    pub author: Option<String>,
    pub src_cr: Option<i32>,
    pub operation: FileOperation,
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq, Clone)]
pub enum FileOperation {
    Added,
    Updated,
    Deleted,
    Merged,
}

impl fpm::Config {
    pub async fn get_history(&self) -> fpm::Result<Vec<FileHistory>> {
        let history_file_path = self.history_file();
        let doc = {
            let history_content = tokio::fs::read_to_string(history_file_path).await?;
            let lib = fpm::FPMLibrary::default();
            fpm::doc::parse_ftd("history.ftd", history_content.as_str(), &lib)?
        };
        Ok(doc.get("fpm#history")?)
    }

    pub async fn get_latest_file_paths(&self) -> fpm::Result<Vec<camino::Utf8PathBuf>> {
        let history_list = self.get_history().await?;
        Ok(
            fpm::history::FileHistory::get_latest_file_edits(history_list.as_slice())?
                .iter()
                .map(|(file_name, file_edit)| self.history_path(file_name, file_edit.version))
                .collect_vec(),
        )
    }
}

impl FileHistory {
    pub(crate) fn get_latest_file_edits(list: &[Self]) -> fpm::Result<Vec<(String, FileEdit)>> {
        Ok(list
            .iter()
            .filter_map(|v| {
                v.get_latest_file_edit()
                    .map(|file_edit| (v.filename.to_string(), file_edit))
            })
            .collect_vec())
    }

    fn get_latest_file_edit(&self) -> Option<FileEdit> {
        for file_edit in self.file_edit.iter() {
            if file_edit.operation.eq(&FileOperation::Merged) {
                return Some(file_edit.clone());
            }
            if file_edit.src_cr.is_none() {
                match file_edit.operation {
                    FileOperation::Added | FileOperation::Merged | FileOperation::Updated => {
                        return Some(file_edit.clone());
                    }
                    FileOperation::Deleted => {
                        return None;
                    }
                }
            }
        }
        None
    }
}
