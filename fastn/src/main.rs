pub fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(traced_main())
}

async fn traced_main() {
    use tracing_subscriber::layer::SubscriberExt;
    let level = std::env::var("TRACING")
        .unwrap_or_else(|_| "info".to_string())
        .parse::<tracing_forest::util::LevelFilter>()
        .unwrap_or(tracing_forest::util::LevelFilter::INFO);

    // only difference between the two branches of this if condition is the extra forest layer.
    if fastn_core::utils::is_traced() {
        tracing_forest::worker_task()
            .set_global(true)
            .build_with(|_layer: tracing_forest::ForestLayer<_, _>| {
                tracing_subscriber::Registry::default()
                    .with(tracing_forest::ForestLayer::default())
                    .with(level)
            })
            .on(outer_main())
            .await
    } else {
        tracing_forest::worker_task()
            .set_global(true)
            .build_with(|_layer: tracing_forest::ForestLayer<_, _>| {
                tracing_subscriber::Registry::default().with(level)
            })
            .on(outer_main())
            .await
    }
}

async fn outer_main() {
    if let Err(e) = async_main().await {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
}

async fn async_main() -> fastn_core::Result<()> {
    use colored::Colorize;
    use fastn_core::utils::ValueOf;

    let matches = app(version()).get_matches();

    match matches.subcommand() {
        Some((fastn_core::commands::stop_tracking::COMMAND, matches)) => {
            return fastn_core::commands::stop_tracking::handle_command(matches).await;
        }
        Some((fastn_core::commands::sync_status::COMMAND, matches)) => {
            return fastn_core::commands::sync_status::handle_command(matches).await;
        }
        _ => {}
    }

    if let Some(project) = matches.subcommand_matches("create-package") {
        // project-name => required field (any package Url or standard project name)
        let name = project.value_of_("name").unwrap();
        // project-path is optional
        let path = project.value_of_("path");
        let download_base_url = project.value_of_("download-base-url");
        return fastn_core::create_package(name, path, download_base_url).await;
    }

    if let Some(serve) = matches.subcommand_matches("serve") {
        let port = serve.value_of_("port").map(|p| match p.parse::<u16>() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Provided port {} is not a valid port.", p.to_string().red());
                std::process::exit(1);
            }
        });

        let bind = serve.value_of_("bind").unwrap_or("127.0.0.1").to_string();
        let download_base_url = serve.value_of_("download-base-url");
        let edition = serve.value_of_("edition");
        let external_js = serve.values_of_("external-js");
        let inline_js = serve.values_of_("js");
        let external_css = serve.values_of_("external-css");
        let inline_css = serve.values_of_("css");

        if serve.get_flag("cached-parse") {
            fastn_core::warning!("using cached parser, files modifications may be ignored");
            fastn_core::utils::enable_parse_caching(true);
        }

        return fastn_core::listen(
            bind.as_str(),
            port,
            download_base_url.map(ToString::to_string),
            edition.map(ToString::to_string),
            external_js,
            inline_js,
            external_css,
            inline_css,
        )
        .await;
    }

    if let Some(clone) = matches.subcommand_matches("clone") {
        return fastn_core::clone(clone.value_of_("source").unwrap()).await;
    }

    let mut config = fastn_core::Config::read(None, true, None).await?;

    if matches.subcommand_matches("update").is_some() {
        return fastn_core::update(&config).await;
    }

    if let Some(edit) = matches.subcommand_matches("edit") {
        return fastn_core::edit(
            &config,
            edit.value_of_("file").unwrap(),
            edit.value_of_("cr").unwrap(),
        )
        .await;
    }

    if let Some(add) = matches.subcommand_matches("add") {
        // TODO: support multiple files
        return fastn_core::add(&config, add.value_of_("file").unwrap(), add.value_of_("cr")).await;
    }

    if let Some(rm) = matches.subcommand_matches("rm") {
        return fastn_core::rm(&config, rm.value_of_("file").unwrap(), rm.value_of_("cr")).await;
    }

    if let Some(merge) = matches.subcommand_matches("merge") {
        return fastn_core::merge(
            &config,
            merge.value_of_("src"),
            merge.value_of_("dest").unwrap(),
            merge.value_of_("file"), // TODO: support multiple files
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

        config = config
            .add_edition(edition)?
            .add_external_js(external_js)
            .add_inline_js(inline_js)
            .add_external_css(external_css)
            .add_inline_css(inline_css);

        return fastn_core::build(
            &mut config,
            build.value_of_("file"), // TODO: handle more than one files
            build.value_of_("base").unwrap_or("/"),
            build.get_flag("ignore-failed"),
        )
        .await;
    }

    if let Some(mark_resolve) = matches.subcommand_matches("mark-resolved") {
        return fastn_core::mark_resolved(&config, mark_resolve.value_of_("path").unwrap()).await;
    }

    if let Some(abort_merge) = matches.subcommand_matches("abort-merge") {
        return fastn_core::abort_merge(&config, abort_merge.value_of_("path").unwrap()).await;
    }

    if let Some(revert) = matches.subcommand_matches("revert") {
        return fastn_core::revert(&config, revert.value_of_("path").unwrap()).await;
    }

    if let Some(sync) = matches.subcommand_matches("sync") {
        return if let Some(source) = sync.get_many::<String>("file") {
            let sources = source.map(|v| v.to_string()).collect();
            fastn_core::sync2(&config, Some(sources)).await
        } else {
            fastn_core::sync2(&config, None).await
        };
    }
    if let Some(create_cr) = matches.subcommand_matches("create-cr") {
        return fastn_core::create_cr(&config, create_cr.value_of_("title")).await;
    }
    if let Some(close_cr) = matches.subcommand_matches("close-cr") {
        return fastn_core::close_cr(&config, close_cr.value_of_("cr").unwrap()).await;
    }
    if let Some(status) = matches.subcommand_matches("status") {
        // TODO: handle multiple files
        return fastn_core::status(&config, status.value_of_("file")).await;
    }
    if matches.subcommand_matches("translation-status").is_some() {
        return fastn_core::translation_status(&config).await;
    }
    if let Some(diff) = matches.subcommand_matches("diff") {
        let all = diff.get_flag("all");
        return if let Some(source) = diff.get_many::<String>("file") {
            fastn_core::diff(&config, Some(source.map(|v| v.to_string()).collect()), all).await
        } else {
            fastn_core::diff(&config, None, all).await
        };
    }

    if let Some(resolve_conflict) = matches.subcommand_matches("resolve-conflict") {
        let use_ours = resolve_conflict.get_flag("use-ours");
        let use_theirs = resolve_conflict.get_flag("use-theirs");
        let print = resolve_conflict.get_flag("print");
        let revive_it = resolve_conflict.get_flag("revive-it");
        let delete_it = resolve_conflict.get_flag("delete-it");
        let source = resolve_conflict.value_of_("file").unwrap(); // TODO: handle multiple files
        return fastn_core::resolve_conflict(
            &config, source, use_ours, use_theirs, print, revive_it, delete_it,
        )
        .await;
    }
    if let Some(tracks) = matches.subcommand_matches("start-tracking") {
        let source = tracks.value_of_("source").unwrap();
        let target = tracks.value_of_("target").unwrap();
        return fastn_core::start_tracking(&config, source, target).await;
    }
    if let Some(mark) = matches.subcommand_matches("mark-upto-date") {
        let source = mark.value_of_("source").unwrap();
        let target = mark.value_of_("target");
        return fastn_core::mark_upto_date(&config, source, target).await;
    }

    unreachable!("No subcommand matched");
}

fn app(version: &'static str) -> clap::Command {
    clap::Command::new("fastn: FTD Package Manager")
        .version(version)
        .arg_required_else_help(true)
        .arg(clap::arg!(verbose: -v "Sets the level of verbosity"))
        .arg(clap::arg!(--test "Runs the command in test mode").hide(true))
        .arg(clap::arg!(--trace "Activate tracing").hide(true))
        .subcommand(
            // Initial subcommand format
            // fastn create-package <project-name> [project-path]
            //                   -n or --name   -p or --path
            // Necessary <project-name> with Optional [project-path]
            clap::Command::new("create-package")
                .about("Create a new fastn package")
                .arg(clap::arg!(name: <NAME> "The name of the package to create"))
                .arg(clap::arg!(-p --path [PATH] "Where to create the package (relative or absolute path, default value: the name)"))
                .arg(clap::arg!(--"download-base-url" <DOWNLOAD_BASE_URL> "base url of the package where it can downloaded"))
        )
        .subcommand(
            clap::Command::new("build")
                .about("Build static site from this fastn package")
                .arg(clap::arg!(file: [FILE]... "The file to build (if specified only these are built, else entire package is built)"))
                .arg(clap::arg!(-b --base [BASE] "The base path.").default_value("/"))
                .arg(clap::arg!(--"ignore-failed" "Ignore failed files."))
                .arg(clap::arg!(--"external-js" <URL> "Script added in ftd files")
                    .action(clap::ArgAction::Append))
                .arg(clap::arg!(--"js" <URL> "Script text added in ftd files")
                    .action(clap::ArgAction::Append))
                .arg(clap::arg!(--"external-css" <URL> "CSS added in ftd files")
                    .action(clap::ArgAction::Append))
                .arg(clap::arg!(--"css" <URL> "CSS text added in ftd files")
                    .action(clap::ArgAction::Append))
                .arg(clap::arg!(--edition <EDITION> "The FTD edition"))
        )
        .subcommand(
            clap::Command::new("mark-resolved")
                .about("Marks the conflicted file as resolved")
                .arg(clap::arg!(path: <PATH> "The path of the conflicted file"))
                .hide(true), // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("abort-merge")
                .about("Aborts the remote changes")
                .arg(clap::arg!(path: <PATH> "The path of the conflicted file"))
                .hide(true), // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("clone")
                .about("Clone a package into a new directory")
                .arg(clap::arg!(source: <SOURCE> "The source package to clone"))
                .hide(true)
        )
        .subcommand(
            clap::Command::new("edit")
                .about("Edit a file in CR workspace")
                .arg(clap::arg!(file: <FILE> "The file to edit"))
                .arg(clap::arg!(--cr <CR> "The CR to edit the file in").required(true))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("add")
                .about("Add one or more files in the workspace")
                .arg(clap::arg!(file: <FILE>... "The file(s) to add"))
                .arg(clap::arg!(--cr <CR> "The CR to add the file(s) in"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("rm")
                .about("Removes one or more files from the workspace")
                .arg(clap::arg!(file: <FILE>... "The file(s) to remove"))
                .arg(clap::arg!(--cr <CR> "The CR to remove the file(s) from"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("merge")
                .about("Merge two manifests together")
                .arg(clap::arg!(src: <SRC> "The source manifest to merge"))
                .arg(clap::arg!(dest: <DEST> "The destination manifest to merge"))
                .arg(clap::arg!(file: <FILE>... "The file(s) to merge"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("revert")
                .about("Reverts the local changes")
                .arg(clap::arg!(path: <PATH> "The path of the conflicted file"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("update")
                .about("Reinstall all the dependency packages")
        )
        .subcommand(
            clap::Command::new("sync")
                .about("Sync with fastn-repo (or .history folder if not using fastn-repo)")
                .arg(clap::arg!(file: <FILE>... "The file(s) to sync (leave empty to sync entire package)"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("status")
                .about("Show the status of files in this fastn package")
                .arg(clap::arg!(file: <FILE>... "The file(s) to see status of (leave empty to see status of entire package)").required(false))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(fastn_core::commands::sync_status::command())
        .subcommand(
            clap::Command::new("create-cr")
                .about("Create a Change Request")
                .arg(clap::arg!(title: <TITLE> "The title of the new CR"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("close-cr")
                .about("Create a Change Request")
                .arg(clap::arg!(cr: <CR> "The CR to Close"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("translation-status")
                .about("Show the translation status of files in this fastn package")
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("diff")
                .about("Show un-synced changes to files in this fastn package")
                .arg(clap::arg!(file: <FILE>... "The file(s) to see diff of (leave empty to see diff of entire package)").required(false))
                .arg(clap::arg!(-a --all "Show all changes."))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("resolve-conflict")
                .about("Show un-synced changes to files in this fastn package")
                .arg(clap::arg!(--"use-ours" "Use our version of the file"))
                .arg(clap::arg!(--"use-theirs" "Use their version of the file"))
                .arg(clap::arg!(--"revive-it" "Revive the file"))
                .arg(clap::arg!(--"delete-it" "Delete the file"))
                .arg(clap::arg!(--"print" "Print the file to stdout"))
                .arg(clap::arg!(file: <FILE> "The file to resolve the conflict for"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("check")
                .about("Check if everything is fine with current fastn package")
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("mark-upto-date")
                .about("Marks file as up to date.")
                .arg(clap::arg!(source: <SOURCE> "The source file to mark as up to date"))
                .arg(clap::arg!(--target <TARGET> "The target file to mark as up to date"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("start-tracking")
                .about("Add a tracking relation between two files")
                .arg(clap::arg!(source: <SOURCE> "The source file to start track"))
                .arg(clap::arg!(--target <TARGET> "The target file that will track the source").required(true))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(fastn_core::commands::stop_tracking::command())
        .subcommand(sub_command::serve())
        .subcommand(sub_command::publish_static())
}

mod sub_command {
    pub fn serve() -> clap::Command {
        let serve = clap::Command::new("serve")
            .about("Serve package content over HTTP")
            .after_help("fastn packages can have dynamic features. If your package uses any \
            dynamic feature, then you want to use `fastn serve` instead of `fastn build`.\n\n\
            Read more about it on https://fastn.io/serve/")
            .arg(clap::arg!(--port <PORT> "The port to listen on [default: first available port starting 8000]"))
            .arg(clap::arg!(--bind <ADDRESS> "The address to bind to").default_value("127.0.0.1"))
            .arg(clap::arg!(--"cached-parse" "Use cached parser"))
            .arg(clap::arg!(--edition <EDITION> "The FTD edition"))
            .arg(clap::arg!(--"external-js" <URL> "Script added in ftd files")
                .action(clap::ArgAction::Append))
            .arg(clap::arg!(--"js" <URL> "Script text added in ftd files")
                .action(clap::ArgAction::Append))
            .arg(clap::arg!(--"external-css" <URL> "CSS added in ftd files")
                .action(clap::ArgAction::Append))
            .arg(clap::arg!(--"css" <URL> "CSS text added in ftd files")
                .action(clap::ArgAction::Append))
            .arg(clap::arg!(--"download-base-url" <URL> "If running without files locally, download needed files from here"));
        if cfg!(feature = "remote") {
            serve
        } else {
            serve
                .arg(
                    clap::arg!(identities: --identities <IDENTITIES> "Http request identities, fastn allows these identities to access documents")
                        .hide(true) // this is only for testing purpose
                )
        }
    }
    pub fn publish_static() -> clap::Command {
        let publish_static = clap::Command::new("publish-static")
            .about("publish fastn packages to fastn-cloud")
            .arg(clap::arg!(--port <PORT> "The port to listen on [default: first available port starting 8000]"));
        publish_static
    }

}

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
