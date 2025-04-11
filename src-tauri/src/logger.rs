use crate::event::{self, SystemEventSender};

pub struct Logger;

impl Logger {
    pub fn init() {
        log::set_logger(&Logger).unwrap();
        log::set_max_level(log::LevelFilter::Debug);
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        match record.level() {
            log::Level::Error => {
                println!("[ERROR] {}", record.args());
                SystemEventSender.log(event::LogLevel::Error, record.args());
            }
            log::Level::Warn => {
                println!("[WARN] {}", record.args());
                SystemEventSender.log(event::LogLevel::Warn, record.args());
            }
            log::Level::Info => {
                println!("[INFO] {}", record.args());
                SystemEventSender.log(event::LogLevel::Info, record.args());
            }
            log::Level::Debug => {
                println!("[DEBUG] {}", record.args());
                SystemEventSender.log(event::LogLevel::Debug, record.args());
            }
            log::Level::Trace => {
                println!("[TRACE] {}", record.args());
            }
        }
    }

    fn flush(&self) {}
}
