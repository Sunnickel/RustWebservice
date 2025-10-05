use log::warn;
use rustls::{ClientConfig, ClientConnection, RootCertStore};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum ProxySchema {
    HTTP,
    HTTPS,
}

pub(crate) struct Proxy {
    url: String,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) path: String,
    pub(crate) scheme: ProxySchema,
}

impl Proxy {
    pub(crate) fn new(url: String) -> Self {
        Self {
            url,
            host: String::new(),
            port: 0u16,
            path: String::new(),
            scheme: ProxySchema::HTTPS,
        }
    }

    pub(crate) fn parse_url(&mut self) -> Option<()> {
        let mut parts = self.url.splitn(2, "://");
        let scheme = parts.next()?.to_lowercase();
        let rest = parts.next()?;

        let (host_port, path) = match rest.split_once('/') {
            Some((hp, p)) => (hp, format!("/{}", p)),
            None => (rest, "/".to_string()),
        };

        let (host, port) = match host_port.split_once(':') {
            Some((h, p)) => {
                let port_num = p.parse::<u16>().ok()?;
                (h.to_string(), port_num)
            }
            None => {
                let default_port = match scheme.as_str() {
                    "https" => 443,
                    "http" => 80,
                    _ => return None,
                };
                (host_port.to_string(), default_port)
            }
        };

        self.scheme = match scheme.as_str() {
            "https" => ProxySchema::HTTPS,
            "http" => ProxySchema::HTTP,
            _ => return None,
        };
        self.host = host;
        self.port = port;
        self.path = path;

        Some(())
    }

    pub(crate) fn connect_to_server(host: &str, port: u16) -> Option<TcpStream> {
        let address = format!("{}:{}", host, port);

        match TcpStream::connect(&address) {
            Ok(stream) => {
                println!("Connected to {}", address);
                Some(stream)
            }
            Err(e) => {
                warn!("Failed to connect: {}", e);
                None
            }
        }
    }

    pub(crate) fn send_http_request(
        stream: &mut TcpStream,
        path: &str,
        host: &str,
    ) -> Option<Vec<u8>> {
        // Return Vec<u8> instead of String
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nAccept-Encoding: identity\r\n\r\n",
            path, host
        );
        stream.write_all(request.as_bytes()).ok()?;

        let mut buffer = Vec::new();
        let mut chunk = [0u8; 4096];

        loop {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buffer.extend_from_slice(&chunk[..n]),
                Err(e) => {
                    warn!("Failed to read from socket: {}", e);
                    return None;
                }
            }
        }

        if buffer.is_empty() {
            None
        } else {
            Some(buffer)
        }
    }

    pub(crate) fn send_https_request(
        stream: &mut TcpStream,
        path: &str,
        host: &str,
    ) -> Option<Vec<u8>> {
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .ok()?;

        let mut root_store = RootCertStore::empty();
        for cert in rustls_native_certs::load_native_certs().certs {
            root_store.add(cert).ok()?;
        }

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let server_name = host.to_string().try_into().ok()?;
        let mut conn = ClientConnection::new(Arc::new(config), server_name).ok()?;

        while conn.is_handshaking() {
            conn.complete_io(stream).ok()?;
        }

        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nAccept-Encoding: identity\r\n\r\n",
            path, host
        );

        conn.writer().write_all(request.as_bytes()).ok()?;
        conn.complete_io(stream).ok()?;

        let mut buffer = Vec::new();
        let mut chunk = [0u8; 8192];
        let mut headers_complete = false;
        let mut is_chunked = false;

        loop {
            match conn.complete_io(stream) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => break,
                Err(_) => break,
            }

            let mut reader = conn.reader();
            match reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buffer.extend_from_slice(&chunk[..n]);

                    if !headers_complete {
                        if let Some(pos) = find_header_end(&buffer) {
                            headers_complete = true;
                            let headers = String::from_utf8_lossy(&buffer[..pos]);
                            is_chunked = headers
                                .to_lowercase()
                                .contains("transfer-encoding: chunked");
                        }
                    }

                    if headers_complete && is_chunked {
                        if buffer.ends_with(b"0\r\n\r\n") || buffer.ends_with(b"\r\n0\r\n\r\n") {
                            break;
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(_) => break,
            }
        }

        Some(buffer)
    }

    pub(crate) fn parse_http_response_bytes(response: &[u8]) -> (Vec<u8>, String) {
        if let Some(header_end) = find_header_end(response) {
            let headers_str = String::from_utf8_lossy(&response[..header_end]);
            let mut content_type = "text/html".to_string();
            let mut is_chunked = false;

            for line in headers_str.lines() {
                if line.to_lowercase().starts_with("content-type:") {
                    content_type = line
                        .split(':')
                        .nth(1)
                        .unwrap_or("text/html")
                        .trim()
                        .to_string();
                }
                if line.to_lowercase().starts_with("transfer-encoding:") {
                    if line.to_lowercase().contains("chunked") {
                        is_chunked = true;
                    }
                }
            }

            let raw_body = &response[header_end + 4..];
            let body = if is_chunked {
                decode_chunked_body(raw_body)
            } else {
                raw_body.to_vec()
            };

            (body, content_type)
        } else {
            (response.to_vec(), "text/html".to_string())
        }
    }
}

pub(crate) fn find_header_end(buffer: &[u8]) -> Option<usize> {
    for i in 0..buffer.len().saturating_sub(3) {
        if &buffer[i..i + 4] == b"\r\n\r\n" {
            return Some(i);
        }
    }
    None
}

fn decode_chunked_body(body: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut pos = 0;

    while pos < body.len() {
        // Find the chunk size line (ends with \r\n)
        let line_end = body[pos..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|p| pos + p);

        if line_end.is_none() {
            break;
        }

        let line_end = line_end.unwrap();
        let chunk_size_str = String::from_utf8_lossy(&body[pos..line_end]);
        let chunk_size_str = chunk_size_str.trim();

        // Parse hex chunk size
        let chunk_size = match usize::from_str_radix(chunk_size_str, 16) {
            Ok(size) => size,
            Err(_) => break,
        };

        if chunk_size == 0 {
            break; // Last chunk
        }

        // Extract chunk data
        pos = line_end + 1;
        if pos + chunk_size > body.len() {
            break;
        }

        result.extend_from_slice(&body[pos..pos + chunk_size]);
        pos += chunk_size;

        // Skip trailing \r\n after chunk
        if pos + 2 <= body.len() && &body[pos..pos + 2] == b"\r\n" {
            pos += 2;
        }
    }

    result
}
