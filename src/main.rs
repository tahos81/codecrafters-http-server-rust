use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::from_utf8,
};

use itertools::Itertools;

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
    let success_response = b"HTTP/1.1 200 OK\r\n\r\n";
    let not_found_response = b"HTTP/1.1 404 Not Found\r\n\r\n";
    let mut buf = [0; 1024];
    match stream.read(&mut buf) {
        Ok(_) => {
            //let status_line = buf.split(|a| a == &10u8);
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
            println!("status_line {}", from_utf8(status_line).unwrap());
            if let Some((method, path, version)) = status_line.split(|a| a == &32).collect_tuple() {
                println!("method: {}", from_utf8(method).unwrap());
                println!("path: {}", from_utf8(path).unwrap());
                println!("version: {}", from_utf8(version).unwrap());
                match path {
                    &[47] => {
                        stream.write(success_response).unwrap();
                        stream.flush().unwrap();
                    }
                    _ => {
                        stream.write(not_found_response).unwrap();
                        stream.flush().unwrap();
                    }
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
