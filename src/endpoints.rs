use crate::types::{Request, Response, StatusCode};

use anyhow::Result;
use std::{env, path::Path};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn root(mut stream: TcpStream, _request: &Request) -> Result<()> {
    let response = Response::new(StatusCode::Ok, String::new());
    response.write_to_stream(&mut stream).await?;
    Ok(())
}

pub async fn echo(mut stream: TcpStream, request: &Request) -> Result<()> {
    let mut response = Response::new(StatusCode::Ok, request.arg.clone());
    response.add_header("Content-Type".to_string(), "text/plain".to_string());
    response.add_header("Content-Length".to_string(), request.arg.len().to_string());
    response.write_to_stream(&mut stream).await?;
    Ok(())
}

pub async fn user_agent(mut stream: TcpStream, request: &Request) -> Result<()> {
    let agent = request.headers.get("User-Agent");
    let agent = match agent {
        Some(agent) => agent,
        None => "",
    };
    let mut response = Response::new(StatusCode::Ok, agent.to_string());
    response.add_header("Content-Type".to_string(), "text/plain".to_string());
    response.add_header("Content-Length".to_string(), agent.len().to_string());
    response.write_to_stream(&mut stream).await?;
    Ok(())
}

pub async fn not_found(mut stream: TcpStream) -> Result<()> {
    let response = Response::new(StatusCode::NotFound, String::new());
    response.write_to_stream(&mut stream).await?;
    Ok(())
}

pub async fn files(mut stream: TcpStream, request: &Request) -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let directory = args.get(2).expect("no directory");
    let filename = request.arg.as_str();
    let path = Path::new(directory);
    match request.method.as_str() {
        "GET" => {
            let file = File::open(path.join(filename)).await;
            match file {
                Ok(mut file) => {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).await.unwrap();
                    let content_length = contents.len();
                    let mut response = Response::new(StatusCode::Ok, contents);
                    response.add_header(
                        "Content-Type".to_string(),
                        "application/octet-stream".to_string(),
                    );
                    response.add_header("Content-Length".to_string(), content_length.to_string());
                    println!("files response: {}", response);
                    response.write_to_stream(&mut stream).await?;
                }
                Err(_) => {
                    not_found(stream).await?;
                }
            }
        }
        "POST" => {
            let mut file = File::create(path.join(filename)).await?;
            file.write_all(request.body.as_bytes()).await?;
            file.flush().await?;
            let response = Response::new(StatusCode::Created, String::new());
            response.write_to_stream(&mut stream).await?;
        }
        _ => {
            not_found(stream).await?;
        }
    }
    Ok(())
}
