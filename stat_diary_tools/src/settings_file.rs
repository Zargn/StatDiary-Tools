use std::{fs, io};

use crate::{
    db_path::{self, DataBasePath},
    utilities,
};

pub enum Error {
    Io(io::Error),
    DoesNotExist,
    IsCorrupted,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub struct Settings {
    day_switch_offset: i32,
}

type Result<T> = std::result::Result<T, Error>;

impl Settings {
    pub fn load(db_path: &DataBasePath) -> Result<Settings> {
        let settings_path = db_path.root().join("db_settings.txt");
        if !settings_path.exists() {
            log::error!(
                "Database at [{:?}] does not contain a settings file!",
                db_path.root()
            );
            return Err(Error::DoesNotExist);
        }
        let mut lines = utilities::read_lines(settings_path)?;
        let day_switch_offset =
            Settings::get_day_switch_offset(&lines.next().ok_or(Error::IsCorrupted)?)?;

        Ok(Settings { day_switch_offset })
    }
}

// Private functions
impl Settings {
    fn get_day_switch_offset(line: &str) -> Result<i32> {
        let value = line.split('=').nth(1).ok_or(Error::IsCorrupted)?;
        value.parse::<i32>().map_err(|_| Error::IsCorrupted)
    }
}
