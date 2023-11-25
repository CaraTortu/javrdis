use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Error, ErrorKind, Result},
    net::TcpStream,
};

pub struct Client {
    stream: TcpStream,
    buffer: [u8; 1024],
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: [0; 1024],
        }
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        self.stream.shutdown().await
    }

    #[allow(unused)]
    pub async fn read_into_buffer(&mut self, buffer: &mut [u8; 1024]) -> Result<usize> {
        self.stream.readable().await?;
        self.stream.read(buffer).await
    }

    pub async fn read_into_string(&mut self) -> Result<String> {
        let bytes_read = self.stream.read(&mut self.buffer).await?;

        match bytes_read {
            0 => Err(Error::new(
                ErrorKind::TimedOut,
                "Host connection was closed",
            )),
            n => Ok(String::from_utf8_lossy(&self.buffer[0..n - 1]).to_string()),
        }
    }

    async fn send(&mut self, mut msg: String) -> Result<usize> {
        msg.push_str("\r\n");
        self.stream.write(msg.as_bytes()).await
    }

    pub async fn send_simple_string(&mut self, string: &str) -> Result<usize> {
        self.stream.writable().await?;
        self.send(format!("+{string}")).await
    }

    pub async fn send_simple_error(&mut self, string: &str) -> Result<usize> {
        self.stream.writable().await?;
        self.send(format!("-ERR {string}")).await
    }
}
