use rustls::ServerConfig as RustlsConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
use std::sync::Arc;

/// Configuration for the web server.
///
/// This struct holds all the necessary information to configure a web server,
/// including network settings, TLS configuration, and domain information.
///
/// # Examples
///
/// ```rust
/// use your_crate::webserver::server_config::ServerConfig;
///
/// let config = ServerConfig::new([127, 0, 0, 1], 8080)
///     .set_base_domain("example.com".to_string());
/// ```
pub struct ServerConfig {
    /// The IP address the server will bind to.
    pub(crate) host: [u8; 4],
    /// The port number the server will listen on.
    pub(crate) port: u16,
    /// Indicates whether HTTPS is enabled for the server.
    pub(crate) using_https: bool,
    /// Optional TLS configuration for secure connections.
    pub(crate) tls_config: Option<Arc<RustlsConfig>>,
    /// The base domain used for the server.
    pub(crate) base_domain: String,
}

impl ServerConfig {
    /// Creates a new `ServerConfig` with the specified host and port.
    ///
    /// By default, HTTPS is disabled and no TLS configuration is set.
    /// The base domain is initialized to "localhost".
    ///
    /// # Arguments
    ///
    /// * `host` - An array of 4 u8 values representing the IPv4 address.
    /// * `port` - The port number to listen on.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use your_crate::webserver::server_config::ServerConfig;
    ///
    /// let config = ServerConfig::new([127, 0, 0, 1], 8080);
    /// ```
    pub fn new(host: [u8; 4], port: u16) -> ServerConfig {
        Self {
            host,
            port,
            using_https: false,
            tls_config: None,
            base_domain: String::from("localhost"),
        }
    }

    /// Adds TLS certificate and private key to the server configuration.
    ///
    /// This method enables HTTPS for the server by configuring TLS settings
    /// with the provided certificate and private key files.
    ///
    /// # Arguments
    ///
    /// * `private_key_pem` - Path to the PEM file containing the private key.
    /// * `cert_pem` - Path to the PEM file containing the certificate(s).
    ///
    /// # Returns
    ///
    /// * `ServerConfig` - The updated server configuration with TLS enabled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use your_crate::webserver::server_config::ServerConfig;
    ///
    /// let config = ServerConfig::new([127, 0, 0, 1], 8080)
    ///     .add_cert("private_key.pem".to_string(), "cert.pem".to_string())
    ///     .expect("Failed to add certificate");
    /// ```
    pub fn add_cert(mut self, private_key_pem: String, cert_pem: String) -> Self {
        let certs: Result<Vec<_>, _> = CertificateDer::pem_file_iter(cert_pem)
            .unwrap()
            .collect::<Result<Vec<_>, _>>();
        let certs = certs.map_err(|e| format!("Failed to parse certificates: {}", e));
        let key: PrivateKeyDer = PrivateKeyDer::from_pem_file(private_key_pem).unwrap();

        if certs.clone().unwrap().is_empty() {
            panic!("Failed to parse certificates");
        }

        let tls_config = RustlsConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs.unwrap(), key)
            .map_err(|e| format!("Failed to create TLS config: {}", e));

        self.tls_config = Some(Arc::new(tls_config.unwrap()));
        self.using_https = true;

        self
    }

    /// Sets the base domain for the server.
    ///
    /// This domain is used as a default for various server operations,
    /// such as generating URLs or handling cookies.
    ///
    /// # Arguments
    ///
    /// * `base_domain` - The base domain string to set.
    ///
    /// # Returns
    ///
    /// * `ServerConfig` - The updated server configuration with the new base domain.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use your_crate::webserver::server_config::ServerConfig;
    ///
    /// let config = ServerConfig::new([127, 0, 0, 1], 8080)
    ///     .set_base_domain("example.com".to_string());
    /// ```
    pub fn set_base_domain(mut self, base_domain: String) -> Self {
        self.base_domain = base_domain;
        self
    }

    /// Converts the server configuration to a string representation.
    ///
    /// This method returns a formatted string containing the IP address and port,
    /// useful for logging or debugging purposes.
    ///
    /// # Returns
    ///
    /// * `String` - A string in the format "ip.ip.ip.ip:port".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use your_crate::webserver::server_config::ServerConfig;
    ///
    /// let config = ServerConfig::new([127, 0, 0, 1], 8080);
    /// assert_eq!(config.ip_as_string(), "127.0.0.1:8080");
    /// ```
    pub(crate) fn ip_as_string(&self) -> String {
        format!(
            "{}.{}.{}.{}:{}",
            self.host[0], self.host[1], self.host[2], self.host[3], self.port
        )
    }
}
