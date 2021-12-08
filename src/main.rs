#[tokio::main]
async fn main() {
    let matches = clap::App::new("FTD Package Manager")
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
                .about("Builds the current directory")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("sync")
                .about("`sync` with `fpm-repo` or `.history` folder if not using `fpm-repo`")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("status")
                .about("Shows how many files have changed, comparing it with latest version of that file in `.history` folder")
                .version(env!("CARGO_PKG_VERSION")),
        ).subcommand(
            clap::SubCommand::with_name("diff")
                .about("Show changes in files, comparing it with latest version of that file in `.history` folder")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("check")
                .about("Checks the folder structure of the current .FPM.ftd file")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("mark").arg(
                clap::Arg::with_name("mark")
                    .number_of_values(2)
                    .required(true)
                )
                .about("Marks the file up to date")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .subcommand(
            clap::SubCommand::with_name("tracks")
                .arg(
                    clap::Arg::with_name("tracks")
                        .number_of_values(2)
                        .required(true)
                )
                .about("Tracks the document")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .get_matches();

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
            .expect("tracks failed");
    }
}
