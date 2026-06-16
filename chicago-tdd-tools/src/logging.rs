use std::sync::{Arc, Mutex};
use std::io::Write;
use std::fs::File;
use std::path::Path;

/// Severity levels for log messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Designates fine-grained informational events that are most useful to debug an application.
    Debug,
    /// Designates informational messages that highlight the progress of the application at coarse-grained level.
    Info,
    /// Designates potentially harmful situations.
    Warn,
    /// Designates error events that might still allow the application to continue running.
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        };
        write!(f, "{}", s)
    }
}

/// A trait for types that can receive and process log messages.
pub trait LogSink: Send + Sync {
    /// Processes a log message with the given level.
    fn log(&self, level: LogLevel, message: &str);
}

/// A log sink that writes messages to standard output.
pub struct StdoutSink;

impl LogSink for StdoutSink {
    fn log(&self, level: LogLevel, message: &str) {
        println!("[{}] {}", level, message);
    }
}

/// A log sink that writes messages to a file.
pub struct FileSink {
    file: Mutex<File>,
}

impl FileSink {
    /// Creates a new `FileSink` that writes to the specified path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or opened.
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            file: Mutex::new(file),
        })
    }
}

impl LogSink for FileSink {
    fn log(&self, level: LogLevel, message: &str) {
        if let Ok(mut file) = self.file.lock() {
            let _ = writeln!(file, "[{}] {}", level, message);
        }
    }
}

/// A log sink that buffers messages in memory, suitable for TUI applications.
pub struct TuiBufferSink {
    buffer: Arc<Mutex<Vec<String>>>,
}

impl TuiBufferSink {
    /// Creates a new `TuiBufferSink` and returns it along with a handle to the shared buffer.
    pub fn new() -> (Self, Arc<Mutex<Vec<String>>>) {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        (Self { buffer: buffer.clone() }, buffer)
    }
}

impl LogSink for TuiBufferSink {
    fn log(&self, level: LogLevel, message: &str) {
        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.push(format!("[{}] {}", level, message));
        }
    }
}

/// The primary interface for logging messages to multiple sinks.
pub struct Logger {
    sinks: Vec<Box<dyn LogSink>>,
    min_level: LogLevel,
}

impl Logger {
    /// Creates a new `Logger` with no sinks and a minimum level of `Info`.
    pub fn new() -> Self {
        Self {
            sinks: Vec::new(),
            min_level: LogLevel::Info,
        }
    }

    /// Creates a new `Logger` with the specified minimum log level.
    pub fn with_level(level: LogLevel) -> Self {
        Self {
            sinks: Vec::new(),
            min_level: level,
        }
    }

    /// Adds a new log sink to the logger.
    pub fn add_sink(&mut self, sink: Box<dyn LogSink>) {
        self.sinks.push(sink);
    }

    /// Logs a message at the specified level if it meets the minimum level requirement.
    pub fn log(&self, level: LogLevel, message: &str) {
        if level >= self.min_level {
            for sink in &self.sinks {
                sink.log(level, message);
            }
        }
    }

    /// Logs a message at the `Debug` level.
    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    /// Logs a message at the `Info` level.
    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    /// Logs a message at the `Warn` level.
    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }

    /// Logs a message at the `Error` level.
    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::fs;

    #[test]
    fn test_logger_fanning_out() {
        let mut logger = Logger::new();
        
        let (tui_sink, buffer) = TuiBufferSink::new();
        logger.add_sink(Box::new(tui_sink));
        
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_owned();
        let file_sink = FileSink::new(&file_path).unwrap();
        logger.add_sink(Box::new(file_sink));
        
        logger.info("Test message");
        
        // Check TUI buffer
        {
            let buf = buffer.lock().unwrap();
            assert_eq!(buf.len(), 1);
            assert!(buf[0].contains("INFO"));
            assert!(buf[0].contains("Test message"));
        }
        
        // Check File
        {
            let content = fs::read_to_string(file_path).unwrap();
            assert!(content.contains("INFO"));
            assert!(content.contains("Test message"));
        }
    }

    #[test]
    fn test_log_levels() {
        let mut logger = Logger::with_level(LogLevel::Warn);
        let (tui_sink, buffer) = TuiBufferSink::new();
        logger.add_sink(Box::new(tui_sink));
        
        logger.info("Should be ignored");
        logger.warn("Should be recorded");
        logger.error("Should be recorded");
        
        let buf = buffer.lock().unwrap();
        assert_eq!(buf.len(), 2);
        assert!(buf[0].contains("WARN"));
        assert!(buf[1].contains("ERROR"));
    }
}
