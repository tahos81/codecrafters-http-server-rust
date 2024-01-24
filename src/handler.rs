use crate::{endpoints, parser::parse_request};

use anyhow::Result;
use tokio::{io::AsyncReadExt, net::TcpStream};

pub async fn handle_stream(mut stream: TcpStream) -> Result<()> {
    let mut buf = [0; 1024];

    stream.read(&mut buf).await?;

    let request = parse_request(&buf)?;
    println!("request: {:?}", request);

    match request.endpoint.as_str() {
        "" => endpoints::root(stream, &request).await,
        "echo" => endpoints::echo(stream, &request).await,
        "user-agent" => endpoints::user_agent(stream, &request).await,
        "files" => endpoints::files(stream, &request).await,
        _ => endpoints::not_found(stream).await,
    }?;

    Ok(())
}
