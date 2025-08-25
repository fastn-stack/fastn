use tokio::io::{AsyncWriteExt};
use tokio_util::codec::{Framed, LinesCodec};
use futures::{SinkExt, StreamExt};

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9090")
        .await
        .expect("Failed to bind to port 587");

    println!("Server listening on 127.0.0.1:9090");

    loop {
        let (mut stream, _) = listener
            .accept()
            .await
            .expect("Failed to accept connection");

        tokio::spawn(async move {
            if let Err(e) = handle_connection(&mut stream).await {
                eprintln!("Error handling connection: {}", e);
            }

            if let Err(e) = handle_session(stream).await {
                eprintln!("Error handling session: {}", e);
            }
        });
    }
}

async fn handle_connection(stream: &mut tokio::net::TcpStream) -> Result<(), eyre::Error> {
    let (_, mut writer) = stream.split();

    writer
        .write_all(b"220 Welcome to the SMTP server\r\n")
        .await?;

    Ok(())
}

enum SmtpState {
    Command,
    Data,
    Quit,
}

async fn handle_session(stream: tokio::net::TcpStream) -> eyre::Result<()> {
    let RE_SMTP_MAIL = regex::Regex::new(r"(?i)from: ?<(.+)>").unwrap();
    let RE_SMTP_RCPT = regex::Regex::new(r"(?i)to: ?<(.+)>").unwrap();

    let mut message = String::new();
    let mut state = SmtpState::Command;

    let mut mailfrom: Option<String> = None;
    let mut rcpts: Vec<String> = Vec::new();

    let mut framed = Framed::new(stream, LinesCodec::new());

    while let Some(line_str) = framed.next().await {
        let line = line_str?;
        match state {
            SmtpState::Command => {
                let space_pos = line.find(" ").unwrap_or(line.len());
                let (command, arg) = line.split_at(space_pos);
                let arg = arg.trim();
                match &*command.trim().to_uppercase() {
                    "HELO" | "EHLO" => {
                        send_commands(&mut framed, vec!["250 Hello".to_string()]).await?;
                    }
                    "MAIL" => {
                        if let Some(address) = RE_SMTP_MAIL.captures(arg).and_then(|cap| cap.get(1))
                        {
                            mailfrom = Some(address.as_str().to_string());
                            send_commands(&mut framed, vec!["250 OK".to_string()]).await?;
                        } else {
                            send_commands(
                                &mut framed,
                                vec!["501 Syntax: MAIL From: <address>".to_string()],
                            )
                            .await?;
                        }
                    }
                    "RCPT" => {
                        if mailfrom.is_none() {
                            send_commands(
                                &mut framed,
                                vec!["503 Error: Send MAIL first".to_string()],
                            )
                            .await?;
                        } else {
                            if let Some(address) =
                                RE_SMTP_RCPT.captures(arg).and_then(|cap| cap.get(1))
                            {
                                rcpts.push(address.as_str().to_string());
                                send_commands(&mut framed, vec!["250 OK".to_string()]).await?;
                            } else {
                                send_commands(
                                    &mut framed,
                                    vec!["501 Syntax: RCPT TO: <address>".to_string()],
                                )
                                .await?;
                            }
                        }
                    }
                    "DATA" => {
                        if rcpts.is_empty() {
                            send_commands(&mut framed, vec!["503 Error: MAIL FROM and RCPT TO must be set before sending DATA".to_string()]).await?;
                        } else {
                            state = SmtpState::Data;
                            send_commands(
                                &mut framed,
                                vec!["354 End data with <CR><LF>.<CR><LF>".to_string()],
                            )
                            .await?;
                        }
                    }
                    "RSET" => {
                        mailfrom = None;
                        rcpts = Vec::new();
                        message = String::new();
                        send_commands(&mut framed, vec!["250 OK".to_string()]).await?;
                    }
                    "QUIT" => {
                        send_commands(&mut framed, vec!["221 Bye".to_string()]).await?;
                        state = SmtpState::Quit;
                    }
                    _ => {
                        send_commands(&mut framed, vec!["500 Unknown command".to_string()]).await?;
                    }
                }
            }
            SmtpState::Data => {
                if line.trim() == "." {
                    send_commands(&mut framed, vec!["250 OK".to_string()]).await?;

                    handle_email(mailfrom.clone(), rcpts.clone(), message.clone()).await;

                    // reset the state and variables for the next email
                    mailfrom = None;
                    rcpts = Vec::new();
                    message = String::new();
                    state = SmtpState::Command;
                } else {
                    message.push_str(&line);
                    message.push_str("\n");
                }
            }
            SmtpState::Quit => {
                break;
            }
        }
    }

    Ok(())
}

async fn handle_email(mailfrom: Option<String>, rcpts: Vec<String>, message: String) {
    println!("Received email from: {:?}", mailfrom);
    println!("Recipients: {:?}", rcpts);
    println!("Message:\n{}", message);
}

async fn send_commands(
    framed: &mut Framed<tokio::net::TcpStream, LinesCodec>,
    commands: Vec<String>,
) -> Result<(), eyre::Error> {
    // only need to add \r because the codec adds \n
    let messages = futures::stream::iter(commands.into_iter().map(|x| format!("{}\r", x)));
    framed.send_all(&mut messages.map(Ok)).await?;
    Ok(())
}
