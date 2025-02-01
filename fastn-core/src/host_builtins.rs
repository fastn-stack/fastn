/// Calling: `ftd.app-url(path = /test/)` in an ftd file of a mounted app will return the path
/// prefixed with the `mountpoint` of the app.
///
/// The `path` arg must start with a forward slash (/)
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
/// -- ftd.text: $ftd.app-url(path = /test/)
/// ```
///
/// Visiting `/app/` in browser should render /app/test/
#[inline]
pub fn app_path(
    config: &fastn_core::Config,
    req_path: &str,
) -> (String, fastn_resolved::Definition) {
    let app_system_name = config
        .package
        .apps
        .iter()
        .find(|a| req_path.starts_with(&a.mount_point))
        .and_then(|a| a.package.system.clone())
        .unwrap_or_default();

    let name = "ftd#app-url".to_string();
    let def = fastn_resolved::Definition::Function(fastn_resolved::Function {
        name: name.clone(),
        return_kind: fastn_resolved::KindData {
            kind: fastn_resolved::Kind::string(),
            caption: false,
            body: false,
        },
        arguments: vec![
            fastn_resolved::Argument {
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
            },
            fastn_resolved::Argument {
                name: "app".to_string(),
                kind: fastn_resolved::KindData::new(fastn_resolved::Kind::string()),
                mutable: false,
                value: Some(fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::String {
                        text: app_system_name,
                    },
                    is_mutable: false,
                    line_number: 0,
                }),
                access_modifier: Default::default(),
                line_number: 0,
            },
        ],
        expression: vec![fastn_resolved::FunctionExpression {
            expression: "ftd.app_url_ex(path, app)".to_string(),
            line_number: 0,
        }],
        js: None,
        line_number: 0,
        external_implementation: false,
    });

    (name, def)
}

/// Ftd string variable that holds the name of the package.
///
/// Useful to determine if the package is run standalone or as a dependency:
#[inline]
pub fn main_package(config: &fastn_core::Config) -> (String, fastn_resolved::Definition) {
    let name = "ftd#main-package".to_string();
    let def = fastn_resolved::Definition::Variable(fastn_resolved::Variable {
        name: name.clone(),
        kind: fastn_resolved::Kind::string().into_kind_data(),
        value: fastn_resolved::PropertyValue::Value {
            value: fastn_resolved::Value::String {
                text: config.package.name.clone(),
            },
            is_mutable: false,
            line_number: 0,
        },
        conditional_value: vec![],
        mutable: false,
        is_static: false,
        line_number: 0,
    });

    (name, def)
}

/// Ftd string variable that holds the `fastn.app` mounts
///
/// Used by `ftd.app-url` to determine the mountpoint of the app
#[inline]
pub fn app_mounts(config: &fastn_core::Config) -> (String, fastn_resolved::Definition) {
    let name = "ftd#app-urls".to_string();
    let variants = config
        .app_mounts()
        .unwrap_or_default()
        .into_iter()
        .map(|(k, v)| {
            fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                &k,
                fastn_resolved::Kind::string().into_kind_data().caption(),
                false,
                Some(fastn_resolved::Value::new_string(&v).into_property_value(false, 0)),
                0,
            ))
        })
        .collect();

    let def = fastn_resolved::Definition::OrType(fastn_resolved::OrType {
        name: name.clone(),
        line_number: 0,
        variants,
    });

    (name, def)
}
