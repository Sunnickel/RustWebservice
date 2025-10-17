extern crate chrono;
extern crate log;
extern crate rustls;
extern crate rustls_pki_types;
use crate::webserver::logger;

/// The web server module containing logger and other web-related functionality.
pub mod webserver;

/// A global logger instance for web server logging.
///
/// This static variable provides a convenient way to access the custom-colored
/// logger implementation throughout the app.
///
/// # Examples
///
/// ```rust
/// use log::SetLoggerError;
/// use sunweb::WEB_LOGGER;
///
/// # fn main() -> Result<(), SetLoggerError> {
/// log::set_logger(&WEB_LOGGER).unwrap();
/// log::set_max_level(log::LevelFilter::Trace);
/// // Now logging will use the colored output
/// log::info!("This is an info message");
/// log::error!("This is an error message");
/// # Ok(())
/// # }
/// ```
pub static WEB_LOGGER: logger::Logger = logger::Logger;
