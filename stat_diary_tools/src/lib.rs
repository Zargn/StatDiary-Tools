use std::{ffi::OsStr, io};

use log::{error, LevelFilter, SetLoggerError};
use walkdir::WalkDir;

use crate::{
    cache_handling::regenerate_caches,
    data_entry::{DataFile, ReadDataFileError},
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

fn merge_tags_wrapper(
    db_path: &DataBasePath,
    tag_1: u16,
    tag_2: u16,
) -> Result<(), MergeTagsError> {
    let Ok(db_status) = DBStatus::activate(db_path, ActiveTask::MergeTags(tag_1, tag_2)) else {
        println!("Database is busy! Aborting...");
        return Err(MergeTagsError::DataBaseBusy);
    };

    println!("Merging tags");
    if let Err(error) = merge_tags(db_path, tag_1, tag_2) {
        //println!("Error occured!\n{:?}", error);

        db_status.deactivate();
        return Err(error);
    } // */
    db_status.deactivate();

    if let Err(e) = regenerate_tag_sums(db_path) {
        error!(
            "MergeTags() received {:?} when attempting to regenerate tag sums!",
            e
        );
    }

    if let Err(e) = regenerate_caches(db_path) {
        error!(
            "MergeTags() received {:?} when attempting to regenerate caches!",
            e
        );
    }

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

#[derive(Debug)]
pub enum MergeTagsError {
    Io(io::Error),
    WalkDir(walkdir::Error),
    Tags(TagsError),
    DataBaseBusy,
}

impl From<io::Error> for MergeTagsError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<walkdir::Error> for MergeTagsError {
    fn from(value: walkdir::Error) -> Self {
        Self::WalkDir(value)
    }
}

impl From<TagsError> for MergeTagsError {
    fn from(value: TagsError) -> Self {
        Self::Tags(value)
    }
}

fn merge_tags(db_path: &DataBasePath, tag_1: u16, tag_2: u16) -> Result<(), MergeTagsError> {
    let mut tags = TagList::from_file(db_path)?;

    // We use get_tag here to automatically returns a error if either tag doesn't exist.
    let _ = tags.get_tag(tag_1)?;
    let _ = tags.get_tag(tag_2)?;

    tags.remove_tag(tag_1)?;

    for path in WalkDir::new(db_path.data()) {
        let path = path?;
        let filepath = path.path();
        if DataFile::is_data_file(filepath) {}

        if !filepath.is_file() {
            continue;
        }

        if filepath.extension() != Some(OsStr::new("statdiary")) {
            continue;
        }

        let mut data_file = match DataFile::read_from_file(filepath) {
            Ok(data_file) => data_file,
            Err(ReadDataFileError::CorruptedDataFile) => {
                error!("Data file [{:?}] is corrupted! This file will not be represented in the cache!", filepath);
                continue;
            }
            Err(ReadDataFileError::Io(io_err)) => return Err(MergeTagsError::Io(io_err)),
        };

        data_file.merge_tags(tag_1, tag_2);

        data_file.save()?;
    }

    tags.save()?;

    Ok(())
}

fn rename_tag(db_path: &DataBasePath, old_tag: String, new_tag: String) -> Result<(), TagsError> {
    let mut tags = TagList::from_file(db_path)?;
    tags.rename_tag(old_tag, new_tag)?;
    tags.save()
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
