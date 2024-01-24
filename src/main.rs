mod endpoints;
mod handler;
mod parser;
pub mod types;

use anyhow::Result;
use handler::handle_stream;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move { handle_stream(stream).await });
    }
}
