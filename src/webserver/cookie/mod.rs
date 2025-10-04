use crate::webserver::Domain;
use chrono::{Duration, Utc};

#[derive(Clone, Copy, Debug)]
pub enum SameSite {
    None,
    Lax,
    Strict,
}

#[derive(Clone, Debug)]
pub struct Cookie {
    pub(crate) key: String,
    value: String,
    max_age: Option<u64>,
    path: String,
    domain: Domain,
    same_site: SameSite,
    secure: bool,
    is_http_only: bool,
}

impl Cookie {
    pub fn new(key: &str, value: &str, domain: &Domain) -> Cookie {
        Self {
            key: key.to_string(),
            value: value.to_string(),
            max_age: None,
            path: "/".to_string(),
            domain: domain.clone(),
            same_site: SameSite::Lax, // sensible default
            secure: false,
            is_http_only: false,
        }
    }

    pub(crate) fn as_string(&self) -> String {
        let mut base = format!("{}={}; ", self.key, self.value);

        if let Some(seconds) = self.max_age {
            base.push_str(&format!("Max-Age={}; ", seconds));

            let expires = Utc::now() + Duration::seconds(seconds as i64);
            base.push_str(&format!(
                "Expires={}; ",
                expires.format("%a, %d %b %Y %H:%M:%S GMT")
            ));
        }

        base.push_str(&format!("Path={}; ", self.path));
        base.push_str(&format!("Domain={}; ", &self.domain.name));

        let same_site_str = match self.same_site {
            SameSite::None => "None",
            SameSite::Lax => "Lax",
            SameSite::Strict => "Strict",
        };
        base.push_str(&format!("SameSite={}; ", same_site_str));

        if self.secure {
            base.push_str("Secure; ");
        }
        if self.is_http_only {
            base.push_str("HttpOnly; ");
        }

        base.trim_end().to_string()
    }

    // --- Setters ---
    pub fn expires(mut self, max_age: Option<u64>) -> Self {
        self.max_age = max_age;
        self
    }

    pub fn secure(mut self) -> Self {
        self.secure = true;
        self
    }

    pub fn http_only(mut self) -> Self {
        self.is_http_only = true;
        self
    }

    pub fn path(mut self, path: &str) -> Self {
        self.path = path.to_string();
        self
    }

    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = same_site;
        self
    }
}
