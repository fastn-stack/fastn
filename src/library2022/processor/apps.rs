pub fn process<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fastn::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    use itertools::Itertools;
    #[derive(Debug, serde::Serialize)]
    struct UiApp {
        name: String,
        package: String,
        #[serde(rename = "url")]
        url: String,
        icon: Option<ftd::ImageSrc>,
    }

    let apps = config
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

    let installed_apps = fastn::ds::LengthList::from_owned(apps);
    doc.from_json(&installed_apps, &kind, value.line_number())
}
