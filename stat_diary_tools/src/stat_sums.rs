use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

use log::error;
use time::Date;
use walkdir::WalkDir;

use crate::{data_entry::DataFile, db_path::DataBasePath, utilities::read_lines};

//

//

#[derive(Debug)]
pub enum StatSumsError {
    Io(io::Error),
    WalkDir(walkdir::Error),
    CorruptedStatSumFile,
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
    tags: HashMap<u16, u32>,
}

impl Tags {
    /// Adds one occurance of the provided tag_id to this tags instance.
    fn add(&mut self, tag_id: u16) {
        *self.tags.entry(tag_id).or_default() += 1;
    }

    /// Removes one occurance of the provided tag_id from this tags instance.
    fn remove(&mut self, tag_id: u16) {
        match self.tags.get_mut(&tag_id) {
            Some(occurances) => *occurances -= 1,
            None => {
                log::warn!("Attempted to remove 1 instance of tag [tag_id] from the StatSumFile, but there was nothing to remove!")
            }
        }
    }

    /// Extracts the internal hashmap of this Tags instance, then returning it as a sorted vec of (id,
    /// occurances) where it is sorted by the number of occurances with the most common placed
    /// first.
    fn as_sorted_vec(&self) -> Vec<(u16, u32)> {
        let mut tags: Vec<(u16, u32)> = self.tags.clone().into_iter().collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1));
        tags
    }
}

pub struct StatSumFile {
    tags: Tags,
    path: PathBuf,
}

impl StatSumFile {
    /// Attempts to load the statsum file at the provided path. If none exist a empty StatSumFile
    /// is returned.
    /// Remembers the path provided and uses it later when the save() function is called.
    pub fn load(path: &Path) -> Result<StatSumFile, StatSumsError> {
        let Ok(lines) = read_lines(path) else {
            // File does not exist
            if let Some(parent) = path.parent() {
                create_directory(parent)?;
            }
            return Ok(StatSumFile {
                tags: Tags::default(),
                path: path.to_path_buf(),
            });
        };

        let mut tags = HashMap::new();
        for line in lines {
            let line_values: Vec<u32> = line
                .split(|c: char| !c.is_ascii_digit())
                .filter(|s| !s.is_empty())
                .map(|s| {
                    s.parse::<u32>()
                        .map_err(|_| StatSumsError::CorruptedStatSumFile)
                })
                .collect::<Result<_, _>>()?;
            if line_values.len() != 2 {
                log::error!(
                    "StatSumsFile::load(): Unexpected amount of numbers in line: [{}]",
                    line
                );
                return Err(StatSumsError::CorruptedStatSumFile);
            }
            tags.insert(line_values[1] as u16, line_values[0]);
        }
        Ok(StatSumFile {
            tags: Tags { tags },
            path: path.to_path_buf(),
        })
    }

    pub fn add_tags(&mut self, tags: &[u16]) -> &mut Self {
        for tag_id in tags {
            self.tags.add(*tag_id);
        }
        self
    }

    pub fn remove_tags(&mut self, tags: &[u16]) -> &mut Self {
        for tag_id in tags {
            self.tags.remove(*tag_id);
        }
        self
    }

    pub fn save(&mut self) -> Result<(), StatSumsError> {
        let tmp_path = self.path.with_extension("tmp");
        let mut writer = BufWriter::new(File::create(&tmp_path)?);

        for (id, occurances) in self.tags.as_sorted_vec() {
            if occurances == 0 {
                continue;
            }
            writeln!(writer, "{} {}", occurances, id)?;
        }
        writer.flush()?;
        fs::rename(tmp_path, &self.path)?;
        Ok(())
    }
}

//

//

/// Returns 3 pathbufs constructed using the provided data.
/// Paths returned are in the following order:
/// (Global_sums, Time_sums, Time_and_day_sums)
fn get_paths(db_path: &DataBasePath, date: Date, hour: u8) -> (PathBuf, PathBuf, PathBuf) {
    let base_path = db_path.stat_sums();
    let global_sums_path = base_path.join("global_sums.txt");
    let time_sums_path = base_path.join(format!("time/{:02}.txt", hour));
    let time_and_day_sums_path =
        base_path.join(format!("time_and_day/{}/{:02}.txt", date.weekday(), hour));
    (global_sums_path, time_sums_path, time_and_day_sums_path)
}

/// Adds 1 instance of the provided tags to the appropriate stat sum files.
pub fn add_tags(
    db_path: &DataBasePath,
    date: Date,
    hour: u8,
    tags: &[u16],
) -> Result<(), StatSumsError> {
    let (global, time, time_and_day) = get_paths(db_path, date, hour);

    StatSumFile::load(&global)?.add_tags(tags).save()?;
    StatSumFile::load(&time)?.add_tags(tags).save()?;
    StatSumFile::load(&time_and_day)?.add_tags(tags).save()?;
    Ok(())
}

//

//

/// Removes 1 instance of the provided tags from the appropriate stat sum files.
pub fn remove_tags(
    db_path: &DataBasePath,
    date: Date,
    hour: u8,
    tags: &[u16],
) -> Result<(), StatSumsError> {
    let (global, time, time_and_day) = get_paths(db_path, date, hour);

    StatSumFile::load(&global)?.remove_tags(tags).save()?;
    StatSumFile::load(&time)?.remove_tags(tags).save()?;
    StatSumFile::load(&time_and_day)?.remove_tags(tags).save()?;
    Ok(())
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
            Err(crate::data_entry::Error::CorruptedDataFile) => {
                error!("Data file [{:?}] is corrupted! This file will not be represented in the stat sums!", filepath);
                continue;
            }
            Err(crate::data_entry::Error::Io(io_err)) => return Err(io_err),
            _ => continue, // Remaining errors can not occur here.
        };

        for data_entry in data_file.entries().values() {
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
    if let Err(e) = fs::create_dir_all(path) {
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
    for (tag_id, occurances) in tags.as_sorted_vec() {
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
