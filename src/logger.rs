use log::{Level, Metadata, Record};

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{: >6} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
