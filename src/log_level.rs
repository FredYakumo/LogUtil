use std::fmt;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
pub enum LogLevel {
    Debug = 4,
    Warn = 3,
    Info = 2,
    Error = 1,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LogLevel::Info => "INFO",
                LogLevel::Debug => "DEBUG",
                LogLevel::Error => "ERROR",
                LogLevel::Warn => "WARNING",
            }
        )
    }
}
