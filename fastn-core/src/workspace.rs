#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq, Eq, Clone)]
pub struct WorkspaceEntry {
    pub filename: String,
    pub deleted: Option<bool>,
    pub version: Option<i32>,
    pub cr: Option<usize>,
}

/*pub(crate) struct CRWorkspace {
    pub cr: usize,
    pub workspace: std::collections::BTreeMap<String, fastn_core::workspace::WorkspaceEntry>,
}

impl CRWorkspace {
    fn new(
        cr: usize,
        workspace: std::collections::BTreeMap<String, fastn_core::workspace::WorkspaceEntry>,
    ) -> CRWorkspace {
        CRWorkspace { cr, workspace }
    }

    pub(crate) fn insert_workspace_entry(&mut self, workspace: WorkspaceEntry) {
        let cr_file_path = format!("{}/{}", fastn_core::cr::cr_path(self.cr), workspace.filename);
        let workspace = fastn_core::workspace::WorkspaceEntry {
            filename: cr_file_path,
            deleted: workspace.deleted,
            version: workspace.version,
            cr: Some(self.cr),
        };
        self.workspace
            .insert(workspace.filename.to_string(), workspace);
    }
}*/

impl fastn_core::Config {
    pub(crate) async fn evaluate_clone_workspace(&self) -> fastn_core::Result<Vec<WorkspaceEntry>> {
        use itertools::Itertools;

        let history_list = self.get_history().await?;
        Ok(
            fastn_core::history::FileHistory::get_remote_manifest(history_list.as_slice(), true)?
                .into_iter()
                .map(|(file_name, file_edit)| file_edit.into_workspace(file_name.as_str()))
                .collect_vec(),
        )
    }

    pub(crate) async fn create_clone_workspace(&self) -> fastn_core::Result<()> {
        let workspace_list = self.evaluate_clone_workspace().await?;
        self.write_workspace(workspace_list.as_slice()).await?;
        Ok(())
    }

    pub(crate) async fn write_clone_available_cr(
        &self,
        reserved_crs: &[i32],
    ) -> fastn_core::Result<()> {
        use itertools::Itertools;

        fastn_core::utils::update(
            &self.clone_available_crs_path(),
            reserved_crs
                .iter()
                .map(|v| v.to_string())
                .join("\n")
                .as_bytes(),
        )
        .await?;
        Ok(())
    }

    pub async fn get_available_crs(&self) -> fastn_core::Result<Vec<i32>> {
        let mut response = vec![];
        if self.clone_available_crs_path().exists() {
            let crs = tokio::fs::read_to_string(self.clone_available_crs_path()).await?;
            for cr in crs.split('\n') {
                response.push(cr.parse()?)
            }
        }
        Ok(response)
    }

    pub async fn extract_cr_number(&self) -> fastn_core::Result<i32> {
        let mut available_crs = self.get_available_crs().await?;
        if available_crs.is_empty() {
            return fastn_core::usage_error("No available cr number, try `fastn sync`".to_string());
        }
        let cr_number = available_crs.remove(0);
        self.write_clone_available_cr(available_crs.as_slice())
            .await?;
        Ok(cr_number)
    }

    pub(crate) async fn read_workspace(&self) -> fastn_core::Result<Vec<WorkspaceEntry>> {
        let workspace = {
            let workspace = tokio::fs::read_to_string(self.workspace_file());
            let lib = fastn_core::FastnLibrary::default();
            fastn_core::doc::parse_ftd("fastn", workspace.await?.as_str(), &lib)?
        };
        Ok(workspace.get("fastn#client-workspace")?)
    }

    pub(crate) async fn get_workspace_map(
        &self,
    ) -> fastn_core::Result<std::collections::BTreeMap<String, fastn_core::workspace::WorkspaceEntry>>
    {
        Ok(self
            .read_workspace()
            .await?
            .into_iter()
            .map(|v| (v.filename.to_string(), v))
            .collect())
    }

    pub(crate) async fn get_clone_workspace(
        &self,
    ) -> fastn_core::Result<std::collections::BTreeMap<String, fastn_core::workspace::WorkspaceEntry>>
    {
        Ok(self
            .get_workspace_map()
            .await?
            .into_iter()
            // .filter(|(_, v)| v.cr.is_none())
            .collect())
    }

    /*pub(crate) async fn get_cr_workspace(&self, cr_number: usize) -> fastn_core::Result<CRWorkspace> {
        let workspace = self
            .get_workspace_map()
            .await?
            .into_iter()
            .filter_map(|(k, v)| match v.cr {
                Some(ref cr) if cr_number.eq(cr) => {
                    Some((self.cr_path_to_file_name(cr_number, k.as_str())?, v))
                }
                _ => None,
            })
            .collect::<std::collections::BTreeMap<String, fastn_core::workspace::WorkspaceEntry>>();
        Ok(CRWorkspace::new(cr_number, workspace))
    }*/

    pub(crate) async fn write_workspace(
        &self,
        workspace: &[WorkspaceEntry],
    ) -> fastn_core::Result<()> {
        let workspace_path = self.workspace_file();
        fastn_core::utils::update(
            &workspace_path,
            WorkspaceEntry::get_ftd_string(workspace).as_bytes(),
        )
        .await?;
        Ok(())
    }

    pub(crate) async fn update_workspace(
        &self,
        workspace: Vec<WorkspaceEntry>,
    ) -> fastn_core::Result<()> {
        use itertools::Itertools;

        let workspace = {
            let mut initial_workspace = self.get_workspace_map().await?;
            initial_workspace.extend(workspace.into_iter().map(|v| (v.filename.to_string(), v)));
            initial_workspace
        };
        self.write_workspace(workspace.into_values().collect_vec().as_slice())
            .await
    }
}

impl WorkspaceEntry {
    fn get_ftd_string(workspace: &[Self]) -> String {
        let mut workspace_data = "-- import: fastn".to_string();
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
            let cr = if let Some(cr) = workspace_entry.cr {
                format!("cr: {}\n", cr)
            } else {
                "".to_string()
            };
            workspace_data = format!(
                "{}\n\n-- fastn.client-workspace: {}\n{}{}{}",
                workspace_data, workspace_entry.filename, version, deleted, cr
            );
        }
        workspace_data
    }

    pub(crate) fn set_deleted(&mut self) {
        self.deleted = Some(true);
    }
}
