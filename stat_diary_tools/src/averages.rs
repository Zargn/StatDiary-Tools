use std::{collections::HashMap, io, path::Path};

use walkdir::WalkDir;

use crate::data_entry::DataEntry;

#[derive(Debug)]
pub enum DBAveragesError {
    IoError(io::Error),
    WalkDirError(walkdir::Error),
    InvalidFileName(String),
}

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
}

type Result<T> = std::result::Result<T, DBAveragesError>;

#[derive(Debug, Default)]
struct Tags {
    tags: HashMap<u16, u16>,
}

impl Tags {
    fn add(&mut self, tag_id: u16) {
        *self.tags.entry(tag_id).or_default() += 1;
    }
}

pub fn regenerate_tag_sums(db_path: &Path) -> Result<()> {
    let mut general = Tags::default();
    let mut times: HashMap<u8, Tags> = HashMap::new();
    let mut day_and_times: HashMap<u8, HashMap<u8, Tags>> = HashMap::new();

    for path in WalkDir::new(db_path.join("data")) {
        let filepath = path?;
        if filepath.path().is_file() {
            let filename = filepath
                .path()
                .file_stem()
                .ok_or(DBAveragesError::InvalidFileName(format!("{:?}", filepath)))?
                .to_str()
                .ok_or(DBAveragesError::InvalidFileName(format!("{:?}", filepath)))?;

            let mut parts = filename.split('-');
            parts.next();
            let weekday_nr = parts
                .next()
                .ok_or(DBAveragesError::InvalidFileName(filename.to_string()))?
                .parse::<u8>()
                .map_err(|_| DBAveragesError::InvalidFileName(filename.to_string()))?;

            let mut weekday_times = day_and_times.entry(weekday_nr).or_default();

            for data_entry in DataEntry::read_from_file(filepath.path())? {
                for tag in data_entry.tags {
                    general.add(tag);
                    times.entry(data_entry.hour).or_default().add(tag);
                    weekday_times.entry(data_entry.hour).or_default().add(tag);
                }
            }

            println!("{:?}", filepath.path().file_stem());
        }
    }

    Ok(())
}
