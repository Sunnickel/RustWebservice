pub fn generate_response(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
        Date: Sun, 28 Sep 2025 12:01:00 GMT\r\n\
        Server: ExampleServer/1.0\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\
        \r\n\
        {}",
        body.len(),
        body
    )
}

pub fn generate_static_response(body: &str, content_type: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
        Date: Sun, 28 Sep 2025 12:01:00 GMT\r\n\
        Server: ExampleServer/1.0\r\n\
        Content-Type: {}\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\
        \r\n\
        {}",
        content_type,
        body.len(),
        body
    )
}
