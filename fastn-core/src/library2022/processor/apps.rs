pub fn process(
    value: ftd_ast::VariableValue,
    kind: fastn_resolved::Kind,
    doc: &ftd::interpreter::TDoc,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<fastn_resolved::Value> {
    use itertools::Itertools;
    #[derive(Debug, serde::Serialize)]
    struct UiApp {
        name: String,
        package: String,
        #[serde(rename = "url")]
        url: String,
        icon: Option<ftd::ImageSrc>,
    }

    let apps = req_config
        .config
        .package
        .apps
        .iter()
        .map(|a| UiApp {
            name: a.name.clone(),
            package: a.package.name.clone(),
            url: a.mount_point.to_string(),
            icon: a.package.icon.clone(),
        })
        .collect_vec();

    let installed_apps = fastn_core::ds::LengthList::from_owned(apps);
    doc.from_json(&installed_apps, &kind, &value)
}
