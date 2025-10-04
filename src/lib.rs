extern crate chrono;
extern crate log;
extern crate rustls;
extern crate rustls_pki_types;

use crate::webserver::logger;

pub mod webserver;

pub static WEB_LOGGER: logger::Logger = logger::Logger;
