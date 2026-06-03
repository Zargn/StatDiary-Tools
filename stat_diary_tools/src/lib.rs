use log::{LevelFilter, SetLoggerError};

use crate::logger::DBLogger;
mod backup;
pub mod c_wrapper;
mod cache_handling;
pub mod data_base;
mod data_entry;
mod db_path;
mod db_status;
mod diary_file;
mod logger;
mod settings_file;
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
//
// Add Diary Entry function
// Insert Diary Entry function
//
// Adjust day_switch_offset function
// (
//      This will require moving data entries between datafiles.
//      For the current data entries the "day" changes at 04:00, meaning a entry at 1:00 am will
//      not be placed in the next day, but rather be left at the current day.
//      Meaning that if we want to change the offset back to 0, so a new day file begins at 00:00
//      then we will have to move any entry at 00:00 up to 04:00 to the next day file.
//
//      The best way to do this is likely to include a db_settings file which holds the current
//      offset.
//
//      When we change the offset we should make the changes on a copy of the database instead of
//      the original. Since if however unlikely the program stops mid-change there will be no way
//      to tell which entries has been moved or not. We could probably find a way to "save" what
//      has beem changed, but I think in this case a full copy and swap once complete is the better
//      choice.
// )
// Update Data Entry functions to use the day_switch_offset when selecting data files.
// Update TemporaryUpdateDataBase to take in a int representing the current offset.
// Update DataBase to include a day_switch_offset field read from db_settings.txt
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
