use std::collections::HashMap;

#[derive(Clone)]
pub struct Request {
    pub protocol: String,
    pub method: String,
    pub route: String,
    pub values: HashMap<String, String>,
}

impl Request {
    pub fn new(request: String) -> Option<Request> {
        let mut lines = request.lines();

        let request_line = lines.next()?;
        let (method, route, protocol) = parse_request_line(request_line)?;

        let values = lines
            .filter_map(|line| line.split_once(':'))
            .map(|(key, value)| {
                (
                    key.trim().to_lowercase().to_string(),
                    value.trim().to_string(),
                )
            })
            .collect();

        Some(Self {
            protocol,
            method,
            route,
            values,
        })
    }

    pub fn get_cookies(&self) -> Option<HashMap<&str, &str>> {
        let mut cookies = HashMap::new();
        let mut cookie_string: Option<&String>;

        if self.values.contains_key("cookies") {
            for cookie in self.values.get("cookies").unwrap().split(';') {
                cookie_string = self.values.get(cookie);
                if cookie_string.is_some() {
                    let cookie_pair = cookie_string.unwrap().splitn(2, '=').collect::<Vec<_>>();
                    cookies.insert(cookie_pair[0], cookie_pair[1]);
                }
            }
            Some(cookies)
        } else {
            return None;
        }
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
