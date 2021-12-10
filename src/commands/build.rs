pub async fn build() -> fpm::Result<()> {
    let config = fpm::Config::read().await?;
    tokio::fs::create_dir_all(format!("{}/.build", config.root.as_str()).as_str()).await?;

    for doc in fpm::process_dir(config.root.as_str(), &config, fpm::ignore_history()).await? {
        write(&doc, &config).await?;
    }
    Ok(())
}

async fn write(doc: &fpm::FileFound, config: &fpm::Config) -> fpm::Result<()> {
    // Create the .build folder in case it doesn't exist

    match doc {
        fpm::FileFound::FTDDocument(doc) => process_doc(doc, config, false).await?,
        fpm::FileFound::StaticAsset(sa) => process_static(sa).await?,
        fpm::FileFound::MarkdownDocument(doc) => process_doc(doc, config, true).await?,
    }

    Ok(())
}

pub async fn process_doc(
    doc: &fpm::Document,
    config: &fpm::Config,
    is_md: bool,
) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;

    let ext = if is_md { "md" } else { "ftd" };
    if !(doc.depth == 1 && doc.id.eq(format!("index.{}", ext).as_str())) {
        std::fs::create_dir_all(format!(
            "{}/.build/{}",
            doc.base_path.as_str(),
            doc.id.replace(format!(".{}", ext).as_str(), "")
        ))?;
    }
    let file_rel_path = if doc.id.eq(format!("index.{}", ext).as_str()) {
        "index.html".to_string()
    } else {
        doc.id.replace(format!(".{}", ext).as_str(), "/index.html")
    };
    let lib = fpm::Library {
        markdown: if is_md {
            Some((
                doc.id.as_str().to_string(),
                doc.document.as_str().to_string(),
            ))
        } else {
            None
        },
    };
    let doc_str = if is_md {
        if let Ok(c) = tokio::fs::read_to_string("./FPM/markdown.ftd").await {
            c
        } else {
            let d = indoc::indoc! {"
            -- import: fpm

            -- ftd.text:

            $fpm.markdown-content
            "};
            d.to_string()
        }
    } else {
        doc.document.clone()
    };
    let b = match ftd::p2::Document::from(&doc.id, doc_str.as_str(), &lib) {
        Ok(v) => v,
        Err(e) => {
            return Err(fpm::Error::ConfigurationError {
                message: format!("failed to parse {:?}", &e),
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
