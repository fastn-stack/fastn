pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::Config,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    use itertools::Itertools;
    let root = config.get_root_for_package(&config.package);
    let snapshots = fastn_core::snapshot::get_latest_snapshots(&config.ds.root())
        .await
        .map_err(|_e| ftd::interpreter::Error::ParseError {
            message: "fastn-error: error in package-tree processor `get_latest_snapshots`"
                .to_string(),
            doc_id: doc.name.to_string(),
            line_number: value.line_number(),
        })?;
    let workspaces = fastn_core::snapshot::get_workspace(config)
        .await
        .map_err(|_e| ftd::interpreter::Error::ParseError {
            message: "fastn-error: error in package-tree processor `get_workspace`".to_string(),
            doc_id: doc.name.to_string(),
            line_number: value.line_number(),
        })?;
    let all_files = config
        .get_files(&config.package)
        .await
        .map_err(|_e| ftd::interpreter::Error::ParseError {
            message: "fastn-error: error in package-tree processor `get_files`".to_string(),
            doc_id: doc.name.to_string(),
            line_number: value.line_number(),
        })?
        .into_iter()
        .map(|v| v.get_id().to_string())
        .collect_vec();
    let deleted_files = snapshots
        .keys()
        .filter(|v| !all_files.contains(v))
        .map(|v| v.to_string());

    let mut files = config
        .get_all_file_paths(&config.package)
        .map_err(|_e| ftd::interpreter::Error::ParseError {
            message: "fastn-error: error in package-tree processor `get_all_file_paths1`"
                .to_string(),
            doc_id: doc.name.to_string(),
            line_number: value.line_number(),
        })?
        .into_iter()
        .filter(|v| v.is_file())
        .map(|v| {
            v.strip_prefix(&root)
                .unwrap_or_else(|_| v.as_path())
                .to_string()
                .replace(std::path::MAIN_SEPARATOR.to_string().as_str(), "/")
        })
        .collect_vec();
    files.extend(deleted_files);

    let tree = fastn_core::library::package_tree::construct_tree(
        config,
        files.as_slice(),
        &snapshots,
        &workspaces,
    )
    .await
    .unwrap();
    doc.from_json(&tree, &kind, &value)
}
