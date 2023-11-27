use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, Error, ErrorKind, Result},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

pub struct Client {
    writer: BufWriter<OwnedWriteHalf>,
    reader: BufReader<OwnedReadHalf>,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        let (reader, writer) = stream.into_split();

        Self {
            writer: BufWriter::new(writer),
            reader: BufReader::new(reader),
        }
    }

    pub async fn shutdown(self) {
        let hr = self.reader.into_inner();
        let hw = self.writer.into_inner();

        let shutdown_result = match hr.reunite(hw) {
            Err(_) => Ok(println!("[-] ERROR: could not reunite reader and writer.")),
            Ok(mut joined) => joined.shutdown().await,
        };

        match shutdown_result {
            Err(_) => println!("[-] ERROR: could not shut down TcpStream."),
            Ok(_) => (),
        };
    }

    pub async fn read_into_string(&mut self) -> Result<String> {
        let mut message = Vec::new();
        let bytes_read = self.reader.read_until(b'\n', &mut message).await?;

        match bytes_read {
            0 => Err(Error::new(
                ErrorKind::TimedOut,
                "Host connection was closed",
            )),
            n => {
                let s = String::from_utf8_lossy(&message[..n])
                    .strip_suffix("\r\n")
                    .unwrap_or("")
                    .to_string();

                if s.len() == 0 {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Data supplied is not valid",
                    ));
                }

                Ok(s)
            }
        }
    }

    async fn write_to_buffer(&mut self, buffer: &[u8]) {
        match self.writer.write(buffer).await {
            Ok(_) => (),
            Err(_) => println!("[-] ERROR: could not add data to buffer"),
        }
    }

    async fn send(&mut self) {
        self.write_to_buffer("\r\n".as_bytes()).await;
        match self.writer.flush().await {
            Ok(_) => (),
            Err(_) => println!("[-] ERROR: Could not flush to TcpStream"),
        }
    }

    pub async fn send_simple_string(&mut self, string: &str) {
        self.write_to_buffer(format!("+{string}").as_bytes()).await;
        self.send().await
    }

    pub async fn send_simple_error(&mut self, string: &str) {
        self.write_to_buffer(format!("- ERR {string}").as_bytes())
            .await;
        self.send().await
    }
}
