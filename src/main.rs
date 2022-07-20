#[tokio::main]
async fn main() -> fpm::Result<()> {
    let matches = app(authors(), version()).get_matches();

    // Block of code to run when start-project subcommand is used
    if let Some(project) = matches.subcommand_matches("start-project") {
        // project-name => required field (any package Url or standard project name)
        let name = project.value_of("package-name").unwrap();
        // project-path is optional
        let path = project.value_of("package-path");
        fpm::start_project(name, path).await?;
        return Ok(());
    }

    // Serve block moved up
    if let Some(mark) = matches.subcommand_matches("serve") {
        let port = mark
            .value_of("port")
            .map_or(mark.value_of("positional_port"), Some)
            .map(|p| {
                p.parse::<u16>()
                    .unwrap_or_else(|_| panic!("provided port {} is wrong", p))
            });

        let bind = mark.value_of("bind").unwrap_or("127.0.0.1").to_string();
        tokio::task::spawn_blocking(move || {
            fpm::serve2(bind.as_str(), port).expect("http service error");
        })
        .await
        .expect("Thread spawn error");
        return Ok(());
    }

    if let Some(clone) = matches.subcommand_matches("clone") {
        fpm::clone(clone.value_of("source").unwrap()).await?;
        return Ok(());
    }

    let mut config = fpm::Config::read2(None, true).await?;

    if matches.subcommand_matches("update").is_some() {
        fpm::update(&config).await?;
    }

    if let Some(add) = matches.subcommand_matches("add") {
        fpm::add(&config, add.value_of("file").unwrap()).await?;
        return Ok(());
    }

    if let Some(rm) = matches.subcommand_matches("rm") {
        fpm::rm(&config, rm.value_of("file").unwrap()).await?;
        return Ok(());
    }

    if let Some(build) = matches.subcommand_matches("build") {
        // Evaluate the aliases for the package
        config.package.aliases();
        if build.is_present("verbose") {
            println!("{}", fpm::debug_env_vars());
        }

        fpm::build2(
            &mut config,
            build.value_of("file"),
            build.value_of("base").unwrap(), // unwrap okay because base is required
            build.is_present("ignore-failed"),
        )
        .await?;
    }

    if let Some(mark_resolve) = matches.subcommand_matches("mark-resolve") {
        fpm::mark_resolve(&config, mark_resolve.value_of("path").unwrap()).await?;
    }

    if let Some(abort_merge) = matches.subcommand_matches("abort-merge") {
        fpm::abort_merge(&config, abort_merge.value_of("path").unwrap()).await?;
    }

    if let Some(revert) = matches.subcommand_matches("revert") {
        fpm::revert(&config, revert.value_of("path").unwrap()).await?;
    }

    if let Some(sync) = matches.subcommand_matches("sync") {
        if let Some(source) = sync.values_of("source") {
            let sources = source.map(|v| v.to_string()).collect();
            fpm::sync2(&config, Some(sources)).await?;
        } else {
            fpm::sync2(&config, None).await?;
        }
    }
    if let Some(status) = matches.subcommand_matches("status") {
        let source = status.value_of("source");
        fpm::status(&config, source).await?;
    }
    if matches.subcommand_matches("translation-status").is_some() {
        fpm::translation_status(&config).await?;
    }
    if let Some(diff) = matches.subcommand_matches("diff") {
        let all = diff.is_present("all");
        if let Some(source) = diff.values_of("source") {
            let sources = source.map(|v| v.to_string()).collect();
            fpm::diff(&config, Some(sources), all).await?;
        } else {
            fpm::diff(&config, None, all).await?;
        }
    }
    if let Some(tracks) = matches.subcommand_matches("start-tracking") {
        let source = tracks.value_of("source").unwrap();
        let target = tracks.value_of("target").unwrap();
        fpm::start_tracking(&config, source, target).await?;
    }
    if let Some(mark) = matches.subcommand_matches("mark-upto-date") {
        let source = mark.value_of("source").unwrap();
        let target = mark.value_of("target");
        fpm::mark_upto_date(&config, source, target).await?;
    }
    if let Some(mark) = matches.subcommand_matches("stop-tracking") {
        let source = mark.value_of("source").unwrap();
        let target = mark.value_of("target");
        fpm::stop_tracking(&config, source, target).await?;
    }
    Ok(())
}

fn app(authors: &'static str, version: &'static str) -> clap::App<'static, 'static> {
    clap::App::new("fpm: FTD Package Manager")
        .version(version)
        .author(authors)
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .arg(
            clap::Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            clap::Arg::with_name("test")
                .long("--test")
                .help("Runs the command in test mode")
                .hidden(true),
        )
        .subcommand(
            // Initial subcommand format
            // fpm start-project <project-name> [project-path]
            //                   -n or --name   -p or --path
            // Necessary <project-name> with Optional [project-path]
            clap::SubCommand::with_name("start-project")
                .about("Creates a template ftd project at the target location with the given project name")
                .arg(
                    clap::Arg::with_name("package-name")
                        .required(true)
                        .help("Package name")
                )
                .arg(
                    clap::Arg::with_name("package-path")
                        .short("p")
                        .long("path")
                        .takes_value(true)
                        .help("Package path (relative)")
                )
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("build")
                .about("Build static site from this fpm package")
                .arg(clap::Arg::with_name("file").required(false))
                .arg(
                    clap::Arg::with_name("base")
                        .long("base")
                        .takes_value(true)
                        .default_value("/")
                        .help("Base URL"),
                )
                .arg(
                    clap::Arg::with_name("ignore-failed")
                        .long("ignore-failed")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    clap::Arg::with_name("verbose")
                        .long("verbose")
                        .short("v")
                        .takes_value(false)
                        .required(false),
                )
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
        clap::SubCommand::with_name("mark-resolve")
            .about("Marks the conflicted file as resolved")
            .arg(clap::Arg::with_name("path").required(true))
            .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("abort-merge")
                .about("Aborts the remote changes")
                .arg(clap::Arg::with_name("path").required(true))
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("clone")
                .about("Clone a package into a new directory")
                .arg(clap::Arg::with_name("source").required(true))
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("add")
                .about("Adds a file in workspace")
                .arg(clap::Arg::with_name("file").required(true))
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("rm")
                .about("Removes a file in workspace")
                .arg(clap::Arg::with_name("file").required(true))
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("revert")
                .about("Reverts the local changes")
                .arg(clap::Arg::with_name("path").required(true))
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("update")
                .about("Reinstall all the dependency packages")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("sync")
                .arg(clap::Arg::with_name("source").multiple(true))
                .about("Sync with fpm-repo or .history folder if not using fpm-repo")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("status")
                .arg(clap::Arg::with_name("source"))
                .about("Show the status of files in this fpm package")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("translation-status")
                .about("Show the translation status of files in this fpm package")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("diff")
                .args(&[
                    clap::Arg::with_name("source").multiple(true),
                    clap::Arg::with_name("all").long("--all").short("a"),
                ])
                .about("Show un-synced changes to files in this fpm package")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("check")
                .about("Check if everything is fine with current fpm package")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("mark-upto-date")
                .args(&[
                    clap::Arg::with_name("source").required(true),
                    clap::Arg::with_name("target")
                        .long("--target")
                        .takes_value(true),
                ])
                .about("Marks file as up to date.")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("start-tracking")
                .args(&[
                    clap::Arg::with_name("source").required(true),
                    clap::Arg::with_name("target")
                        .long("--target")
                        .takes_value(true)
                        .required(true),
                ])
                .about("Add a tracking relation between two files")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("stop-tracking")
                .args(&[
                    clap::Arg::with_name("source").required(true),
                    clap::Arg::with_name("target")
                        .long("--target")
                        .takes_value(true),
                ])
                .about("Remove a tracking relation between two files")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("serve")
                .arg(clap::Arg::with_name("port")
                        .required_unless("positional_port")
                        .required(false)
                        .help("Specify the port to serve on")
                )
                .arg(clap::Arg::with_name("positional_port")
                        .long("--port")
                        .required_unless("bind")
                        .takes_value(true)
                        .required(false)
                        .help("Specify the port to serve on")
                )
                .arg(clap::Arg::with_name("bind")
                        .long("--bind")
                        .takes_value(true)
                        .required(false)
                        .help("Specify the bind address to serve on")
                )
                .about("Create an http server and serves static files")
                .version(env!("CARGO_PKG_VERSION")),
        )
}

pub fn version() -> &'static str {
    if std::env::args().any(|e| e == "--test") {
        env!("CARGO_PKG_VERSION")
    } else {
        Box::leak(
            format!("{} [{}]", env!("CARGO_PKG_VERSION"), env!("VERGEN_GIT_SHA")).into_boxed_str(),
        )
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
