pub async fn query(
    config: &fastn_core::Config,
    stage: &str,
    path: Option<&str>,
    with_null: bool,
) -> fastn_core::Result<()> {
    let documents = std::collections::BTreeMap::from_iter(
        config
            .get_files(&config.package, &None)
            .await?
            .into_iter()
            .map(|v| (v.get_id().to_string(), v)),
    );

    if let Some(path) = path {
        let file = documents.values().find(|v| v.get_id().eq(path)).ok_or(
            fastn_core::Error::UsageError {
                message: format!("{path} not found in the package"),
            },
        )?;

        let value = get_ftd_json(file, stage)?;
        println!(
            "{}",
            if with_null {
                fastn_core::utils::value_to_colored_string(&value, 1)
            } else {
                fastn_core::utils::value_to_colored_string_without_null(&value, 1)
            }
        );

        return Ok(());
    }
    let mut values: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for file in documents.values() {
        if file.is_ftd() {
            let value = get_ftd_json(file, stage)?;
            values.insert(file.get_id().to_string(), value);
        }
    }

    let value = serde_json::Value::Object(values);

    println!(
        "{}",
        if with_null {
            fastn_core::utils::value_to_colored_string(&value, 1)
        } else {
            fastn_core::utils::value_to_colored_string_without_null(&value, 1)
        }
    );

    Ok(())
}

pub(crate) fn get_ftd_json(
    file: &fastn_core::File,
    stage: &str,
) -> fastn_core::Result<serde_json::Value> {
    let document = if let fastn_core::File::Ftd(document) = file {
        document
    } else {
        return Err(fastn_core::Error::UsageError {
            message: format!("{} is not an ftd file", file.get_id()),
        });
    };

    match stage {
        "p1" => get_p1_json(document),
        "ast" => get_ast_json(document),
        _ => unimplemented!(),
    }
}

fn get_p1_json(document: &fastn_core::Document) -> fastn_core::Result<serde_json::Value> {
    let p1 = ftd_p1::parse(
        document.content.as_str(),
        document.id_with_package().as_str(),
    )?;
    let value = serde_json::to_value(p1)?;

    Ok(value)
}

fn get_ast_json(document: &fastn_core::Document) -> fastn_core::Result<serde_json::Value> {
    let id = document.id_with_package();
    let p1 = ftd_p1::parse(document.content.as_str(), id.as_str())?;

    let ast = ftd_ast::Ast::from_sections(p1.as_slice(), id.as_str())?;
    let value = serde_json::to_value(ast)?;

    Ok(value)
}
