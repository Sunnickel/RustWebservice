use log::error;
use rustls::{Certificate, PrivateKey, ServerConfig as RustlsConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::io::{BufReader, Cursor};
use std::sync::Arc;

pub struct ServerConfig {
    pub host: [u8; 4],
    pub port: u16,
    pub using_https: bool,
    pub tls_config: Option<Arc<RustlsConfig>>,
    pub base_domain: String,
}

impl ServerConfig {
    pub fn new(host: [u8; 4], port: u16) -> ServerConfig {
        Self {
            host,
            port,
            using_https: false,
            tls_config: None,
            base_domain: String::from("localhost"),
        }
    }

    pub fn add_cert(mut self, private_key_pem: String, cert_pem: String) -> Result<Self, String> {
        let cert_reader = &mut BufReader::new(Cursor::new(cert_pem.as_bytes()));
        let certs: Vec<Certificate> = certs(cert_reader)
            .map_err(|e| format!("Failed to parse certificates: {}", e))?
            .into_iter()
            .map(|cert| Certificate(cert))
            .collect();

        if certs.is_empty() {
            return Err("No valid certificates found".to_string());
        }

        let key_reader = &mut BufReader::new(Cursor::new(private_key_pem.as_bytes()));
        let mut keys = pkcs8_private_keys(key_reader)
            .map_err(|e| format!("Failed to parse private keys: {}", e))?;

        if keys.is_empty() {
            return Err("No private key found in key file".to_string());
        }

        let private_key = PrivateKey(keys.remove(0));

        let tls_config = RustlsConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, private_key)
            .map_err(|e| format!("Failed to create TLS config: {}", e))?;

        self.tls_config = Some(Arc::new(tls_config));
        self.using_https = true;

        Ok(self)
    }

    pub fn set_base_domain(mut self, base_domain: String) -> Self {
        self.base_domain = base_domain;
        self
    }

    pub fn ip_as_string(&self) -> String {
        format!(
            "{}.{}.{}.{}:{}",
            self.host[0], self.host[1], self.host[2], self.host[3], self.port
        )
    }
}
