use itertools::Itertools;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_stream(stream).await;
        });
    }
}

async fn handle_stream(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    match stream.read(&mut buf).await {
        Ok(_) => {
            let request_lines: Vec<&[u8]> = buf
                .split(|&b| b == b'\n')
                .filter_map(|line| {
                    if line.ends_with(b"\r") {
                        Some(&line[..line.len() - 1])
                    } else {
                        None
                    }
                })
                .collect();
            let status_line = request_lines[0];
            if let Some((_method, path, _version)) = status_line.split(|a| a == &32).collect_tuple()
            {
                let subpaths: Vec<&[u8]> = path.splitn(3, |ch| ch == &47).collect();
                match subpaths[1] {
                    b"" => root(stream).await,
                    b"echo" => echo(stream, subpaths[2]).await,
                    b"user-agent" => user_agent(stream, request_lines).await,
                    _ => not_found(stream).await,
                }
            } else {
                eprintln!("error parsing");
            }
        }
        Err(e) => {
            eprintln!("{}", e)
        }
    }
}

async fn root(stream: TcpStream) {
    let ok_response = b"HTTP/1.1 200 OK\r\n\r\n";
    write_response(stream, ok_response).await;
}

async fn echo(stream: TcpStream, arg: &[u8]) {
    let echo_response = [
        b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: ",
        arg.len().to_string().as_bytes(),
        b"\r\n\r\n",
        arg,
    ]
    .concat();
    write_response(stream, &echo_response).await;
}

async fn user_agent(stream: TcpStream, request: Vec<&[u8]>) {
    for header in request {
        if header.starts_with(b"User-Agent") {
            let agent = &header[12..];
            let response = [
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: ",
                agent.len().to_string().as_bytes(),
                b"\r\n\r\n",
                agent,
            ]
            .concat();
            write_response(stream, &response).await;
            return;
        }
    }
}

async fn not_found(stream: TcpStream) {
    let not_found_response = b"HTTP/1.1 404 Not Found\r\n\r\n";
    write_response(stream, not_found_response).await;
}

async fn write_response(mut stream: TcpStream, response: &[u8]) {
    stream.write(response).await.unwrap();
    stream.flush().await.unwrap();
}
