use log::{LevelFilter, SetLoggerError};

use crate::logger::DBLogger;
mod backup;
pub mod c_wrapper;
mod cache_handling;
pub mod data_base;
mod data_entry;
mod db_path;
mod db_status;
mod logger;
mod stat_diary_error;
mod stat_sums;
mod tags;
mod update_database;

const DATAFILEEXTENSION: &str = "statdiary";

pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(DBLogger)).map(|()| log::set_max_level(LevelFilter::Info))
}

//

//

mod utilities {
    use std::{
        fs::{self, File},
        io::{self, BufRead},
        path::{Path, PathBuf},
    };

    //

    //

    pub fn read_lines<P>(path: P) -> io::Result<impl Iterator<Item = String>>
    where
        P: AsRef<Path>,
    {
        Ok(io::BufReader::new(File::open(path)?)
            .lines()
            .map_while(Result::ok))
    }

    //

    //

    /// Creates a sorted vec with paths visiting all items in the provided directory.
    pub fn read_sorted_directory(directory_path: &Path) -> Result<Vec<PathBuf>, io::Error> {
        let mut files = fs::read_dir(directory_path)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        files.sort();
        Ok(files)
    }
}

#[cfg(test)]
mod tests {}
