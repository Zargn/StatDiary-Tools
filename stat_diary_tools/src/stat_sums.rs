use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
};

use log::error;
use walkdir::WalkDir;

use crate::{
    data_entry::{DataFile, ReadDataFileError},
    db_path::DataBasePath,
};

//

//

#[derive(Debug)]
pub enum StatSumsError {
    Io(io::Error),
    WalkDir(walkdir::Error),
}

impl From<io::Error> for StatSumsError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<walkdir::Error> for StatSumsError {
    fn from(value: walkdir::Error) -> Self {
        Self::WalkDir(value)
    }
}

//

//

//type Result<T> = std::result::Result<T, StatSumsError>;

/// Contains a list of tag ids and the number of times each tag id has been added to this instance.
#[derive(Debug, Default)]
struct Tags {
    tags: HashMap<u16, u16>,
}

impl Tags {
    /// Adds one occurnace of the provided tag_id to this tags instance.
    fn add(&mut self, tag_id: u16) {
        *self.tags.entry(tag_id).or_default() += 1;
    }

    /// Extracts the internal hashmap of this Tags instance, then returning it as a sorted vec of (id,
    /// occurnaces) where it is sorted by the number of occurances with the most common placed
    /// first.
    fn into_sorted_vec(self) -> Vec<(u16, u16)> {
        let mut tags: Vec<(u16, u16)> = self.tags.into_iter().collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1));
        tags
    }
}

//

//

/// Regenerates all types of tag sums for the entire database.
pub fn regenerate_tag_sums(db_path: &DataBasePath) -> Result<(), io::Error> {
    let mut general = Tags::default();
    let mut times: HashMap<u8, Tags> = HashMap::new();
    let mut day_and_times: HashMap<u8, HashMap<u8, Tags>> = HashMap::new();

    for path in WalkDir::new(db_path.data()) {
        let path = path?;
        let filepath = path.path();

        if !DataFile::is_data_file(filepath) {
            continue;
        }

        let weekday_nr = match weekday_nr_from_filename(filepath) {
            Some(nr) => nr,
            None => {
                log::warn!("Could not get weekday number from file: {filepath:?}");
                continue;
            }
        };
        let weekday_times = day_and_times.entry(weekday_nr).or_default();

        let data_file = match DataFile::read_from_file(filepath) {
            Ok(data_file) => data_file,
            Err(ReadDataFileError::CorruptedDataFile) => {
                error!("Data file [{:?}] is corrupted! This file will not be represented in the stat sums!", filepath);
                continue;
            }
            Err(ReadDataFileError::Io(io_err)) => return Err(io_err),
        };

        for data_entry in data_file.entries() {
            for tag in &data_entry.tags {
                general.add(*tag);
                times.entry(data_entry.hour).or_default().add(*tag);
                weekday_times.entry(data_entry.hour).or_default().add(*tag);
            }
        }
    }

    save_stat_sums(db_path, general, times, day_and_times)?;

    Ok(())
}

//

//

/// Saves the provided stat sums to the provided database.
fn save_stat_sums(
    db_path: &DataBasePath,
    general: Tags,
    times: HashMap<u8, Tags>,
    day_and_times: HashMap<u8, HashMap<u8, Tags>>,
) -> Result<(), io::Error> {
    let stat_sums_path = db_path.stat_sums();
    create_directory(&stat_sums_path)?;
    write_to_file(general, &stat_sums_path.join("global_sums.txt"))?;

    let time_sums_path = stat_sums_path.join("time");
    time_stats(times, &time_sums_path)?;

    let time_and_day_sums_path = stat_sums_path.join("time_and_day");
    create_directory(&time_and_day_sums_path)?;
    for (day_index, time_tags) in day_and_times.into_iter() {
        time_stats(
            time_tags,
            &time_and_day_sums_path.join(weekday_str(day_index)),
        )?;
    }
    Ok(())
}

//

//

/// Creates a directory at the provided path, then saving each time_tags instance to its own file
/// within said directory.
fn time_stats(time_tags: HashMap<u8, Tags>, path: &Path) -> Result<(), io::Error> {
    create_directory(path)?;
    for (hour, tags) in time_tags.into_iter() {
        write_to_file(tags, &path.join(format!("{:02}.txt", hour)))?;
    }
    Ok(())
}

//

//

/// Ensures a directory exists at the provided path, creating one if it doesn't exist.
fn create_directory(path: &Path) -> Result<(), io::Error> {
    if let Err(e) = fs::create_dir(path) {
        if e.kind() != io::ErrorKind::AlreadyExists {
            return Err(e);
        }
    }
    Ok(())
}

//

//

/// Writes the provided tags instance to the provided file_path.
fn write_to_file(tags: Tags, file_path: &Path) -> Result<(), io::Error> {
    let mut writer = BufWriter::new(File::create(file_path)?);
    for (tag_id, occurances) in tags.into_sorted_vec() {
        writeln!(writer, "{} {}", occurances, tag_id)?;
    }
    writer.flush()?;
    Ok(())
}

//

//

/// Attempts to get the &str filename from the provided path, returning a StatSumsError::InvalidFileName
/// if unsuccessful.
fn filename(filepath: &Path) -> Option<&str> {
    filepath.file_stem().and_then(|s| s.to_str())
}

//

//

/// Returns the weekday index in the second part of a datafile name.
/// Datafiles use the format "{day_number}-{weekday_index}.statdiary"
fn weekday_nr_from_filename(filepath: &Path) -> Option<u8> {
    let name = filename(filepath)?;
    name.split('-').nth(1)?.parse::<u8>().ok()
}

//

//

/// Returns the string representation for the provided day index.
///
/// Names start with a uppercase letter. Example: "Wednesday"
/// If a day index outside 0-6 is provided then "INVALID_DAY_INDEX" is returned.
fn weekday_str(day_index: u8) -> String {
    let result = match day_index {
        0 => "Monday",
        1 => "Tuesday",
        2 => "Wednesday",
        3 => "Thursday",
        4 => "Friday",
        5 => "Saturday",
        6 => "Sunday",
        _ => "INVALID_DAY_INDEX",
    };
    result.to_string()
}
