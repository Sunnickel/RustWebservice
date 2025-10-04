use log::{Level, Metadata, Record};

/// ANSI color code for red text.
const RED: &str = "\x1b[31m";

/// ANSI color code for yellow text.
const YELLOW: &str = "\x1b[33m";

/// ANSI color code for blue text.
const BLUE: &str = "\x1b[34m";

/// ANSI color code for green text.
const GREEN: &str = "\x1b[32m";

/// ANSI color code for dimmed text.
const DIM: &str = "\x1b[2m";

/// ANSI color code to reset text formatting.
const RESET: &str = "\x1b[0m";

/// A custom logger implementation that provides colored console output based on log level.
///
/// This logger uses ANSI escape codes to colorize log messages:
/// - Error messages are displayed in red.
/// - Trace messages are displayed dimmed.
/// - Warning messages are displayed in yellow.
/// - Info messages are displayed in blue.
/// - Debug messages are displayed in green.
///
/// # Examples
///
/// ```rust
/// use log::SetLoggerError;
/// use crate::webserver::logger::Logger;
///
/// # fn main() -> Result<(), SetLoggerError> {
/// log::set_logger(&Logger).unwrap();
/// log::set_max_level(log::LevelFilter::Trace);
/// // Now logging will use the colored output
/// # Ok(())
/// # }
/// ```
pub struct Logger;

impl log::Log for Logger {
    /// Determines if a log message should be processed based on its metadata.
    ///
    /// This implementation checks if the log level of the metadata is less than or equal to
    /// the maximum allowed log level set by `log::max_level()`.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The metadata associated with the log message.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if the message should be logged, `false` otherwise.
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    /// Logs a record with appropriate coloring based on its level.
    ///
    /// This method checks if the record is enabled (using `enabled`) and then formats
    /// and prints the message with color codes corresponding to its log level.
    ///
    /// # Arguments
    ///
    /// * `record` - The log record to be processed and displayed.
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

    /// Flushes any buffered records.
    ///
    /// This implementation does nothing as the logger writes directly to stdout.
    fn flush(&self) {}
}
