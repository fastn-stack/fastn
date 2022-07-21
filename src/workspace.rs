use itertools::Itertools;

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq, Clone)]
pub struct WorkspaceEntry {
    pub filename: String,
    pub deleted: Option<bool>,
    pub version: Option<i32>,
}

impl fpm::Config {
    pub(crate) async fn evaluate_client_workspace(&self) -> fpm::Result<Vec<WorkspaceEntry>> {
        let history_list = self.get_history().await?;
        Ok(
            fpm::history::FileHistory::get_latest_file_edits(history_list.as_slice())?
                .into_iter()
                .map(|(file_name, file_edit)| file_edit.to_workspace(file_name.as_str()))
                .collect_vec(),
        )
    }

    pub(crate) async fn create_client_workspace(&self) -> fpm::Result<()> {
        let workspace_list = self.evaluate_client_workspace().await?;
        self.write_workspace(workspace_list.as_slice()).await?;
        Ok(())
    }

    pub(crate) async fn read_workspace(&self) -> fpm::Result<Vec<WorkspaceEntry>> {
        let workspace = {
            let workspace = tokio::fs::read_to_string(self.workspace_file());
            let lib = fpm::FPMLibrary::default();
            fpm::doc::parse_ftd("FPM", workspace.await?.as_str(), &lib)?
        };
        Ok(workspace.get("fpm#client-workspace")?)
    }

    pub(crate) async fn write_workspace(&self, workspace: &[WorkspaceEntry]) -> fpm::Result<()> {
        let workspace_path = self.workspace_file();
        fpm::utils::update(
            &workspace_path,
            WorkspaceEntry::get_ftd_string(workspace).as_bytes(),
        )
        .await?;
        Ok(())
    }
}

impl WorkspaceEntry {
    fn get_ftd_string(workspace: &[Self]) -> String {
        let mut workspace_data = "-- import: fpm".to_string();
        for workspace_entry in workspace {
            let deleted = if let Some(deleted) = workspace_entry.deleted {
                format!("deleted: {}\n", deleted)
            } else {
                "".to_string()
            };
            let version = if let Some(version) = workspace_entry.version {
                format!("version: {}\n", version)
            } else {
                "".to_string()
            };
            workspace_data = format!(
                "{}\n\n-- fpm.client-workspace: {}\n{}{}",
                workspace_data, workspace_entry.filename, version, deleted
            );
        }
        workspace_data
    }

    pub(crate) fn set_deleted(&mut self) {
        self.deleted = Some(true);
    }
}
