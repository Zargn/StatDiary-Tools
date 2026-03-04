use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
};

use walkdir::WalkDir;

use crate::data_entry::DataEntry;

/*
#[derive(Debug)]
pub enum DBAveragesError {
    IoError(io::Error),
    WalkDirError(walkdir::Error),
    InvalidFileName(String),
}*/

/*
impl From<io::Error> for DBAveragesError {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<walkdir::Error> for DBAveragesError {
    fn from(value: walkdir::Error) -> Self {
        Self::WalkDirError(value)
    }
}

impl DBAveragesError {
    pub fn into_code(self) -> i32 {
        match self {
            Self::IoError(_) => 1,
            Self::WalkDirError(_) => 2,
            Self::InvalidFileName(_) => 3,
        }
    }
} */

#[derive(Debug)]
pub enum StatSumsError {
    Io(io::Error),
    WalkDir(walkdir::Error),
    InvalidFileName(String),
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

type Result<T> = std::result::Result<T, StatSumsError>;

#[derive(Debug, Default)]
struct Tags {
    tags: HashMap<u16, u16>,
}

impl Tags {
    fn add(&mut self, tag_id: u16) {
        *self.tags.entry(tag_id).or_default() += 1;
    }

    fn to_sorted_vec(self) -> Vec<(u16, u16)> {
        let mut tags: Vec<(u16, u16)> = self.tags.into_iter().collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1));
        tags
    }
}

pub fn regenerate_tag_sums(db_path: &Path) -> Result<()> {
    let mut general = Tags::default();
    let mut times: HashMap<u8, Tags> = HashMap::new();
    let mut day_and_times: HashMap<u8, HashMap<u8, Tags>> = HashMap::new();

    for path in WalkDir::new(db_path.join("data")) {
        let path = path?;
        let filepath = path.path();

        if !filepath.is_file() {
            continue;
        }

        let filename = filename(filepath)?;

        if filepath.extension() != Some(OsStr::new("statdiary")) {
            continue;
        }

        let weekday_times = day_and_times.entry(weekday_nr(filename)?).or_default();

        for data_entry in DataEntry::read_from_file(filepath)? {
            for tag in data_entry.tags {
                general.add(tag);
                times.entry(data_entry.hour).or_default().add(tag);
                weekday_times.entry(data_entry.hour).or_default().add(tag);
            }
        }

        //println!("{:?}", filepath.file_stem());
    }

    let stat_sums_path = db_path.join("stat_sums");
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

fn time_stats(time_tags: HashMap<u8, Tags>, path: &Path) -> Result<()> {
    create_directory(path)?;
    for (hour, tags) in time_tags.into_iter() {
        write_to_file(tags, &path.join(format!("{:02}.txt", hour)))?;
    }
    Ok(())
}

fn create_directory(path: &Path) -> Result<()> {
    if let Err(e) = fs::create_dir(path) {
        if e.kind() != io::ErrorKind::AlreadyExists {
            return Err(StatSumsError::Io(e));
        }
    }
    Ok(())
}

fn write_to_file(tags: Tags, file_path: &Path) -> Result<()> {
    let mut writer = BufWriter::new(File::create(file_path)?);
    for (tag_id, occurances) in tags.to_sorted_vec() {
        writeln!(writer, "{} {}", occurances, tag_id)?;
    }
    writer.flush()?;
    Ok(())
}

fn filename(filepath: &Path) -> Result<&str> {
    filepath
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| StatSumsError::InvalidFileName(format!("{:?}", filepath)))
}

fn weekday_nr(filename: &str) -> Result<u8> {
    filename
        .split('-')
        .nth(1)
        .ok_or_else(|| StatSumsError::InvalidFileName(filename.to_string()))?
        .parse::<u8>()
        .map_err(|_| StatSumsError::InvalidFileName(filename.to_string()))
}

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
