#[tokio::main]
async fn main() {
    let matches = app().get_matches();

    if matches.subcommand_matches("build").is_some() {
        fpm::build().await.expect("build failed");
    }
    if matches.subcommand_matches("sync").is_some() {
        fpm::sync().await.expect("sync failed");
    }
    if matches.subcommand_matches("status").is_some() {
        fpm::status().await.expect("status failed");
    }
    if matches.subcommand_matches("diff").is_some() {
        fpm::diff().await.expect("diff failed");
    }
    if let Some(tracks) = matches.subcommand_matches("tracks") {
        let tracks: Vec<&str> = tracks.values_of("tracks").unwrap().collect();
        fpm::tracks(tracks.first().unwrap(), tracks.last().unwrap())
            .await
            .expect("tracks failed");
    }
    if let Some(mark) = matches.subcommand_matches("mark") {
        let mark: Vec<&str> = mark.values_of("mark").unwrap().collect();
        fpm::mark(mark.first().unwrap(), mark.last().unwrap())
            .await
            .expect("mark failed");
    }
}

fn app() -> clap::App<'static, 'static> {
    clap::App::new("FTD Package Manager")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Description...")
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
            clap::SubCommand::with_name("build")
                .about("Build static site from this fpm package.")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("sync")
                .about("`sync` with `fpm-repo` or `.history` folder if not using `fpm-repo`")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("status")
                .about("Show the status of files in this fpm package.")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("diff")
                .about("Show un-synced changes to files in this fpm package.")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("check")
                .about("Check if everything is fine with current fpm package.")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("mark")
                .arg(
                    clap::Arg::with_name("mark")
                        .number_of_values(2)
                        .required(true),
                )
                .about("Marks file as up to date.")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("tracks")
                .arg(
                    clap::Arg::with_name("tracks")
                        .number_of_values(2)
                        .required(true),
                )
                .about("Add a tracking relation between two files.")
                .version(env!("CARGO_PKG_VERSION")),
        )
}
