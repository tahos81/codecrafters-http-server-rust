use anyhow::Result;
use std::{collections::HashMap, fmt::Display};
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub endpoint: String,
    pub arg: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug)]
pub struct Response {
    pub version: String,
    pub status: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub enum StatusCode {
    Ok,
    NotFound,
    Created,
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut status_str = String::new();
        match self {
            StatusCode::Ok => {
                status_str.push_str("200 OK");
            }
            StatusCode::NotFound => {
                status_str.push_str("404 Not Found");
            }
            StatusCode::Created => {
                status_str.push_str("201 Created");
            }
        };
        write!(f, "{}", status_str)
    }
}

impl Response {
    pub fn new(status: StatusCode, body: String) -> Self {
        Self {
            version: "1.1".to_string(),
            status: status.to_string(),
            headers: HashMap::new(),
            body,
        }
    }

    pub fn add_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub async fn write_to_stream(&self, stream: &mut TcpStream) -> Result<()> {
        println!("response: {}", self);
        stream.write(b"HTTP/").await?;
        stream.write(self.version.as_bytes()).await?;
        stream.write(b" ").await?;
        stream.write(self.status.as_bytes()).await?;
        stream.write(b"\r\n").await?;
        for (key, value) in &self.headers {
            stream.write(key.as_bytes()).await?;
            stream.write(b": ").await?;
            stream.write(value.as_bytes()).await?;
            stream.write(b"\r\n").await?;
        }
        stream.write(b"\r\n").await?;
        stream.write(self.body.as_bytes()).await?;
        stream.flush().await.unwrap();
        Ok(())
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res_str = String::new();
        res_str.push_str(&"HTTP/");
        res_str.push_str(&self.version);
        res_str.push_str(" ");
        res_str.push_str(&self.status);
        res_str.push_str("\r\n");
        for (key, value) in &self.headers {
            res_str.push_str(&key);
            res_str.push_str(": ");
            res_str.push_str(&value);
            res_str.push_str("\r\n");
        }
        res_str.push_str("\r\n");
        res_str.push_str(&self.body);
        write!(f, "{}", res_str)
    }
}
