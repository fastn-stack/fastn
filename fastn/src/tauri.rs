#[allow(unexpected_cfgs)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(fastn_serve_port: u16) {
    // fastn main has setup tokio so we make tauri use that
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    tauri::Builder::default()
        .setup(move |app| {
            use tauri::Manager;

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let webview = app
                .get_webview_window("main")
                .expect("main label is created by tauri");

            let url = format!("http://localhost:{fastn_serve_port}")
                .parse()
                .unwrap();

            webview
                .navigate(url)
                .expect("navigation to fastn served url failed");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
