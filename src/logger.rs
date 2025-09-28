use log::{Level, Metadata, Record};

const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const GREEN: &str = "\x1b[32m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                Level::Error => {
                    // Full line in red
                    println!("{}[ERROR] - {}{}", RED, record.args(), RESET);
                }
                Level::Trace => {
                    // Full line dimmed
                    println!("{}[TRACE] - {}{}", DIM, record.args(), RESET);
                }
                Level::Warn => {
                    println!("{}[WARN ]{} - {}", YELLOW, RESET, record.args());
                }
                Level::Info => {
                    println!("{}[INFO ]{} - {}", BLUE, RESET, record.args());
                }
                Level::Debug => {
                    println!("{}[DEBUG]{} - {}", GREEN, RESET, record.args());
                }
            }
        }
    }

    fn flush(&self) {}
}
