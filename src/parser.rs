use crate::types::Request;

use anyhow::Result;
use nom::{
    bytes::complete::{tag, take, take_till, take_until, take_while, take_while1, take_while_m_n},
    character::complete::line_ending,
    combinator::map_res,
    sequence::{preceded, separated_pair, terminated, Tuple},
};
use std::{collections::HashMap, str::from_utf8};

pub fn parse_request(buf: &[u8]) -> Result<Request> {
    let (headers_and_body, request_line) =
        terminated(take_until("\r\n"), line_ending::<_, ()>)(buf)?;

    let (method, endpoint, arg, version) = parse_request_line(request_line)?;

    let (body, headers) =
        terminated(take_until::<_, _, ()>("\r\n\r\n"), tag("\r\n\r\n"))(headers_and_body)?;

    let headers = parse_headers(headers)?;

    let content_length = headers.get("Content-Length");
    let content_length = match content_length {
        Some(content_length) => content_length.parse::<usize>().unwrap_or(0),
        None => 0,
    };

    let body = parse_body(body, content_length)?;

    return Ok(Request {
        method,
        endpoint,
        arg,
        version,
        headers,
        body,
    });
}

fn parse_request_line(
    request_line: &[u8],
) -> Result<(String, String, String, String), anyhow::Error> {
    let is_alpha = |c| (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z');
    let method = map_res(take_while1(is_alpha), from_utf8);
    let space = tag(" ");
    let slash = tag::<_, _, ()>("/");
    let endpoint = map_res(take_while(|c| c != b'/' && c != b' '), from_utf8);
    let slash_null_safe = take_while_m_n(0, 1, |c| c == b'/');
    let arg = map_res(take_while(|c| c != b' '), from_utf8);
    let is_version = |c| (c >= b'0' && c <= b'9') || c == b'.';
    let http = tag("HTTP/");
    let version = map_res(take_while1(is_version), from_utf8);
    let http_version = preceded(http, version);
    let (_, (method, _, _, endpoint, _, arg, _, version)) = (
        method,
        &space,
        slash,
        endpoint,
        slash_null_safe,
        arg,
        &space,
        http_version,
    )
        .parse(request_line)?;

    Ok((
        method.to_string(),
        endpoint.to_string(),
        arg.to_string(),
        version.to_string(),
    ))
}

fn parse_headers(headers: &[u8]) -> Result<HashMap<String, String>> {
    let is_alpha = |c| (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z');
    let is_header = |c| is_alpha(c) || c == b'-';
    let is_carrige_return = |c| c == b'\r';
    let header_name = map_res(take_while1(is_header), from_utf8);
    let colon = tag::<_, _, ()>(": ");
    let header_value = map_res(take_till(is_carrige_return), from_utf8);
    let mut header_parser = separated_pair(header_name, colon, header_value);
    let mut headers_map = HashMap::new();
    for header in headers.split_inclusive(|&b| b == b'\n') {
        if header.is_empty() {
            break;
        }
        let (_, (name, value)) = header_parser(header)?;
        headers_map.insert(name.to_string(), value.to_string());
    }
    Ok(headers_map)
}

fn parse_body(body: &[u8], content_length: usize) -> Result<String> {
    let mut body_parser = map_res(take::<_, _, ()>(content_length), from_utf8);
    let (_, body) = body_parser(body)?;
    Ok(body.to_string())
}
