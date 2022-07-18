use itertools::Itertools;

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq)]
pub struct WorkspaceEntry {
    pub filename: String,
    pub deleted: Option<bool>,
    pub version: i32,
}

impl fpm::Config {
    pub(crate) async fn evaluate_client_workspace(&self) -> fpm::Result<Vec<WorkspaceEntry>> {
        let history_list = self.get_history().await?;
        Ok(
            fpm::history::FileHistory::get_latest_file_edits(history_list.as_slice())?
                .into_iter()
                .map(|(file_name, file_edit)| WorkspaceEntry {
                    filename: file_name,
                    deleted: None,
                    version: file_edit.version,
                })
                .collect_vec(),
        )
    }

    pub(crate) async fn create_client_workspace(&self) -> fpm::Result<Vec<WorkspaceEntry>> {
        let workspace_list = self.evaluate_client_workspace().await?;
        let workspace_path = self.client_workspace_file();
        fpm::utils::update(
            &workspace_path,
            WorkspaceEntry::get_ftd_string(workspace_list.as_slice()).as_bytes(),
        )
        .await?;
        Ok(workspace_list)
    }
}

impl WorkspaceEntry {
    fn get_ftd_string(workspace: &[Self]) -> String {
        let mut workspace_data = "-- import: fpm".to_string();
        for workspace_entry in workspace {
            let deleted = if let Some(deleted) = workspace_entry.deleted {
                format!("deleted: {}", deleted)
            } else {
                "".to_string()
            };
            workspace_data = format!(
                "{}\n\n-- fpm.client-workspace: {}\nversion: {}\n{}",
                workspace_data, workspace_entry.filename, workspace_entry.version, deleted
            );
        }
        workspace_data
    }
}
