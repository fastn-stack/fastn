fn main() -> fpm::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}

async fn async_main() -> fpm::Result<()> {
    use colored::Colorize;

    let matches = app(authors(), version()).get_matches();

    if let Some(project) = matches.subcommand_matches("start-project") {
        // project-name => required field (any package Url or standard project name)
        let name = project.value_of_("package-name").unwrap();
        // project-path is optional
        let path = project.value_of_("package-path");
        fpm::start_project(name, path).await?;
        return Ok(());
    }

    if let Some(mark) = matches.subcommand_matches("serve") {
        let port = mark.value_of_("port").map(|p| match p.parse::<u16>() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Provided port {} is not a valid port.", p.to_string().red());
                std::process::exit(1);
            }
        });

        let bind = mark.value_of_("bind").unwrap_or("127.0.0.1").to_string();
        let download_base_url = mark.value_of_("download-base-url");

        fpm::listen(
            bind.as_str(),
            port,
            download_base_url.map(ToString::to_string),
        )
        .await?;
        return Ok(());
    }

    if let Some(clone) = matches.subcommand_matches("clone") {
        fpm::clone(clone.value_of_("source").unwrap()).await?;
        return Ok(());
    }

    let mut config = fpm::Config::read(None, true, None).await?;

    if matches.subcommand_matches("update").is_some() {
        fpm::update(&config).await?;
    }

    if let Some(edit) = matches.subcommand_matches("edit") {
        fpm::edit(
            &config,
            edit.value_of_("file").unwrap(),
            edit.value_of_("cr").unwrap(),
        )
        .await?;
        return Ok(());
    }

    if let Some(add) = matches.subcommand_matches("add") {
        fpm::add(&config, add.value_of_("file").unwrap(), add.value_of_("cr")).await?;
        return Ok(());
    }

    if let Some(rm) = matches.subcommand_matches("rm") {
        fpm::rm(&config, rm.value_of_("file").unwrap(), rm.value_of_("cr")).await?;
        return Ok(());
    }

    if let Some(merge) = matches.subcommand_matches("merge") {
        fpm::merge(
            &config,
            merge.value_of_("src"),
            merge.value_of_("dest").unwrap(),
            merge.value_of_("file"),
        )
        .await?;
        return Ok(());
    }

    if let Some(build) = matches.subcommand_matches("build") {
        if build.contains_id("verbose") {
            println!("{}", fpm::debug_env_vars());
        }

        fpm::build(
            &mut config,
            build.value_of_("file"),
            build.value_of_("base").unwrap(), // unwrap okay because base is required
            build.contains_id("ignore-failed"),
        )
        .await?;
    }

    if let Some(mark_resolve) = matches.subcommand_matches("mark-resolve") {
        fpm::mark_resolve(&config, mark_resolve.value_of_("path").unwrap()).await?;
    }

    if let Some(abort_merge) = matches.subcommand_matches("abort-merge") {
        fpm::abort_merge(&config, abort_merge.value_of_("path").unwrap()).await?;
    }

    if let Some(revert) = matches.subcommand_matches("revert") {
        fpm::revert(&config, revert.value_of_("path").unwrap()).await?;
    }

    if let Some(sync) = matches.subcommand_matches("sync") {
        if let Some(source) = sync.get_many::<String>("source") {
            let sources = source.map(|v| v.to_string()).collect();
            fpm::sync2(&config, Some(sources)).await?;
        } else {
            fpm::sync2(&config, None).await?;
        }
    }
    if let Some(status) = matches.subcommand_matches("sync-status") {
        let source = status.value_of_("source");
        fpm::sync_status(&config, source).await?;
    }
    if let Some(create_cr) = matches.subcommand_matches("create-cr") {
        let title = create_cr.value_of_("title");
        fpm::create_cr(&config, title).await?;
    }
    if let Some(close_cr) = matches.subcommand_matches("close-cr") {
        let cr = close_cr.value_of_("cr").unwrap();
        fpm::close_cr(&config, cr).await?;
    }
    if let Some(status) = matches.subcommand_matches("status") {
        let source = status.value_of_("source");
        fpm::status(&config, source).await?;
    }
    if matches.subcommand_matches("translation-status").is_some() {
        fpm::translation_status(&config).await?;
    }
    if let Some(diff) = matches.subcommand_matches("diff") {
        let all = diff.contains_id("all");
        if let Some(source) = diff.get_many::<String>("source") {
            let sources = source.map(|v| v.to_string()).collect();
            fpm::diff(&config, Some(sources), all).await?;
        } else {
            fpm::diff(&config, None, all).await?;
        }
    }
    if let Some(resolve_conflict) = matches.subcommand_matches("resolve-conflict") {
        let use_ours = resolve_conflict.contains_id("use-ours");
        let use_theirs = resolve_conflict.contains_id("use-theirs");
        let print = resolve_conflict.contains_id("print");
        let revive_it = resolve_conflict.contains_id("revive-it");
        let delete_it = resolve_conflict.contains_id("delete-it");
        let source = resolve_conflict.value_of_("source").unwrap();
        fpm::resolve_conflict(
            &config, source, use_ours, use_theirs, print, revive_it, delete_it,
        )
        .await?;
    }
    if let Some(tracks) = matches.subcommand_matches("start-tracking") {
        let source = tracks.value_of_("source").unwrap();
        let target = tracks.value_of_("target").unwrap();
        fpm::start_tracking(&config, source, target).await?;
    }
    if let Some(mark) = matches.subcommand_matches("mark-upto-date") {
        let source = mark.value_of_("source").unwrap();
        let target = mark.value_of_("target");
        fpm::mark_upto_date(&config, source, target).await?;
    }
    if let Some(mark) = matches.subcommand_matches("stop-tracking") {
        let source = mark.value_of_("source").unwrap();
        let target = mark.value_of_("target");
        fpm::stop_tracking(&config, source, target).await?;
    }
    Ok(())
}

fn app(authors: &'static str, version: &'static str) -> clap::Command {
    clap::Command::new("fpm: FTD Package Manager")
        .version(version)
        .author(authors)
        .arg_required_else_help(true)
        .arg(
            clap::Arg::new("verbose")
                .short('v')
                .help("Sets the level of verbosity"),
        )
        .arg(
            clap::Arg::new("test")
                .long("test")
                .help("Runs the command in test mode")
                .hide(true),
        )
        .subcommand(
            // Initial subcommand format
            // fpm start-project <project-name> [project-path]
            //                   -n or --name   -p or --path
            // Necessary <project-name> with Optional [project-path]
            clap::Command::new("start-project")
                .about("Creates a template ftd project at the target location with the given project name")
                .arg(
                    clap::Arg::new("package-name")
                        .required(true)
                        .help("Package name")
                )
                .arg(
                    clap::Arg::new("package-path")
                        .short('p')
                        .long("path")
                        .action(clap::ArgAction::Set)
                        .help("Package path (relative)")
                )
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("build")
                .about("Build static site from this fpm package")
                .arg(clap::Arg::new("file").required(false))
                .arg(
                    clap::Arg::new("base")
                        .long("base")
                        .action(clap::ArgAction::Set)
                        .default_value("/")
                        .help("Base URL"),
                )
                .arg(
                    clap::Arg::new("ignore-failed")
                        .long("ignore-failed")
                        .required(false),
                )
                .arg(
                    clap::Arg::new("verbose")
                        .long("verbose")
                        .short('v')
                        .required(false),
                )
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
        clap::Command::new("mark-resolve")
            .about("Marks the conflicted file as resolved")
            .arg(clap::Arg::new("path").required(true))
            .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("abort-merge")
                .about("Aborts the remote changes")
                .arg(clap::Arg::new("path").required(true))
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("clone")
                .about("Clone a package into a new directory")
                .arg(clap::Arg::new("source").required(true))
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("edit")
                .about("Edit a file in CR workspace")
                .args(&[
                    clap::Arg::new("file").required(true),
                    clap::Arg::new("cr").long("cr").action(clap::ArgAction::Set).required(true),
                ])
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("add")
                .about("Adds a file in workspace")
                .args(&[
                    clap::Arg::new("file").required(true),
                    clap::Arg::new("cr").long("cr").action(clap::ArgAction::Set),
                ])
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("rm")
                .about("Removes a file in workspace")
                .args(&[
                    clap::Arg::new("file").required(true),
                    clap::Arg::new("cr").long("cr").action(clap::ArgAction::Set),
                ])
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("merge")
                .about("Merge two manifests together")
                .args(&[
                    clap::Arg::new("src").long("src").action(clap::ArgAction::Set),
                    clap::Arg::new("dest").long("dest").action(clap::ArgAction::Set).required(true),
                    clap::Arg::new("file"),
                ])
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("revert")
                .about("Reverts the local changes")
                .arg(clap::Arg::new("path").required(true))
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("update")
                .about("Reinstall all the dependency packages")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("sync")
                .arg(clap::Arg::new("source").action(clap::ArgAction::Append))
                .about("Sync with fpm-repo or .history folder if not using fpm-repo")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("status")
                .arg(clap::Arg::new("source"))
                .about("Show the status of files in this fpm package")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("sync-status")
                .arg(clap::Arg::new("source"))
                .about("Show the sync status of files in this fpm package")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("create-cr")
                .arg(clap::Arg::new("title"))
                .about("Create a Change Request")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("close-cr")
                .arg(clap::Arg::new("cr").required(true))
                .about("Create a Change Request")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("translation-status")
                .about("Show the translation status of files in this fpm package")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("diff")
                .args(&[
                    clap::Arg::new("source").action(clap::ArgAction::Append),
                    clap::Arg::new("all").long("all").short('a'),
                ])
                .about("Show un-synced changes to files in this fpm package")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("resolve-conflict")
                .args(&[
                    clap::Arg::new("use-ours").long("use-ours"),
                    clap::Arg::new("use-theirs").long("use-theirs"),
                    clap::Arg::new("revive-it").long("revive-it"),
                    clap::Arg::new("delete-it").long("delete-it"),
                    clap::Arg::new("print").long("print"),
                    clap::Arg::new("source").required(true),
                ])
                .about("Show un-synced changes to files in this fpm package")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("check")
                .about("Check if everything is fine with current fpm package")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("mark-upto-date")
                .args(&[
                    clap::Arg::new("source").required(true),
                    clap::Arg::new("target")
                        .long("target")
                        .action(clap::ArgAction::Set),
                ])
                .about("Marks file as up to date.")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("start-tracking")
                .args(&[
                    clap::Arg::new("source").required(true),
                    clap::Arg::new("target")
                        .long("target")
                        .action(clap::ArgAction::Set)
                        .required(true),
                ])
                .about("Add a tracking relation between two files")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::Command::new("stop-tracking")
                .args(&[
                    clap::Arg::new("source").required(true),
                    clap::Arg::new("target")
                        .long("target")
                        .action(clap::ArgAction::Set),
                ])
                .about("Remove a tracking relation between two files")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(sub_command::serve())
}

mod sub_command {
    pub fn serve() -> clap::Command {
        let serve = clap::Command::new("serve")
            .arg(
                clap::Arg::new("port")
                    .long("port")
                    .action(clap::ArgAction::Set)
                    .help("Specify the port to serve on"),
            )
            .arg(
                clap::Arg::new("bind")
                    .long("bind")
                    .action(clap::ArgAction::Set)
                    .help("Specify the bind address to serve on"),
            )
            .arg(
                clap::Arg::new("download-base-url")
                    .long("download-base-url")
                    .action(clap::ArgAction::Set)
                    .help("URL of Package to download documents, where it is stored."),
            );

        if cfg!(feature = "remote") {
            serve
        } else {
            serve
                .arg(
                    clap::Arg::new("identities")
                        .long("identities")
                        .action(clap::ArgAction::Set)
                        .required(false)
                        .help(
                            "Http request identities, fpm allows these identities to access documents",
                        ),
                )
                .about("Create an http server and serves static files")
                .version(env!("CARGO_PKG_VERSION"))
        }
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

pub fn authors() -> &'static str {
    Box::leak(
        env!("CARGO_PKG_AUTHORS")
            .split(':')
            .map(|v| v.split_once('<').map(|(v, _)| v.trim()).unwrap_or_default())
            .collect::<Vec<_>>()
            .join(", ")
            .into_boxed_str(),
    )
}

trait ValueOf {
    fn value_of_(&self, name: &str) -> Option<&str>;
}

impl ValueOf for clap::ArgMatches {
    fn value_of_(&self, name: &str) -> Option<&str> {
        self.get_one::<String>(name).map(|v| v.as_str())
    }
}
