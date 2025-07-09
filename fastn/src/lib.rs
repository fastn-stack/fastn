mod tauri;

// TODO: use lets-os
const DEFAULT_UI_PKG: &str = "design-system";

/// WARN: Do not call this function!
/// The main entry point for the fastn UI application.
#[allow(unexpected_cfgs, unused)]
#[cfg_attr(mobile, tauri_macros::mobile_entry_point)]
fn fastn_app_entrypoint() {
    async fn inner() {
        log::info!("=========================== FASTN UI ============================");
        tauri::run(tauri::FastnPackage::Default);
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(inner())
}

/// Launch a native window displaying the `package_slug` got from cli args
/// or launch the default pkg if no cli args are provided
pub async fn fastn_ui_cli(matches: &clap::ArgMatches) -> fastn_core::Result<()> {
    // Act as an app if no arg is supplied
    if std::env::args().count() == 1 {
        tauri::run(tauri::FastnPackage::Default);
        return Ok(());
    }

    let Some(slug) = matches.get_one::<String>("package_slug") else {
        // We return with Ok so that other subcmds can run if `package_slug` is not provided
        return Ok(());
    };

    tauri::run(tauri::FastnPackage::Custom(slug.as_str()));

    Ok(())
}

pub async fn fastn_core_commands(matches: &clap::ArgMatches) -> fastn_core::Result<()> {
    if matches.subcommand_name().is_none() {
        return Ok(());
    }

    #[cfg(feature = "fifthtry")]
    if matches.subcommand_matches("upload").is_some() {
        clift::upload(matches).await;
        return Ok(());
    }

    let pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>> =
        actix_web::web::Data::new(scc::HashMap::new());

    let current_dir: camino::Utf8PathBuf = std::env::current_dir()?.canonicalize()?.try_into()?;
    let ds = fastn_ds::DocumentStore::new(current_dir, pg_pools);

    if let Some(update) = matches.subcommand_matches("update") {
        let check = update.get_flag("check");
        return fastn_update::update(&ds, check).await;
    }

    if let Some(serve) = matches.subcommand_matches("serve") {
        let port = serve.value_of_("port").map(|p| match p.parse::<u16>() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Provided port {p} is not a valid port.");
                std::process::exit(1);
            }
        });

        let bind = serve.value_of_("bind").unwrap_or("127.0.0.1").to_string();
        let edition = serve.value_of_("edition");
        let external_js = serve.values_of_("external-js");
        let inline_js = serve.values_of_("js");
        let external_css = serve.values_of_("external-css");
        let inline_css = serve.values_of_("css");
        let offline = serve.get_flag("offline");

        if cfg!(feature = "use-config-json") && !offline {
            fastn_update::update(&ds, false).await?;
        }

        let config = fastn_core::Config::read(ds, false, &None)
            .await?
            .add_edition(edition.map(ToString::to_string))?
            .add_external_js(external_js.clone())
            .add_inline_js(inline_js.clone())
            .add_external_css(external_css.clone())
            .add_inline_css(inline_css.clone());

        return fastn_core::listen(std::sync::Arc::new(config), bind.as_str(), port).await;
    }

    if let Some(test) = matches.subcommand_matches("test") {
        let edition = test.value_of_("edition").map(ToString::to_string);
        let external_js = test.values_of_("external-js");
        let inline_js = test.values_of_("js");
        let external_css = test.values_of_("external-css");
        let inline_css = test.values_of_("css");
        let offline: bool = test.get_flag("offline");

        if !offline {
            fastn_update::update(&ds, false).await?;
        }

        let mut config = fastn_core::Config::read(ds, true, &None).await?;

        config = config
            .add_edition(edition)?
            .add_external_js(external_js)
            .add_inline_js(inline_js)
            .add_external_css(external_css)
            .add_inline_css(inline_css)
            .set_test_command_running();

        return fastn_core::test(
            &config,
            test.value_of_("file"), // TODO: handle more than one files
            test.value_of_("base").unwrap_or("/"),
            test.get_flag("headless"),
            test.get_flag("script"),
            test.get_flag("verbose"),
        )
        .await;
    }

    if let Some(build) = matches.subcommand_matches("build") {
        if matches.get_flag("verbose") {
            println!("{}", fastn_core::debug_env_vars());
        }

        let edition = build.value_of_("edition").map(ToString::to_string);
        let external_js = build.values_of_("external-js");
        let inline_js = build.values_of_("js");
        let external_css = build.values_of_("external-css");
        let inline_css = build.values_of_("css");
        let zip_url = build.value_of_("zip-url");
        let offline: bool = build.get_flag("offline");

        if !offline {
            fastn_update::update(&ds, false).await?;
        }

        let mut config = fastn_core::Config::read(ds, true, &None).await?;

        config = config
            .add_edition(edition)?
            .add_external_js(external_js)
            .add_inline_js(inline_js)
            .add_external_css(external_css)
            .add_inline_css(inline_css);

        return fastn_core::build(
            &config,
            build.value_of_("file"), // TODO: handle more than one files
            build.value_of_("base").unwrap_or("/"),
            build.get_flag("ignore-failed"),
            matches.get_flag("test"),
            build.get_flag("check-build"),
            zip_url,
            &None,
        )
        .await;
    }

    let config = fastn_core::Config::read(ds, true, &None).await?;

    if let Some(fmt) = matches.subcommand_matches("fmt") {
        return fastn_core::fmt(
            &config,
            fmt.value_of_("file"),
            fmt.get_flag("noindentation"),
        )
        .await;
    }

    if let Some(wasmc) = matches.subcommand_matches("wasmc") {
        if let Err(e) = fastn_ds::wasmc(wasmc.value_of_("file").unwrap()).await {
            eprintln!("failed to compile: {e:?}");
            std::process::exit(1);
        }
    }

    if let Some(query) = matches.subcommand_matches("query") {
        return fastn_core::query(
            &config,
            query.value_of_("stage").unwrap(),
            query.value_of_("path"),
            query.get_flag("null"),
        )
        .await;
    }

    if matches.subcommand_matches("check").is_some() {
        return fastn_core::post_build_check(&config).await;
    }

    Ok(())
}

pub async fn check_for_update_cmd(matches: &clap::ArgMatches) -> fastn_core::Result<()> {
    let env_var_set = {
        if let Ok(val) = std::env::var("FASTN_CHECK_FOR_UPDATES") {
            val != "false"
        } else {
            false
        }
    };

    let flag = matches.get_flag("check-for-updates");

    // if the env var is set or the -c flag is passed, then check for updates
    if flag || env_var_set {
        check_for_update(flag).await?;
    }

    Ok(())
}

async fn check_for_update(report: bool) -> fastn_core::Result<()> {
    #[derive(serde::Deserialize, Debug)]
    struct GithubRelease {
        tag_name: String,
    }

    let url = "https://api.github.com/repos/fastn-stack/fastn/releases/latest";
    let release: GithubRelease = reqwest::Client::new()
        .get(url)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .header(reqwest::header::USER_AGENT, "fastn")
        .send()
        .await?
        .json()
        .await?;

    let current_version = version();

    if release.tag_name != current_version {
        println!(
            "You are using fastn {current_version}, and latest release is {}, visit https://fastn.com/install/ to learn how to upgrade.",
            release.tag_name
        );
    } else if report {
        // log only when -c is passed
        println!("You are using the latest release of fastn.");
    }

    Ok(())
}

pub fn set_env_vars(is_test_running: bool) {
    let checked_in = {
        if let Ok(status) = std::process::Command::new("git")
            .arg("ls-files")
            .arg("--error-unmatch")
            .arg(".env")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
        {
            status.success() // .env is checked in
        } else {
            false
        }
    };

    let ignore = {
        if let Ok(val) = std::env::var("FASTN_DANGER_ACCEPT_CHECKED_IN_ENV") {
            val != "false"
        } else {
            false
        }
    };

    if checked_in && !ignore {
        eprintln!(
            "ERROR: the .env file is checked in to version control! This is a security risk.
Remove it from your version control system or run fastn again with
FASTN_DANGER_ACCEPT_CHECKED_IN_ENV set"
        );
        std::process::exit(1);
    } else {
        if checked_in && ignore {
            println!(
                "WARN: your .env file has been detected in the version control system! This poses a
significant security risk in case the source code becomes public."
            );
        }

        if dotenvy::dotenv().is_ok() && !is_test_running {
            println!("INFO: loaded environment variables from .env file.");
        }
    }
}

/// Get the fastn version
pub fn version() -> &'static str {
    if std::env::args().any(|e| e == "--test") {
        env!("CARGO_PKG_VERSION")
    } else {
        match option_env!("GITHUB_SHA") {
            Some(sha) => {
                Box::leak(format!("{} [{}]", env!("CARGO_PKG_VERSION"), sha).into_boxed_str())
            }
            None => env!("CARGO_PKG_VERSION"),
        }
    }
}

/// fastn command line interface
pub fn cmd() -> clap::Command {
    let version = version();
    clap::Command::new("fastn: Full-stack Web Development Made Easy")
        .version(version)
        .arg_required_else_help(true)
        .subcommand_negates_reqs(true)
        .arg(clap::arg!(-c --"check-for-updates" "Check for updates"))
        .arg(clap::arg!(verbose: -v "Sets the level of verbosity"))
        .arg(clap::arg!(--test "Runs the command in test mode").hide(true))
        .arg(clap::arg!(--trace "Activate tracing").hide(true))
        .arg(
            clap::Arg::new("package_slug")
                .help("Slug of the package to launch in a native window. The slug should be downloadable from fifthtry.com")
                .required(true)
                .index(1),
        )
        .group(
            clap::ArgGroup::new("ui_mode")
                .args(["package_slug"])
                .required(true)
                .multiple(false),
        )
        .subcommand(
            clap::Command::new("build")
                .about("Build static site from this fastn package")
                .arg(clap::arg!(file: [FILE]... "The file to build (if specified only these are built, else entire package is built)"))
                .arg(clap::arg!(-b --base [BASE] "The base path.").default_value("/"))
                .arg(clap::arg!(--"zip-url" <URL> "The zip archive url for this package"))
                .arg(clap::arg!(--"ignore-failed" "Ignore failed files."))
                .arg(clap::arg!(--"check-build" "Checks .build for index files validation."))
                .arg(clap::arg!(--"external-js" <URL> "Script added in ftd files")
                    .action(clap::ArgAction::Append))
                .arg(clap::arg!(--js <URL> "Script text added in ftd files")
                    .action(clap::ArgAction::Append))
                .arg(clap::arg!(--"external-css" <URL> "CSS added in ftd files")
                    .action(clap::ArgAction::Append))
                .arg(clap::arg!(--css <URL> "CSS text added in ftd files")
                    .action(clap::ArgAction::Append))
                .arg(clap::arg!(--edition <EDITION> "The FTD edition"))
                .arg(clap::arg!(--offline "Disables automatic package update checks to operate in offline mode"))
        )
        .subcommand(
            clap::Command::new("fmt")
                .about("Format the fastn package")
                .arg(clap::arg!(file: [FILE]... "The file to format").required(false))
                .arg(clap::arg!(-i --noindentation "No indentation added to file/package").required(false))
        )
        .subcommand(
            clap::Command::new("wasmc")
                .about("Convert .wasm to .wasmc file")
                .arg(clap::arg!(file: [FILE]... "The file to compile").required(false))
        )
        .subcommand(
            clap::Command::new("test")
                .about("Run the test files in `_tests` folder")
                .arg(clap::arg!(file: [FILE]... "The file to build (if specified only these are built, else entire package is built)"))
                .arg(clap::arg!(-b --base [BASE] "The base path.").default_value("/"))
                .arg(clap::arg!(--"headless" "Run the test in headless mode"))
                .arg(clap::arg!(--"external-js" <URL> "Script added in ftd files")
                    .action(clap::ArgAction::Append))
                .arg(clap::arg!(--js <URL> "Script text added in ftd files")
                    .action(clap::ArgAction::Append))
                .arg(clap::arg!(--"external-css" <URL> "CSS added in ftd files")
                    .action(clap::ArgAction::Append))
                .arg(clap::arg!(--css <URL> "CSS text added in ftd files")
                    .action(clap::ArgAction::Append))
                .arg(clap::arg!(--edition <EDITION> "The FTD edition"))
                .arg(clap::arg!(--script "Generates a script file (for debugging purposes)"))
                .arg(clap::arg!(--verbose "To provide more better logs (for debugging purposes)"))
                .arg(clap::arg!(--offline "Disables automatic package update checks to operate in offline mode"))
        )
        .subcommand(
            clap::Command::new("query")
                .about("JSON Dump in various stages")
                .arg(clap::arg!(--stage <STAGE> "The stage. Currently supported (p1)").required
                (true))
                .arg(clap::arg!(-p --path [PATH] "The path of the file"))
                .arg(clap::arg!(-n --null "JSON with null and empty list"))
        )
        .subcommand(
            clap::Command::new("check")
                .about("Check if everything is fine with current fastn package")
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("update")
                .about("Update dependency packages for this fastn package")
                .arg(clap::arg!(--check "Check if packages are in sync with FASTN.ftd without performing updates."))
        )
        .subcommand(serve_cmd())
}

pub fn serve_cmd() -> clap::Command {
    let serve = clap::Command::new("serve")
        .about("Serve package content over HTTP")
        .after_help("fastn packages can have dynamic features. If your package uses any \
          dynamic feature, then you want to use `fastn serve` instead of `fastn build`.\n\n\
          Read more about it on https://fastn.com/")
        .arg(clap::arg!(--port <PORT> "The port to listen on [default: first available port starting 8000]"))
        .arg(clap::arg!(--bind <ADDRESS> "The address to bind to").default_value("127.0.0.1"))
        .arg(clap::arg!(--edition <EDITION> "The FTD edition"))
        .arg(clap::arg!(--"external-js" <URL> "Script added in ftd files")
            .action(clap::ArgAction::Append))
        .arg(clap::arg!(--js <URL> "Script text added in ftd files")
            .action(clap::ArgAction::Append))
        .arg(clap::arg!(--"external-css" <URL> "CSS added in ftd files")
            .action(clap::ArgAction::Append))
        .arg(clap::arg!(--css <URL> "CSS text added in ftd files")
            .action(clap::ArgAction::Append))
        .arg(clap::arg!(--"download-base-url" <URL> "If running without files locally, download needed files from here"))
        .arg(clap::arg!(--offline "Disables automatic package update checks to operate in offline mode"));

    serve
        .arg(
          clap::arg!(identities: --identities <IDENTITIES> "Http request identities, fastn allows these identities to access documents")
            .hide(true) // this is only for testing purpose
        )
}

pub trait ValueOf {
    fn value_of_(&self, name: &str) -> Option<&str>;
    fn values_of_(&self, name: &str) -> Vec<String>;
}

impl ValueOf for clap::ArgMatches {
    fn value_of_(&self, name: &str) -> Option<&str> {
        self.get_one::<String>(name).map(|v| v.as_str())
    }
    fn values_of_(&self, name: &str) -> Vec<String> {
        self.get_many(name)
            .map(|v| v.cloned().collect::<Vec<String>>())
            .unwrap_or_default()
    }
}
