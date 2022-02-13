use log::LevelFilter;
use simple_logger::SimpleLogger;

pub mod macros;

pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace
}

pub fn init_with(log_level: LogLevel) {
    let level_filter = match log_level {
        LogLevel::Error => LevelFilter::Error,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Trace => LevelFilter::Trace,
    };
    SimpleLogger::new().with_level(level_filter).init().unwrap();
}

pub fn init() {
    SimpleLogger::new().init().unwrap();
}

