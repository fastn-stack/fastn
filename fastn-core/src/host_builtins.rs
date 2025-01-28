/// Calling: `ftd.app-path(path = /test/)` in an ftd file of a mounted app will return the path
/// prefixed with the `mountpoint` of the app.
///
/// The `path` arg must start with a forwar slash (/)
///
/// # Example
///
/// ```FASTN.ftd
/// -- import: fastn
///
/// -- fastn.package: test
///
/// -- fastn.app: Test
/// mountpoint: /app/
/// package: some-test-app.fifthtry.site
/// ```
///
/// ```some-test-app.fifthtry.site/index.ftd
///
/// -- ftd.text: $ftd.app-path(path = /test/)
/// ```
///
/// Visiting `/app/` in browser should render /app/test/
#[inline]
pub fn app_path(pkg: &fastn_core::Package, req_path: &str) -> fastn_resolved::Definition {
    let prefix = pkg
        .apps
        .iter()
        .find(|a| req_path.starts_with(&a.mount_point))
        .map(|a| a.mount_point.clone())
        .unwrap_or_default();
    let prefix = prefix.trim_end_matches('/');

    fastn_resolved::Definition::Function(fastn_resolved::Function {
        name: "ftd#app-path".to_string(),
        return_kind: fastn_resolved::KindData {
            kind: fastn_resolved::Kind::string(),
            caption: false,
            body: false,
        },
        arguments: vec![fastn_resolved::Argument {
            name: "path".to_string(),
            kind: fastn_resolved::KindData {
                kind: fastn_resolved::Kind::string(),
                caption: false,
                body: false,
            },
            mutable: false,
            value: None,
            access_modifier: Default::default(),
            line_number: 0,
        }],
        expression: vec![fastn_resolved::FunctionExpression {
            expression: format!("\"{}\" + path", prefix),
            line_number: 0,
        }],
        js: None,
        line_number: 0,
        external_implementation: false,
    })
}
