use std::net::SocketAddr;

use tokio::{
    self,
    io::Result,
    net::{TcpListener, TcpStream},
};

mod client;
mod command;
use crate::client::Client;
use crate::command::Command;

async fn handle(stream: TcpStream, address: SocketAddr) -> Result<()> {
    // Print connected message
    println!("[+] Address {} connected!", address);

    let mut client: Client = Client::new(stream);

    loop {
        let message = match client.read_into_string().await {
            Err(_) => break,
            Ok(c) => c.strip_suffix("\r").unwrap_or(&c).to_owned(),
        };

        // Parse commands
        for command in Command::parse_commands(&message).await {
            println!("[i] Got command '{:?}'", command);

            match command {
                Command::Ping => {
                    client.send_simple_string("PONG").await?;
                }
                Command::Unknown => {
                    client
                        .send_simple_error(&format!("unknown command '{message}'"))
                        .await?;
                }
            }
        }
    }

    // Goodbye
    client.shutdown().await?;
    println!("[+] Connection with {} ended", address);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        if let Ok((stream, address)) = listener.accept().await {
            tokio::spawn(async move { handle(stream, address).await });
        };
    }
}
