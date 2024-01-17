use itertools::Itertools;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                handle_stream(_stream)
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_stream(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    match stream.read(&mut buf) {
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
                    b"" => root(stream),
                    b"echo" => echo(stream, subpaths[2]),
                    b"user-agent" => user_agent(stream, request_lines),
                    _ => not_found(stream),
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

fn root(stream: TcpStream) {
    let ok_response = b"HTTP/1.1 200 OK\r\n\r\n";
    write_response(stream, ok_response);
}

fn echo(stream: TcpStream, arg: &[u8]) {
    let echo_response = [
        b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: ",
        arg.len().to_string().as_bytes(),
        b"\r\n\r\n",
        arg,
    ]
    .concat();
    write_response(stream, &echo_response);
}

fn user_agent(stream: TcpStream, request: Vec<&[u8]>) {
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
            write_response(stream, &response);
            return;
        }
    }
}

fn not_found(stream: TcpStream) {
    let not_found_response = b"HTTP/1.1 404 Not Found\r\n\r\n";
    write_response(stream, not_found_response);
}

fn write_response(mut stream: TcpStream, response: &[u8]) {
    stream.write(response).unwrap();
    stream.flush().unwrap();
}
