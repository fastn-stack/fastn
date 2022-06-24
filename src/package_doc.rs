impl fpm::Package {
    pub(crate) async fn fs_fetch_by_file_name(
        &self,
        name: &str,
        package_root: Option<&camino::Utf8PathBuf>,
    ) -> fpm::Result<Vec<u8>> {
        let package_root = if let Some(package_root) = package_root {
            package_root.to_owned()
        } else {
            match self.fpm_path.as_ref() {
                Some(path) if path.parent().is_some() => path.parent().unwrap().to_path_buf(),
                _ => {
                    return Err(fpm::Error::PackageError {
                        message: format!("package root not found. Package: {}", &self.name),
                    })
                }
            }
        };
        Ok(tokio::fs::read(package_root.join(name)).await?)
    }

    pub(crate) async fn fs_fetch_by_id(
        &self,
        id: &str,
        package_root: Option<&camino::Utf8PathBuf>,
    ) -> fpm::Result<(String, Vec<u8>)> {
        for name in file_id_to_names(id) {
            if let Ok(data) = self
                .fs_fetch_by_file_name(name.as_str(), package_root)
                .await
            {
                return Ok((name, data));
            }
        }
        Err(fpm::Error::PackageError {
            message: format!(
                "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {}",
                id, &self.name
            ),
        })
    }

    async fn http_fetch_by_file_name(&self, name: &str) -> fpm::Result<Vec<u8>> {
        let base = self.base.as_ref().ok_or_else(|| fpm::Error::PackageError {
            message: format!(
                "package base not found. Package: {}, File: {}",
                &self.name, name
            ),
        })?;
        fpm::utils::construct_url_and_get(
            format!("{}/{}", base.trim_end_matches('/'), name).as_str(),
        )
        .await
    }

    async fn http_fetch_by_id(&self, id: &str) -> fpm::Result<(String, Vec<u8>)> {
        for name in file_id_to_names(id) {
            if let Ok(data) = self.http_fetch_by_file_name(name.as_str()).await {
                return Ok((name, data));
            }
        }

        Err(fpm::Error::PackageError {
            message: format!(
                "http_fetch_by_id:: Corresponding file not found for id: {}. Package: {}",
                id, &self.name
            ),
        })
    }

    pub(crate) async fn http_download_by_id(
        &self,
        id: &str,
        package_root: Option<&camino::Utf8PathBuf>,
    ) -> fpm::Result<(String, Vec<u8>)> {
        let package_root = if let Some(package_root) = package_root {
            package_root.to_owned()
        } else {
            match self.fpm_path.as_ref() {
                Some(path) if path.parent().is_some() => path.parent().unwrap().to_path_buf(),
                _ => {
                    return Err(fpm::Error::PackageError {
                        message: format!("package root not found. Package: {}", &self.name),
                    })
                }
            }
        };

        let (file_path, data) = self.http_fetch_by_id(id).await?;
        fpm::utils::write(&package_root, file_path.as_str(), data.as_slice()).await?;

        Ok((file_path, data))
    }

    pub(crate) async fn http_download_by_file_name(
        &self,
        file_path: &str,
        package_root: Option<&camino::Utf8PathBuf>,
    ) -> fpm::Result<Vec<u8>> {
        let package_root = if let Some(package_root) = package_root {
            package_root.to_owned()
        } else {
            match self.fpm_path.as_ref() {
                Some(path) if path.parent().is_some() => path.parent().unwrap().to_path_buf(),
                _ => {
                    return Err(fpm::Error::PackageError {
                        message: format!("package root not found. Package: {}", &self.name),
                    })
                }
            }
        };

        let data = self.http_fetch_by_file_name(file_path).await?;
        fpm::utils::write(&package_root, file_path, data.as_slice()).await?;

        Ok(data)
    }

    pub(crate) async fn resolve_by_file_name(
        &self,
        file_path: &str,
        package_root: Option<&camino::Utf8PathBuf>,
        restore_default: bool,
    ) -> fpm::Result<Vec<u8>> {
        if let Ok(response) = self.fs_fetch_by_file_name(file_path, package_root).await {
            return Ok(response);
        }
        if let Ok(response) = self
            .http_download_by_file_name(file_path, package_root)
            .await
        {
            return Ok(response);
        }

        if !restore_default {
            return Err(fpm::Error::PackageError {
                message: format!(
                    "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {}",
                    file_path, &self.name
                ),
            });
        }

        let new_file_path = match file_path.rsplit_once('.') {
            Some((remaining, ext))
                if mime_guess::MimeGuess::from_ext(ext)
                    .first_or_octet_stream()
                    .to_string()
                    .starts_with("image/")
                    && remaining.ends_with("-dark") =>
            {
                format!("{}.{}", remaining.trim_end_matches("-dark"), ext)
            }
            _ => {
                return Err(fpm::Error::PackageError {
                    message: format!(
                        "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {}",
                        file_path, &self.name
                    ),
                })
            }
        };

        let root = if let Some(package_root) = package_root {
            package_root.to_owned()
        } else {
            match self.fpm_path.as_ref() {
                Some(path) if path.parent().is_some() => path.parent().unwrap().to_path_buf(),
                _ => {
                    return Err(fpm::Error::PackageError {
                        message: format!("package root not found. Package: {}", &self.name),
                    })
                }
            }
        };

        if let Ok(response) = self
            .fs_fetch_by_file_name(new_file_path.as_str(), package_root)
            .await
        {
            tokio::fs::copy(root.join(new_file_path), root.join(file_path)).await?;
            return Ok(response);
        }

        match self
            .http_download_by_file_name(new_file_path.as_str(), package_root)
            .await
        {
            Ok(response) => {
                tokio::fs::copy(root.join(new_file_path), root.join(file_path)).await?;
                Ok(response)
            }
            Err(e) => Err(e),
        }
    }

    pub(crate) async fn resolve_by_id(
        &self,
        id: &str,
        package_root: Option<&camino::Utf8PathBuf>,
    ) -> fpm::Result<(String, Vec<u8>)> {
        if let Ok(response) = self.fs_fetch_by_id(id, package_root).await {
            return Ok(response);
        }

        if let Ok(response) = self.http_download_by_id(id, package_root).await {
            return Ok(response);
        }

        let new_id = match id.rsplit_once('.') {
            Some((remaining, ext))
                if mime_guess::MimeGuess::from_ext(ext)
                    .first_or_octet_stream()
                    .to_string()
                    .starts_with("image/")
                    && remaining.ends_with("-dark") =>
            {
                format!("{}.{}", remaining.trim_end_matches("-dark"), ext)
            }
            _ => {
                return Err(fpm::Error::PackageError {
                    message: format!(
                        "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {}",
                        id, &self.name
                    ),
                })
            }
        };

        let root = if let Some(package_root) = package_root {
            package_root.to_owned()
        } else {
            match self.fpm_path.as_ref() {
                Some(path) if path.parent().is_some() => path.parent().unwrap().to_path_buf(),
                _ => {
                    return Err(fpm::Error::PackageError {
                        message: format!("package root not found. Package: {}", &self.name),
                    })
                }
            }
        };

        if let Ok(response) = self.fs_fetch_by_id(new_id.as_str(), package_root).await {
            tokio::fs::copy(root.join(new_id), root.join(id)).await?;
            return Ok(response);
        }

        match self
            .http_download_by_id(new_id.as_str(), package_root)
            .await
        {
            Ok(response) => {
                tokio::fs::copy(root.join(new_id), root.join(id)).await?;
                Ok(response)
            }
            Err(e) => Err(e),
        }
    }
}

fn file_id_to_names(id: &str) -> Vec<String> {
    let id = id.replace("/index.html", "/").replace("index.html", "/");
    if id.eq("/") {
        return vec![
            "index.ftd".to_string(),
            "README.md".to_string(),
            "index.md".to_string(),
        ];
    }
    let mut ids = vec![];
    if !id.ends_with('/') {
        ids.push(id.trim_matches('/').to_string());
    }
    let id = id.trim_matches('/').to_string();
    ids.extend([
        format!("{}.ftd", id),
        format!("{}/index.ftd", id),
        format!("{}.md", id),
        format!("{}/README.md", id),
        format!("{}/index.md", id),
    ]);
    ids
}

pub(crate) async fn read_ftd(
    config: &mut fpm::Config,
    main: &fpm::Document,
    base_url: &str,
    download_assets: bool,
) -> fpm::Result<Vec<u8>> {
    let current_package = config
        .all_packages
        .get(main.package_name.as_str())
        .unwrap_or(&config.package);

    let mut lib = fpm::Library2 {
        config: config.clone(),
        markdown: None,
        document_id: main.id.clone(),
        translated_data: Default::default(),
        base_url: base_url.to_string(),
        packages_under_process: vec![current_package.name.to_string()],
    };

    let main_ftd_doc = match fpm::doc::parse2(
        main.id_with_package().as_str(),
        current_package
            .get_prefixed_body(main.content.as_str(), &main.id, true)
            .as_str(),
        &mut lib,
        base_url,
        download_assets,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return Err(fpm::Error::PackageError {
                message: format!("failed to parse {:?}", &e),
            });
        }
    };

    config.all_packages.extend(lib.config.all_packages);
    config
        .downloaded_assets
        .extend(lib.config.downloaded_assets);

    let doc_title = match &main_ftd_doc.title() {
        Some(x) => x.original.clone(),
        _ => main.id.as_str().to_string(),
    };
    let ftd_doc = main_ftd_doc.to_rt("main", &main.id);

    let file_content = fpm::utils::replace_markers(
        fpm::ftd_html(),
        config,
        main.id_to_path().as_str(),
        doc_title.as_str(),
        base_url,
        &ftd_doc,
    );

    Ok(file_content.into())
}

pub(crate) async fn process_ftd(
    config: &mut fpm::Config,
    main: &fpm::Document,
    base_url: &str,
) -> fpm::Result<Vec<u8>> {
    let current_package = config
        .all_packages
        .get(main.package_name.as_str())
        .unwrap_or(&config.package);

    if main.id.eq("FPM.ftd") {
        tokio::fs::copy(
            config.root.join(main.id.as_str()),
            config.root.join(".build").join(main.id.as_str()),
        )
        .await?;
    }

    let main = {
        let mut main = main.to_owned();
        if main.id.eq("FPM.ftd") {
            main.id = "-.ftd".to_string();
            let path = config.root.join("FPM").join("info.ftd");
            main.content = if path.is_file() {
                std::fs::read_to_string(path)?
            } else {
                let package_info_package = match config
                    .package
                    .get_dependency_for_interface(fpm::PACKAGE_INFO_INTERFACE)
                    .or_else(|| {
                        config
                            .package
                            .get_dependency_for_interface(fpm::PACKAGE_THEME_INTERFACE)
                    }) {
                    Some(dep) => dep.package.name.as_str(),
                    None => fpm::PACKAGE_INFO_INTERFACE,
                };
                config.package.get_prefixed_body(
                    format!(
                        "-- import: {}/package-info as pi\n\n-- pi.package-info-page:",
                        package_info_package
                    )
                    .as_str(),
                    &main.id,
                    true,
                )
            }
        } else {
            main.content = current_package.get_prefixed_body(main.content.as_str(), &main.id, true);
        }
        main
    };

    let file_rel_path = if main.id.contains("index.ftd") {
        main.id.replace("index.ftd", "index.html")
    } else {
        main.id.replace(
            ".ftd",
            format!("{}index.html", std::path::MAIN_SEPARATOR).as_str(),
        )
    };

    let response = read_ftd(config, &main, base_url, true).await?;
    fpm::utils::write(
        &config.build_dir(),
        file_rel_path.as_str(),
        response.as_slice(),
    )
    .await?;

    Ok(response)
}
