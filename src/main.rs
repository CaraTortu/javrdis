use std::net::SocketAddr;

use tokio::{
    self,
    io::AsyncWriteExt,
    io::{AsyncReadExt, Result},
    net::{TcpListener, TcpStream},
};

mod command;
use self::command::Command;

async fn handle(mut stream: TcpStream, address: SocketAddr) -> Result<()> {
    // Print connected message
    println!("[+] Address {} connected!", address);

    let mut buf: [u8; 1024] = [0; 1024];

    // Listen for incoming messages
    loop {
        // Await until it can be readable
        stream.readable().await?;

        // Get message
        let bytes_read = stream.read(&mut buf).await?;
        if bytes_read == 0 {
            break;
        }

        // Translate from bytes to string
        let message = String::from_utf8_lossy(&buf[0..bytes_read]).to_string();

        // Parse commands
        for command in Command::parse_commands(&message).await {
            println!("[i] Got command '{:?}'", command);

            match command {
                Command::Ping => {
                    stream.write(b"+PONG\r\n").await?;
                }
                Command::Unknown => {
                    stream
                        .write(format!("-ERR unknown command '{message}'\r\n").as_bytes())
                        .await?;
                }
            }
        }
    }

    // Goodbye
    stream.shutdown().await?;
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
