pub enum FastnPackage<'a> {
    Custom(&'a str),
    Default,
}

pub fn run(slug: FastnPackage) {
    // fastn main has setup tokio so we make tauri use that
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let slug = match slug {
        FastnPackage::Custom(s) => s,
        FastnPackage::Default => super::DEFAULT_UI_PKG,
    }
    .to_string();

    tauri::Builder::default()
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let slug = slug.clone();
            let handle = app.handle().clone();
            tokio::task::spawn(setup(handle, slug));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub async fn run_backend_server(slug: &str, pkg_dir: &std::path::Path) -> fastn_core::Result<u16> {
    log::info!("setting up pkg");
    log::info!("{:?}", pkg_dir);

    if !std::fs::exists(&pkg_dir)? {
        use std::io::Write;

        let url = format!("https://www.fifthtry.com/{}.zip", slug);
        log::info!("Downloading package from: {}", url);

        let tmp_dir = tempfile::tempdir()?;
        let zip_path = tmp_dir.path().join(format!("{}.zip", slug));

        log::info!("{:?}", zip_path);

        let mut zip_file = std::fs::File::create(&zip_path)?;

        let response = reqwest::get(&url).await?;
        let bytes = response.bytes().await?;
        zip_file.write_all(&bytes)?;
        zip_file.sync_all()?;

        std::fs::create_dir_all(&pkg_dir)?;
        let zip_file = std::fs::File::open(&zip_path)?;
        let mut archive = zip::ZipArchive::new(zip_file)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = pkg_dir.join(file.name());
            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
        // tempdir will be cleaned up automatically
    }

    log::info!(
        "Package directory exists at: {:?}. Launcing fastn serve",
        pkg_dir
    );

    let pkg_dir = camino::Utf8Path::from_path(&pkg_dir).unwrap();
    let pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>> =
        actix_web::web::Data::new(scc::HashMap::new());
    let ds = fastn_ds::DocumentStore::new(pkg_dir, pg_pools);

    log::info!("running fastn update");
    fastn_update::update(&ds, false).await.unwrap();
    log::info!("ran fastn update");

    let config = fastn_core::Config::read(ds, false, &None).await.unwrap();

    log::info!("read config");

    let (server, port) =
        fastn_core::commands::serve::make_server(std::sync::Arc::new(config), "127.0.0.1", None)
            .await?;

    log::info!("started server");

    log::info!("Fastn server is running on port: {}", port);
    tokio::task::spawn(server);

    log::info!("Fastn server is running");

    Ok(port)
}

async fn setup(app: tauri::AppHandle, slug: String) {
    use tauri::Manager;

    log::info!("Setting up fastn UI...");

    let pkg_dir = {
        let mut data_dir = app
            .path()
            .app_data_dir()
            .inspect_err(|e| {
                log::error!("Failed to get app data dir: {e}");
            })
            .unwrap();
        data_dir.push(&slug);
        data_dir
    };

    let port = run_backend_server(&slug, &pkg_dir)
        .await
        .inspect_err(|e| {
            log::error!("Failed to setup package: {e}");
        })
        .unwrap();

    let url = format!("http://127.0.0.1:{port}").parse().unwrap();

    let window = app
        .get_webview_window("main")
        .expect("Failed to get main window");

    window.navigate(url).unwrap();
}
