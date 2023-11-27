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
        // Parse commands
        let command = match Command::parse_command(&mut client).await {
            Ok(v) => v,
            Err(e) => {
                println!("[i] Closing connection with {address} for '{e}'");
                client.send_simple_error(&format!("Closing your connection because {e}")).await;
                break;
            }
        };

        println!("[i] Got command '{:?}'", command);

        match command {
            Command::Ping => client.send_simple_string("PONG").await,
            Command::Unknown => client.send_simple_error(&format!("Unknown command")).await,
            cmd => {
                println!("{cmd:?}");
            }
        }
    }

    // Goodbye
    client.shutdown().await;
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
