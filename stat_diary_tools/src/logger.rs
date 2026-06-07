use std::{
    fs::{File, OpenOptions},
    io::{self, BufWriter, Write},
    path::PathBuf,
    sync::Mutex,
};

use log::Level;
use time::{macros::format_description, OffsetDateTime};

pub struct DBLogger {
    writer: Mutex<BufWriter<File>>,
}

impl log::Log for DBLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut writer = self
                .writer
                .lock()
                .expect("This mutex should never possibly get poisoned.");

            let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

            // TimeZone is only displayed if the local time couldn't be determined.
            let (timezone, datetime) = match OffsetDateTime::now_local() {
                Ok(datetime) => ("", datetime),
                Err(_) => ("UTC: ", OffsetDateTime::now_utc()),
            };

            if let Err(error) = writeln!(
                writer,
                "{}{:?}: {} - {}",
                timezone,
                datetime.format(&format).unwrap(),
                record.level(),
                record.args()
            ) {
                println!("Logifle error: {}", error);
            }

            /*
            self.writer.write(format!(
                "{:?}: {} - {}",
                std::time::Instant::now(),
                record.level(),
                record.args()
            ))?; // */
            //println!("[time]: {} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {
        let mut writer = self
            .writer
            .lock()
            .expect("This mutex should never possibly get poisoned.");

        if let Err(e) = writer.flush() {
            println!("Logfile flush error: {e}");
        }
    }
}

impl DBLogger {
    pub fn new(logfile_path: PathBuf) -> Result<DBLogger, io::Error> {
        let logfile = OpenOptions::new()
            .create(true)
            .append(true)
            .open(logfile_path);
        println!("logfile::new(): {:?}", logfile);
        let writer = BufWriter::new(logfile?);

        Ok(DBLogger {
            writer: Mutex::new(writer),
        })
    }
}
