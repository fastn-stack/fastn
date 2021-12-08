pub async fn build() -> fpm::Result<()> {
    let config = fpm::Config::read().await?;
    tokio::fs::create_dir_all(format!("{}/.build", config.root.as_str()).as_str()).await?;

    for doc in fpm::process_dir(config.root.as_str(), &config, None).await? {
        write(&doc, &config).await?;
    }
    Ok(())
}

async fn write(doc: &fpm::FileFound, config: &fpm::Config) -> fpm::Result<()> {
    // Create the .build folder in case it doesn't exist

    match doc {
        fpm::FileFound::FTDDocument(doc) => process_doc(doc, config).await?,
        fpm::FileFound::StaticAsset(sa) => process_static(sa).await?,
    }

    Ok(())
}

pub async fn process_doc(doc: &fpm::Document, config: &fpm::Config) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;
    if !(doc.depth == 1 && doc.id.eq("index.ftd")) {
        std::fs::create_dir_all(format!(
            "{}/.build/{}",
            doc.base_path.as_str(),
            doc.id.replace(".ftd", "")
        ))?;
    }
    let file_rel_path = if doc.id.eq("index.ftd") {
        "index.html".to_string()
    } else {
        doc.id.replace(".ftd", "/index.html")
    };

    let lib = fpm::Library {};
    let b = match ftd::p2::Document::from(&doc.id, &doc.document, &lib) {
        Ok(v) => v,
        Err(e) => {
            return Err(fpm::Error::ConfigurationParseError {
                message: format!("failed to parse {:?}", &e),
                line_number: 0,
            });
        }
    };
    let new_file_path = format!(
        "{}/.build/{}",
        doc.base_path.as_str(),
        file_rel_path.as_str()
    );
    let mut f = tokio::fs::File::create(new_file_path.as_str()).await?;

    let ftd_doc = b.to_rt("main", &doc.id);

    f.write_all(
        ftd::html()
            .replace(
                "__ftd_data__",
                serde_json::to_string_pretty(&ftd_doc.data)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .replace(
                "__ftd_external_children__",
                serde_json::to_string_pretty(&ftd_doc.external_children)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .replace(
                "__ftd__",
                format!(
                    "{}{}",
                    b.html("main", &doc.id).as_str(),
                    config.get_font_style(),
                )
                .as_str(),
            )
            .as_str()
            .replace("__ftd_js__", ftd::js())
            .as_bytes(),
    )
    .await?;
    println!("Generated {}", file_rel_path.as_str(),);
    Ok(())
}

pub async fn process_static(sa: &fpm::StaticAsset) -> fpm::Result<()> {
    if sa.depth != 1 {
        std::fs::create_dir_all(format!(
            "{}/.build/{}",
            sa.base_path.as_str(),
            sa.id
                .rsplit_once("/")
                .unwrap_or_else(|| ("", sa.id.as_str()))
                .0
        ))?;
    }
    std::fs::copy(
        format!("{}/{}", sa.base_path, sa.id),
        format!("{}/.build/{}", sa.base_path, sa.id),
    )?;
    Ok(())
}
