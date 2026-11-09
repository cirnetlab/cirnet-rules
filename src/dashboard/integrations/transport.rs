//! Small HTTP/1.x response reader for one bounded loopback JSON probe.
use std::{
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpStream},
    time::{Duration, Instant},
};

const DEADLINE: Duration = Duration::from_millis(1_200);
const MAX_HEADERS: usize = 8 * 1024;
const MAX_BODY: usize = 32 * 1024;

pub(super) enum Failure {
    Connect,
    Response,
}

pub(super) struct Response {
    pub(super) status: u16,
    pub(super) body: Vec<u8>,
}

pub(super) fn get(port: u16, endpoint: &str) -> Result<Response, Failure> {
    let deadline = Instant::now() + DEADLINE;
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut stream =
        TcpStream::connect_timeout(&address, DEADLINE).map_err(|_| Failure::Connect)?;
    stream
        .set_write_timeout(Some(remaining(deadline)?))
        .map_err(|_| Failure::Response)?;
    write!(
        stream,
        "GET {endpoint} HTTP/1.0\r\nHost: {address}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
    )
    .map_err(|_| Failure::Response)?;

    let mut bytes = Vec::with_capacity(4096);
    let mut chunk = [0_u8; 4096];
    let (status, body_start, length) = loop {
        let previous = bytes.len();
        let count = read(&mut stream, &mut chunk, deadline)?;
        if count == 0 {
            return Err(Failure::Response);
        }
        bytes.extend_from_slice(&chunk[..count]);
        let scan_start = previous.saturating_sub(3);
        if let Some(offset) = bytes[scan_start..]
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
        {
            let header_end = scan_start + offset;
            if header_end > MAX_HEADERS {
                return Err(Failure::Response);
            }
            let (status, length) = headers(&bytes[..header_end])?;
            break (status, header_end + 4, length);
        }
        if bytes.len() > MAX_HEADERS {
            return Err(Failure::Response);
        }
    };

    if status != 200 {
        return Ok(Response {
            status,
            body: Vec::new(),
        });
    }
    let mut body = bytes.split_off(body_start);
    loop {
        if body.len() > MAX_BODY || length.is_some_and(|size| body.len() > size) {
            return Err(Failure::Response);
        }
        if length == Some(body.len()) {
            break;
        }
        let count = read(&mut stream, &mut chunk, deadline)?;
        if count == 0 {
            if length.is_some() {
                return Err(Failure::Response);
            }
            break;
        }
        body.extend_from_slice(&chunk[..count]);
    }
    Ok(Response { status, body })
}

fn remaining(deadline: Instant) -> Result<Duration, Failure> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err(Failure::Response);
    }
    Ok(remaining)
}

fn read(stream: &mut TcpStream, output: &mut [u8], deadline: Instant) -> Result<usize, Failure> {
    loop {
        stream
            .set_read_timeout(Some(remaining(deadline)?))
            .map_err(|_| Failure::Response)?;
        match stream.read(output) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            result => return result.map_err(|_| Failure::Response),
        }
    }
}

fn headers(bytes: &[u8]) -> Result<(u16, Option<usize>), Failure> {
    let headers = std::str::from_utf8(bytes).map_err(|_| Failure::Response)?;
    let (line, headers) = headers.split_once("\r\n").unwrap_or((headers, ""));
    let mut status_line = line.split_ascii_whitespace();
    if !matches!(status_line.next(), Some("HTTP/1.0" | "HTTP/1.1")) {
        return Err(Failure::Response);
    }
    let status = status_line
        .next()
        .ok_or(Failure::Response)?
        .parse::<u16>()
        .map_err(|_| Failure::Response)?;
    if !(100..=599).contains(&status) {
        return Err(Failure::Response);
    }
    if status != 200 {
        return Ok((status, None));
    }
    let mut length = None;
    for header in headers.lines() {
        let (name, value) = header.split_once(':').ok_or(Failure::Response)?;
        let value = value.trim();
        if name.eq_ignore_ascii_case("content-length") {
            let count = value.parse::<usize>().map_err(|_| Failure::Response)?;
            if length.is_some() || count > MAX_BODY {
                return Err(Failure::Response);
            }
            length = Some(count);
        } else if name.eq_ignore_ascii_case("transfer-encoding")
            || name.eq_ignore_ascii_case("content-encoding")
                && !value.eq_ignore_ascii_case("identity")
        {
            return Err(Failure::Response);
        }
    }
    Ok((status, length))
}
