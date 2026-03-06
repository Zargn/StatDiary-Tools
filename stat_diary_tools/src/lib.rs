use std::{ffi::OsStr, io};

use log::{LevelFilter, SetLoggerError};
use walkdir::WalkDir;

use crate::{
    cache_handling::regenerate_caches,
    data_entry::DataFile,
    db_path::DataBasePath,
    db_status::{ActiveTask, DBStatus, DBStatusError},
    logger::DBLogger,
    stat_sums::regenerate_tag_sums,
    tags::{TagList, TagsError},
};
mod backup;
pub mod c_wrapper;
mod cache_handling;
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

#[derive(Debug)]
pub enum RegenCachesError {
    Io(io::Error),
    DBStatus(DBStatusError),
}

/*
pub fn regenerate_caches_(db_path: &DataBasePath) -> Result<(), RegenCachesError> {
    todo!();
}*/

//

//

fn merge_tags_wrapper(db_path: &DataBasePath, tag_1: u16, tag_2: u16) -> Result<(), TagsError> {
    let Ok(db_status) = DBStatus::activate(db_path, ActiveTask::MergeTags(tag_1, tag_2)) else {
        println!("Database is busy! Aborting...");
        return Err(TagsError::TagAlreadyExists);
        //return Err(DBError::DataBaseBusy);
    };

    println!("Merging tags");
    if let Err(error) = merge_tags(db_path, tag_1, tag_2) {
        //println!("Error occured!\n{:?}", error);

        db_status.deactivate();
        return Err(error);
    } // */
    db_status.deactivate();

    println!("Regenerating Tag sums");
    if let Err(error) = regenerate_tag_sums(db_path) {
        println!("Error occured! \n{:?}", error);
        //db_status.deactivate();
    }

    println!("Regenerating Caches");
    if let Err(error) = regenerate_caches(db_path) {
        println!("Error occured!\n{:?}", error);

        //db_status.deactivate();
    }

    Ok(())
}

//

//

/*
Make sure to add a check when adding tags to fill out any potential empty space left by a
merge.
*/
fn merge_tags(db_path: &DataBasePath, tag_1: u16, tag_2: u16) -> Result<(), TagsError> {
    let mut tags = TagList::from_file(db_path)?;
    tags.merge_tags(tag_1, tag_2)?;

    for path in WalkDir::new(db_path.data()) {
        let path = path.unwrap();
        let filepath = path.path();

        if !filepath.is_file() {
            continue;
        }

        if filepath.extension() != Some(OsStr::new("statdiary")) {
            continue;
        }

        let mut data_file = DataFile::read_from_file(filepath.to_path_buf())?;

        data_file.merge_tags(tag_1, tag_2);

        data_file.save()?;
    }

    //tags.save()?;

    /*
    Get tag ids for both tag1 and tag2.

    iterate through all .statdiary files in the data directory.
        for each data_entry in file
            if tag1 could be removed from data_entry.tags
                ensure tag2 is exists in data_entry.tags.
                (Since we are merging two tags we don't want to store two duplicate tags in a single entry)


    call regenerate_tag_sums function.


    call regenerate_caches function.
    */

    Ok(())
}
fn rename_tag(db_path: &DataBasePath, old_tag: String, new_tag: String) -> Result<(), TagsError> {
    let mut tags = TagList::from_file(db_path)?;
    tags.rename_tag(old_tag, new_tag)?;
    tags.save()
}

//

//

/*
This function is meant to eventually be used to update any old database to use a newer format.
No functionality should be implemented yet as it is not the current priority.

Some potential plans for changes that would require this is:
Better tag storage. Keep the string representations in a separate file and save the tag index
in each entry instead of the full string.

Later update could be to reduce the storage needed even further.
If tags are stored as numbers instead of strings then we are technically only storing numbers
in the entries. Meaning we could avoid using a text file entirely and just go with raw bytes.
1st byte would be a u8 representing the mental score, 2nd byte would be another u8 this time
representing the physical score. After that we could have any number of double bytes
representing a u16 tag id. Then the end of that entry is marked with a double byte u16 of value
u16::MAX.
No strings or text needed. This way the only thing stored in the data files would be the actual
data and one u16 used as a marker for each entry. Much better than storing the data in a human
readable format where a lot of space is taken by "," and "|".
Although for this we should probably make both scores integers instead of floats.

For example the following row in a txt document takes up 41 bytes. (1 byte per char)
|13:00|85,8|81,5|Lunch Geoguessr Youtube|
while if it was translated to raw bytes it would only require 11 bytes.
1 > for hour of day
1 > for mental score
1 > for physical score
6 > for 3 tags (2 bytes per tag to ensure the user doesn't run out of possible tag indexes.)
2 > for end marker u16::MAX

Make sure to call RegenerateCaches after this to create all the caches in the correct folders.
*/

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
