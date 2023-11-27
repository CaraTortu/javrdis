use crate::client::Client;
use tokio::io::{Error, ErrorKind, Result};

// Will implement the rest later on
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum DataType {
    Number(i64),
    String(String),
    Boolean(bool),
    Array(Vec<DataType>),
    Null,
    Unknown(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Ping,
    Unknown,
}

impl Command {
    #[async_recursion::async_recursion]
    pub async fn parse_data(client: &mut Client, data: &str) -> Result<DataType> {
        Ok(
            match data.chars().nth(0).expect("We should have one character") {
                // Simple strings "+Hello there\r\n"
                '+' => DataType::String(data.get(1..).unwrap_or("").to_owned()),

                // Numbers ":-12\r\n"
                ':' => {
                    let number_literal = data.get(1..);

                    if let Some(literal) = number_literal {
                        if let Ok(number) = literal.parse::<i64>() {
                            return Ok(DataType::Number(number));
                        }
                    }

                    return Err(Error::new(ErrorKind::InvalidData, "Wrong integer formmat"));
                }

                // Arrays
                '*' => {
                    let item_count_str = match data.get(1..) {
                        None => {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                "You must give a length to the array",
                            ))
                        }
                        Some(v) => v,
                    };

                    let item_count = match item_count_str.parse::<usize>() {
                        Err(_) => {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                "Invalid integer for array length",
                            ))
                        }
                        Ok(v) => v,
                    };

                    let mut items: Vec<DataType> = Vec::with_capacity(item_count);

                    for _ in 0..item_count {
                        let next_data = client.read_into_string().await?;
                        items.push(Self::parse_data(client, &next_data).await?);
                    }

                    DataType::Array(items)
                }

                // Bulk String
                '$' => {
                    let string_length: usize = data.get(1..).unwrap().parse().unwrap();
                    let s = client.read_into_string().await?;

                    if string_length > s.len() {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Given length longer than actual length",
                        ));
                    }

                    DataType::String(s.get(..string_length).unwrap().to_owned())
                }
                _ => DataType::Unknown(data.to_owned()), //_ => return Err(Error::new(ErrorKind::InvalidData, "Invalid data format")),
            },
        )
    }

    pub async fn parse_command(client: &mut Client) -> Result<Command> {
        let message = client.read_into_string().await?;

        if message.len() == 0 {
            return Err(Error::new(ErrorKind::InvalidData, "Empty payload"));
        }

        if let DataType::Array(values) = Self::parse_data(client, &message).await? {
            if values.len() == 0 {
                return Err(Error::new(ErrorKind::InvalidData, "Empty payload"));
            }
            println!("{:?}", values);
            if let DataType::String(command) = values.get(0).expect("Checked boundaries") {
                return match command.to_lowercase().as_str() {
                    "ping" => Ok(Self::Ping),
                    _ => Ok(Self::Unknown),
                };
            }
        }

        Err(Error::new(ErrorKind::InvalidData, "Invalid data type"))
    }
}
