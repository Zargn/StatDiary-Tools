use std::{
    fs::{File, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

use log::Level;
use zip::DateTime;

pub struct DBLogger {
    writer: BufWriter<File>,
}

impl log::Log for DBLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            /*self.logfile.write(format!(
                "{:?}: {} - {}",
                std::time::Instant::now(),
                record.level(),
                record.args()
            ))?; */

            println!(
                "{:?}: {} - {}",
                std::time::Instant::now(),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

impl DBLogger {
    pub fn new(logfile_path: PathBuf) -> Result<DBLogger, io::Error> {
        let logfile = OpenOptions::new().append(true).open(logfile_path)?;
        let writer = BufWriter::new(logfile);

        Ok(DBLogger { writer })
    }
}
