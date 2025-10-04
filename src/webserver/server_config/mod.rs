use rustls::ServerConfig as RustlsConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
use std::sync::Arc;

pub struct ServerConfig {
    pub(crate) host: [u8; 4],
    pub(crate) port: u16,
    pub(crate) using_https: bool,
    pub(crate) tls_config: Option<Arc<RustlsConfig>>,
    pub(crate) base_domain: String,
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
        let certs: Result<Vec<_>, _> = CertificateDer::pem_file_iter(cert_pem)
            .unwrap()
            .collect::<Result<Vec<_>, _>>();
        let certs = certs.map_err(|e| format!("Failed to parse certificates: {}", e))?;
        let key: PrivateKeyDer = PrivateKeyDer::from_pem_file(private_key_pem).unwrap();

        if certs.is_empty() {
            return Err("No valid certificates found".to_string());
        }

        let tls_config = RustlsConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| format!("Failed to create TLS config: {}", e))?;

        self.tls_config = Some(Arc::new(tls_config));
        self.using_https = true;

        Ok(self)
    }

    pub fn set_base_domain(mut self, base_domain: String) -> Self {
        self.base_domain = base_domain;
        self
    }

    pub(crate) fn ip_as_string(&self) -> String {
        format!(
            "{}.{}.{}.{}:{}",
            self.host[0], self.host[1], self.host[2], self.host[3], self.port
        )
    }
}
