use std::collections::HashMap;

#[derive(Clone)]
pub struct Request {
    pub protocol: String,
    pub method: String,
    pub route: String,
    pub values: HashMap<String, String>,
    pub remote_addr: String,
}

impl Request {
    pub fn new(request: String, remote_addr: String) -> Option<Request> {
        let mut lines = request.lines();

        let request_line = lines.next()?;
        let (method, route, protocol) = parse_request_line(request_line)?;

        let values: HashMap<String, String> = lines
            .filter_map(|line| line.split_once(':'))
            .map(|(key, value)| (key.trim().to_lowercase(), value.trim().to_string()))
            .collect();

        Some(Self {
            protocol,
            method,
            route,
            values,
            remote_addr,
        })
    }

    pub fn get_cookies(&self) -> Option<HashMap<&str, &str>> {
        let cookie_str = self.values.get("cookie")?;

        let mut cookies = HashMap::new();
        for cookie_pair in cookie_str.split(';') {
            if let Some((key, value)) = cookie_pair.trim().split_once('=') {
                cookies.insert(key, value);
            }
        }
        Some(cookies)
    }
}

fn parse_request_line(line: &str) -> Option<(String, String, String)> {
    let mut method = None;
    let mut path = None;
    let mut protocol = None;

    for part in line.split_whitespace() {
        let upper = part.to_uppercase();

        if upper.starts_with("HTTP/") || upper.starts_with("HTTPS/") {
            protocol = Some(part.to_string());
        } else if part.starts_with('/')
            || part.starts_with("http://")
            || part.starts_with("https://")
        {
            path = Some(part.to_string());
        } else {
            method = Some(part.to_string());
        }
    }

    Some((method?, path?, protocol?))
}
