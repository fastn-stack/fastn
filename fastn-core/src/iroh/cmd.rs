pub fn attach(cmd: clap::Command) -> clap::Command {
    cmd.subcommand(
        clap::Command::new("proxy")
            .about("Proxy to a fastn net fastn-http service")
            .arg(clap::arg!(<id> "The id of fastn-net http service.").required(true))
            .arg(clap::arg!(--port <PORT> "The port to listen on [default: first available port starting 8000]"))
    )
}

pub struct Proxy {
    pub(crate) port: u16,
    #[expect(unused)]
    pub(crate) remote_id: String,
    pub(crate) protocol: String,
}

pub fn parse(matches: &clap::ArgMatches) -> fastn_core::Result<Proxy> {
    let matches = matches
        .subcommand_matches("proxy")
        .expect("this function is only called after this check in main");

    let remote_id = matches.get_one::<String>("id").unwrap().to_string();
    let port = match matches.get_one::<String>("port") {
        Some(port) => port.parse::<u16>()?,
        None => 8000,
    };

    Ok(Proxy {
        remote_id,
        port,
        protocol: "http".to_string(),
    })
}
