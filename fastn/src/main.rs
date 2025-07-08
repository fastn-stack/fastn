// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub fn main() {
    fastn_observer::observe();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(outer_main())
}

async fn outer_main() {
    if let Err(e) = async_main().await {
        eprintln!("{e:?}");
        std::process::exit(1);
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("FastnCoreError: {}", _0)]
    FastnCoreError(#[from] fastn_core::Error),
}

async fn async_main() -> Result<(), Error> {
    #[allow(unused_mut)]
    let mut app = fastn_lib::cmd();

    #[cfg(feature = "fifthtry")]
    {
        app = clift::attach_cmd(app);
    }

    let matches = app.get_matches();

    // TODO: figure out how to load .env files for packages run from fastn UI
    fastn_lib::set_env_vars(matches.subcommand_matches("test").is_some());

    futures::try_join!(
        // launches a UI with the [package_name] provided
        // no args passed to the cli means a UI launch with the default pkg
        fastn_lib::fastn_ui_cli(&matches),
        // handles subcmds like "serve", "test" etc
        fastn_lib::fastn_core_commands(&matches),
        fastn_lib::check_for_update_cmd(&matches)
    )?;

    Ok(())
}
