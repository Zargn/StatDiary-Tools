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
const DIARYFILEEXTENSION: &str = "diary";

/*
pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(DBLogger)).map(|()| log::set_max_level(LevelFilter::Info))
}*/

//

//

// TODO:
// ModifyEntry function. (Change the tags and scores for one entry)
// AddEntry function. (Can use the ModifyEntry function but needs to have
//                     a check to ensure no entry exists there yet.)
// Rust-Only displayentry function for testing purposes.
// GetTagId function. (Returns the provided tags id, -1 if it doesn't exist.)
// AddTag function. (Adds the tag as long as it doesn't exist already.
//                   Returns the new tags id once added.)
// RemoveTag function.
//
// Analytical functions? Potential examples:
// - Rank tags by scores.
// - Rank tags by day-scores.
// - Rank tags by timespan scores.

pub mod utilities {
    use std::{
        fs::{self, File},
        io::{self, BufRead},
        path::{Path, PathBuf},
    };

    use crate::{
        data_base::DataBase,
        data_entry::{DataEntry, DataFile},
        db_path::DataBasePath,
        tags::TagList,
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

    pub fn print_data_file(datafile: &DataFile, taglist: &TagList) {
        let mut entries: Vec<&DataEntry> = datafile.entries().values().collect();
        entries.sort_by_key(|a| a.hour);
        for entry in entries {
            print!(
                "[{}] ms: {}, ps: {}, tags:",
                entry.hour, entry.mental_score, entry.physical_score
            );
            for tag in &entry.tags {
                print!(" {}", taglist.get_tag(*tag).unwrap());
            }
            println!();
        }
    }

    pub fn get_taglist(db_path: PathBuf) -> TagList {
        TagList::from_file(&DataBasePath::new(db_path).unwrap()).unwrap()
    }

    pub fn get_datafile(database: &DataBase, year: i32, month: i32, day: i32) -> DataFile {
        let date = DataBase::parse_date(year, month, day).unwrap();
        let filepath = database.get_data_file_path(date).unwrap();
        DataFile::open_data_file(&filepath).unwrap()
    }

    /*
    fn into_sorted_vec() -> Vec<(u16, u16)> {
        let mut tags: Vec<(u16, u16)> = self.tags.into_iter().collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1));
        tags
    }*/
}

#[cfg(test)]
mod tests {}
