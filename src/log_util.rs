use std::fs::{File, OpenOptions};
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{fs, io};

use chrono::NaiveDate;
use colored::Colorize;
use lazy_static::lazy_static;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

use crate::log_level::LogLevel;
#[macro_export]
macro_rules! output_ln {
    ($($arg:tt)*) => {{
        println!($($arg)*)
    }};
}

#[macro_export]
macro_rules! output {
    ($($arg:tt)*) => {{
        print!($($arg)*)
    }};
}

#[macro_export]
macro_rules! output_debug_log_ln {
    ($time:expr, $($arg:tt)*) => {{
        output_ln!("[{} {}] {}", $time, "DEBUG".to_string().bright_black(), format!($($arg)*).bright_black().underline())
    }};
}

#[macro_export]
macro_rules! output_info_log_ln {
    ($time:expr, $($arg:tt)*) => {{
        output_ln!("[{} {}] {}", $time, "INFO".to_string(), format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! output_warn_log_ln {
    ($time:expr, $($arg:tt)*) => {{
        output_ln!("[{} {}] {}", $time, "WARN".to_string().yellow(), format!($($arg)*).yellow())
    }};
}

#[macro_export]
macro_rules! output_error_log_ln {
    ($time:expr, $($arg:tt)*) => {{
        output_ln!("[{} {}] {}", $time, "ERROR".to_string().red().bold(), format!($($arg)*).red().bold())
    }};
}

#[macro_export]
macro_rules! output_debug_log {
    ($time:expr, $($arg:tt)*) => {{
        $crate::output!("[{} {}] {}", $time, "DEBUG".to_string().bright_black(), format!($($arg)*).bright_black().underline())
    }};
}

#[macro_export]
macro_rules! output_info_log {
    ($time:expr, $($arg:tt)*) => {{
        $crate::output!("[{} {}] {}", $time, "INFO".to_string(), format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! output_warn_log {
    ($time:expr, $($arg:tt)*) => {{
        $crate::output!("[{} {}] {}", $time, "WARN".to_string().yellow(), format!($($arg)*).yellow())
    }};
}

#[macro_export]
macro_rules! output_error_log {
    ($time:expr, $($arg:tt)*) => {{
        output!("[{} {}] {}", $time, "ERROR".to_string().red().bold(), format!($($arg)*).red().bold())
    }};
}

#[macro_export]
macro_rules! get_now_time_str {
    () => {{
        // Get current time
        let now = chrono::Local::now();
        now.format("%Y-%m-%d %H:%M:%S")
    }};
}

#[macro_export]
macro_rules! output_log {
    // Match the log level and any number of other arguments
    ($level:expr, $($arg:tt)*) => {{
        let now_str = $crate::get_now_time_str!();
        // Judge the output format based on the log level
        match $level {
            LogLevel::Error => $crate::output_error_log!(now_str, $($arg)*),
            LogLevel::Warn => $crate::output_warn_log!(now_str, $($arg)*),
            LogLevel::Debug => $crate::output_debug_log!(now_str, $($arg)*),
            _ => $crate::output_info_log!(now_str, $($arg)*),
        }
    }};
}

#[macro_export]
macro_rules! output_log_ln {
    // Match the log level and any number of other arguments
    ($level:expr, $($arg:tt)*) => {{
        let now_str = get_now_time_str!();
        // Judge the output format based on the log level
        match $level {
            LogLevel::Error => $crate::output_error_log_ln!(now_str, $($arg)*),
            LogLevel::Warn => $crate::output_warn_log_ln!(now_str, $($arg)*),
            LogLevel::Debug => $crate::output_debug_log_ln!(now_str, $($arg)*),
            _ => $crate::output_info_log_ln!(now_str, $($arg)*),
        }
    }};
}

lazy_static! {
    static ref LOGGER: LogUtil = LogUtil {
        class_name: "",
        out_log_file: None,
        out_log_file_line_position: None,
        out_log_date_file: None,
        out_log_date_file_line_position: None,
        init_date: NaiveDate::default(),
        out_log_date: Arc::new(Mutex::new(NaiveDate::default()))
    };
    pub static ref MAX_LOG_LEVEL: LevelFilter = fetch_max_level_from_env();
}

pub struct LogUtil {
    class_name: &'static str,
    out_log_file: Option<Arc<Mutex<File>>>,
    out_log_file_line_position: Option<Arc<Mutex<u64>>>,
    out_log_date_file: Option<Arc<Mutex<File>>>,
    out_log_date_file_line_position: Option<Arc<Mutex<u64>>>,
    out_log_date: Arc<Mutex<NaiveDate>>,
    init_date: NaiveDate,
}

impl LogUtil {
    pub fn output_progress_msg(&self, log_level: LogLevel, msg: &str, is_process_stop: bool) {
        if log_level as u32 <= *MAX_LOG_LEVEL as u32 {
            let now = chrono::Local::now();
            let now_str = now.format("%Y-%m-%d %H:%M:%S");
            let now_date_str = now.format("%Y%m%d");
            output!("\r");
            match log_level {
                LogLevel::Debug => output_debug_log!(now_str, "{}", msg),
                LogLevel::Error => {
                    output_error_log!(now_str, "{}", msg)
                }
                LogLevel::Warn => output_warn_log!(now_str, "{}", msg),
                _ => output_info_log!(now_str, "{}", msg),
            }
            let _ = io::stdout().flush();
            if let (Some(write_file), Some(write_date_file)) =
                (self.out_log_file.as_ref(), self.out_log_date_file.as_ref())
            {
                let mut out_log_date_locked = self.out_log_date.lock().unwrap();
                if now.date_naive() != *out_log_date_locked {
                    // The dates are inconsistent; the logs need to be rolled over
                    let log_dir = get_or_create_log_dir(self.class_name);
                    let out_file_path = log_dir.join(format!("{}.log", self.class_name).as_str());
                    let out_file = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(&out_file_path)
                        .unwrap_or_else(|_| {
                            panic!(
                                "Create log file: {} failed.",
                                out_file_path.as_os_str().to_str().unwrap()
                            )
                        });
                    let out_date_file_path =
                        log_dir.join(format!("{}_{}.log", self.class_name, now_date_str).as_str());
                    let out_date_file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(&out_date_file_path)
                        .unwrap_or_else(|_| {
                            panic!(
                                "Create log file: {} failed.",
                                out_date_file_path.as_os_str().to_str().unwrap()
                            )
                        });
                    let mut write_file = write_file.lock().unwrap();
                    *write_file = out_file;
                    let mut write_date_file = write_date_file.lock().unwrap();
                    *write_date_file = out_date_file;
                    *out_log_date_locked = now.date_naive();
                }
            }
            // Write normally to the log of the current day
            if let (Some(write_file), Some(line_position)) = (
                self.out_log_file.as_ref(),
                self.out_log_file_line_position.as_ref(),
            ) {
                let mut write_file = write_file.lock().unwrap();
                let now_time = get_now_time_str!();
                // Go back to the beginning of the line
                let mut lp = line_position.lock().unwrap();
                let _ = write_file.seek(io::SeekFrom::Start(*lp));

                write!(write_file, "[{} {}] {}", now_time, log_level, msg).unwrap_or_else(|_f| {});
                // Update lp
                *lp = if let Ok(p) = write_file.stream_position() {
                    if is_process_stop {
                        p
                    } else {
                        p - (msg.len() + format!("[2024-05-08 12:24:05 {}] ", log_level).len())
                            as u64
                    }
                } else {
                    0
                };
            }
            if let (Some(write_file), Some(line_position)) = (
                self.out_log_date_file.as_ref(),
                self.out_log_date_file_line_position.as_ref(),
            ) {
                let mut write_file = write_file.lock().unwrap();
                let now_time = get_now_time_str!();
                // Go back to the beginning of the line
                let mut lp = line_position.lock().unwrap();
                let _ = write_file.seek(io::SeekFrom::Start(*lp));

                write!(write_file, "[{} {}] {}", now_time, log_level, msg).unwrap_or_else(|_f| {});
                // Update lp
                *lp = if let Ok(p) = write_file.stream_position() {
                    if is_process_stop {
                        p
                    } else {
                        p - (msg.len() + calculate_log_prefix_len(&log_level))
                            as u64
                    }
                } else {
                    0
                };
            }
        }
    }
}

#[inline]
fn calculate_log_prefix_len(log_level: &LogLevel) -> usize {
    format!("[2024-05-08 12:24:05 {}] ", log_level).len()
}

include!(concat!(env!("OUT_DIR"), "/version_info.rs"));
impl log::Log for LogUtil {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now = chrono::Local::now();
            let now_str = now.format("%Y-%m-%d %H:%M:%S");
            let now_date_str = now.format("%Y%m%d");
            // Display module path in gray for non-release builds
            let log_location_str = if !IS_RELEASE {
                if let Some(module_path) = record.module_path() {
                    format!("{} ", format!("[{module_path}]").bright_black())
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            // For Error level, also include line number
            let error_location_str = if !IS_RELEASE {
                if let (Some(module_path), Some(line)) = (record.module_path(), record.line()) {
                    format!("{} ", format!("[{module_path}:{line}]").bright_black())
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            match record.level() {
                Level::Debug => output_debug_log_ln!(now_str, "{}{}", log_location_str, record.args()),
                Level::Error => {
                    output_error_log_ln!(now_str, "{}{}", error_location_str, record.args())
                }
                Level::Warn => output_warn_log_ln!(now_str, "{}{}", log_location_str, record.args()),
                _ => output_info_log_ln!(now_str, "{}{}", log_location_str, record.args()),
            }
            if let (Some(write_file), Some(write_date_file)) =
                (self.out_log_file.as_ref(), self.out_log_date_file.as_ref())
            {
                let mut out_log_date_locked = self.out_log_date.lock().unwrap();
                if now.date_naive() != *out_log_date_locked {
                    // The dates are inconsistent; the logs need to be rolled over.
                    let log_dir = get_or_create_log_dir(self.class_name);
                    let out_file_path = log_dir.join(format!("{}.log", self.class_name).as_str());
                    let out_file = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(&out_file_path)
                        .unwrap_or_else(|_| {
                            panic!(
                                "Create log file: {} failed.",
                                out_file_path.as_os_str().to_str().unwrap()
                            )
                        });
                    let out_date_file_path =
                        log_dir.join(format!("{}_{}.log", self.class_name, now_date_str).as_str());
                    let mut out_date_file = OpenOptions::new()
                        // .append(true)
                        .write(true)
                        .read(true)
                        .create(true)
                        .open(&out_date_file_path)
                        .unwrap_or_else(|_| {
                            panic!(
                                "Create log file: {} failed.",
                                out_date_file_path.as_os_str().to_str().unwrap()
                            )
                        });
                    // Jump to the end of the file before starting to write
                    let _ = out_date_file.seek(io::SeekFrom::End(0));
                    let mut write_file = write_file.lock().unwrap();
                    *write_file = out_file;
                    let mut write_date_file = write_date_file.lock().unwrap();
                    *write_date_file = out_date_file;
                    *out_log_date_locked = now.date_naive();
                }
            }
            // Write normally to the log of the current day
            if let (Some(write_file), Some(line_position)) = (
                self.out_log_file.as_ref(),
                self.out_log_file_line_position.as_ref(),
            ) {
                let mut write_file = write_file.lock().unwrap();
                let now_time = get_now_time_str!();
                writeln!(
                    write_file,
                    "[{} {}] {}",
                    now_time,
                    record.level(),
                    record.args()
                )
                .unwrap_or_else(|_f| {});
                // modify the position at the beginning of the line
                let mut lp = line_position.lock().unwrap();
                *lp = write_file.stream_position().unwrap_or_default();
            }
            if let (Some(write_file), Some(line_position)) = (
                self.out_log_date_file.as_ref(),
                self.out_log_date_file_line_position.as_ref(),
            ) {
                let mut write_file = write_file.lock().unwrap();
                let now_time = get_now_time_str!();
                writeln!(
                    write_file,
                    "[{} {}] {}",
                    now_time,
                    record.level(),
                    record.args()
                )
                .unwrap_or_else(|_f| {});
                // modify the position at the beginning of the line
                let mut lp = line_position.lock().unwrap();
                *lp = write_file.stream_position().unwrap_or_default();
            }
        }
    }
    fn flush(&self) {}
}

fn fetch_max_level_from_env() -> LevelFilter {
    match std::env::var("RUST_LOG").unwrap_or_default().as_str() {
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "off" => LevelFilter::Off,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
}

fn get_or_create_log_dir(class_name: &str) -> PathBuf {
    let log_dir = Path::new("log");
    if !log_dir.exists() {
        fs::create_dir(log_dir).expect("Create log dir failed.");
    }
    let log_dir = log_dir.join(class_name);
    if !log_dir.exists() {
        fs::create_dir(log_dir.clone())
            .unwrap_or_else(|_| panic!("Create log/{class_name} dir failed."));
    }
    log_dir
}

impl LogUtil {
    pub fn init() -> Result<&'static LogUtil, SetLoggerError> {
        Self::init_with_logger(&LOGGER)
    }

    pub fn init_with_logger(logger: &'static LogUtil) -> Result<&'static LogUtil, SetLoggerError> {
        let max_level = fetch_max_level_from_env();
        log::set_logger(logger).map(|()| log::set_max_level(max_level))?;
        Ok(logger)
    }
    pub fn new(class_name: &'static str) -> LogUtil {
        let now_date = chrono::Local::now().date_naive();
        let (out_file, out_date_file) = if class_name.is_empty() {
            (None, None)
        } else {
            let log_dir = get_or_create_log_dir(class_name);
            let now_date_str = now_date.format("%Y%m%d").to_string();
            let out_file_path = log_dir.join(format!("{class_name}.log").as_str());
            let out_file = Arc::new(Mutex::new(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&out_file_path)
                    .unwrap_or_else(|_| {
                        panic!(
                            "Create log file: {} failed.",
                            out_file_path.as_os_str().to_str().unwrap()
                        )
                    }),
            ));
            let out_date_file_path =
                log_dir.join(format!("{class_name}_{now_date_str}.log").as_str());
            let out_date_file = Arc::new(Mutex::new({
                let mut f = OpenOptions::new()
                    // .append(true)
                    .write(true)
                    .read(true)
                    .create(true)
                    .open(&out_date_file_path)
                    .unwrap_or_else(|_| {
                        panic!(
                            "Create log file: {} failed.",
                            out_date_file_path.as_os_str().to_str().unwrap()
                        )
                    });
                let _ = f.seek(io::SeekFrom::End(0));
                f
            }));
            // Jump to the end of the file before beginning to write
            (
                Some(Arc::clone(&out_file)),
                Some(Arc::clone(&out_date_file)),
            )
        };
        LogUtil {
            class_name,
            out_log_file: out_file,
            out_log_file_line_position: Some(Arc::new(Mutex::new(0))),
            out_log_date_file: out_date_file,
            out_log_date_file_line_position: Some(Arc::new(Mutex::new(0))),
            init_date: now_date,
            out_log_date: Arc::new(Mutex::new(now_date)),
        }
    }

    pub fn set_class_name(&mut self, class_name: &'static str) {
        self.class_name = class_name;
    }
}

#[macro_export]
#[deprecated]
macro_rules! output_progress_log {
    ($log_level:expr, $($arg:tt)*) => {{
        if $log_level as u32 <= *$crate::my_log::MAX_LOG_LEVEL as u32 {
            $crate::output!("\r");
            $crate::output_log!($log_level, $($arg)*);
            let _ = io::stdout().flush();
        }
    }};
}
