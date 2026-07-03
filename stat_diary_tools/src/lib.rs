use time::{format_description::StaticFormatDescription, macros::format_description};

mod backup;
pub mod c_wrapper;
mod cache_handling;
pub mod data_base;
mod data_entry;
mod day_switch_offset;
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
const TIMEFORMAT: StaticFormatDescription = format_description!(
    "[year]-[month]-[day] TimeZone:([offset_hour \
         sign:mandatory]h) [hour]:[minute]:[second]"
);

/*
pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(DBLogger)).map(|()| log::set_max_level(LevelFilter::Info))
}*/

//

//

// TODO:
//
// Move file path logic to its own file.
//      Currently data file paths are used by more than one system.
//      It would be benefitial to move the logic for creating filepaths based on dates to its
//      own file. It could also be placed in db_path.rs
//
// Have time and date related code in its own file.
//      Time and dates are used in multiple places, and sometimes require the use of the day
//      switch offset.
//      This might not be needed depending on the results from moving the file path logic.
//      So make sure to think it over before doing anything with this.
//
// Move code files into different folders.
//      Currently all code files are placed directly in /src. This is fine when the number of
//      files is low, but now the number is getting a bit to high. Moving some of the files to
//      folders would help make it more organized.
//
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
// Think about the way we store data entries and the hours they belong to.
// Currently I am fairly sure we store the hour directly, meaning that with a offset of +4 hours
// 1 am is actually the next day even though when sorted by value it would show up before all
// entries of that day. It might be benefitial to instead add the offset to the hour provided.
// Meaning that if we get a entry at 2 am with a offset of +4 hours we save the entry with hour
// 2 + 24 to compensate for the offset. That way the hours are easier to sort in the correct order,
// while at the same time enabling us to safely change the day_switch_offset since we will be able
// to tell what entries have and haven't been moved.
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

    pub fn get_datafile(database: &DataBase, year: i32, month: u8, day: u8) -> DataFile {
        let datetime = DataBase::parse_datetime(year, month, day, 12).unwrap();
        let filepath = database.get_date_file_path(datetime).unwrap();
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
