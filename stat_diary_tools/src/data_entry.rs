use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

use log::warn;

use crate::{DATAFILEEXTENSION, DIARYFILEEXTENSION};

/*
#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

impl Error {
    fn with_kind(kind: ErrorKind) -> Error {
        Self { kind }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::with_kind(ErrorKind::Io(value))
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Io(io::Error),
    EntryAlreadyExists,
    CorruptedDataFile,
    InvalidDate,
} */

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    EntryAlreadyExists,
    CorruptedDataFile,
    InvalidDate,
    InvalidData,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

/// Contains all data entries for one data file and the filepath to said file.
pub struct DataFile {
    entries: HashMap<u8, DataEntry>,
    file_path: PathBuf,
}

/// Reads the byte at the provided index in the list of bytes, returning the byte or a
/// ReadDataFileError::CorruptedDataFile error if the index is out of range.
fn read_at_index(bytes: &[u8], index: usize) -> Result<&u8, Error> {
    bytes.get(index).ok_or(Error::CorruptedDataFile)
}

impl DataFile {
    /// Reads all entries in the provided file and returns a list of assembled DataEntry structs
    pub fn read_from_file(file_path: &Path) -> Result<DataFile, Error> {
        let bytes: Vec<u8> = io::BufReader::new(File::open(file_path)?)
            .bytes()
            .map_while(Result::ok)
            .collect();

        let mut i = 0;

        let mut entries = HashMap::new();

        while i < bytes.len() {
            let hour = read_at_index(&bytes, i)?;
            let mental_score = read_at_index(&bytes, i + 1)?;
            let physical_score = read_at_index(&bytes, i + 2)?;

            let mut tags = Vec::new();
            i += 3;
            loop {
                let tag_id = ((*read_at_index(&bytes, i)? as u16) << 8)
                    | *read_at_index(&bytes, i + 1)? as u16;
                if tag_id == u16::MAX {
                    i += 2;
                    break;
                }
                i += 2;

                tags.push(tag_id);
            }

            let data_entry = DataEntry::new(*hour, *mental_score, *physical_score, tags);
            entries.insert(*hour, data_entry);
        }

        Ok(DataFile {
            entries,
            file_path: file_path.to_path_buf(),
        })
    }

    pub fn open_data_file(file_path: &Path) -> Result<DataFile, Error> {
        let Some(file_dir) = file_path.parent() else {
            log::warn!(
                "DataFile::open_data_file(): {:?}.parent() was None!",
                file_path
            );
            return Err(Error::InvalidDate);
        };
        fs::create_dir_all(file_dir)?;

        if file_path.exists() {
            return DataFile::read_from_file(file_path);
        }

        let datafile = DataFile {
            entries: HashMap::new(),
            file_path: file_path.to_path_buf(),
        };

        Ok(datafile)
    }

    /// Inserts the new entry at its hour index, overwriting any existing entry already there.
    pub fn overwrite_entry(&mut self, new_entry: DataEntry) {
        self.entries.insert(new_entry.hour, new_entry);
    }

    /// Adds the new entry to the datafile.
    /// Returns an error if a entry already exist with the same hour.
    pub fn add_entry(&mut self, new_entry: DataEntry) -> Result<(), Error> {
        if self.entries.contains_key(&new_entry.hour) {
            return Err(Error::EntryAlreadyExists);
        }
        self.entries.insert(new_entry.hour, new_entry);
        Ok(())
    }

    //

    //

    /// Returns true if the proivded file is a data file and false if it isn't.
    /// Will log the reason a file is decided to not be a data file to the logger.
    /// If a non-datafile of expected name is encountered this will return false without
    /// logging the reason.
    pub fn is_data_file(file: &Path) -> bool {
        // A directory can not be a datafile.
        if file.is_dir() {
            //info!("DataFile: Ignoring directory: {:?}", file);
            return false;
        }

        let Some(file_extension) = file.extension() else {
            warn!(
                "DataFile::is_data_file(): Ignoring file due to missing file extension! {:?}",
                file
            );
            return false;
        };

        // Don't give warning for expected non-data files.
        // This includes cache files and text diary files.
        if file.file_name() == Some(OsStr::new("month_cache.txt"))
            || file.file_name() == Some(OsStr::new("year_cache.txt"))
            || file_extension == DIARYFILEEXTENSION
        {
            return false;
        }

        if file_extension != DATAFILEEXTENSION {
            warn!("DataFile::is_data_file(): Found unexpected file {:?}", file);
            return false;
        }
        true
    }

    //

    //

    /// Returns a reference to the internal HashMap of data entries.
    pub fn entries(&self) -> &HashMap<u8, DataEntry> {
        &self.entries
    }

    //

    //

    /// Merges tag_2 into tag_1 in each data entry in this file.
    /// One way to visualise what this does is to imagine that the id of tag_2 is changed to the
    /// same as tag_1, after which any duplicate ids are removed.
    pub fn merge_tags(&mut self, tag_1: u16, tag_2: u16) -> &mut Self {
        for data_entry in self.entries.values_mut() {
            data_entry.merge_tags(tag_1, tag_2);
        }
        self
    }

    //

    //

    /// Removes `tag_id` from all data entries in this file.
    pub fn remove_tag(&mut self, tag_id: u16) -> &mut Self {
        for data_entry in self.entries.values_mut() {
            data_entry.remove_tag(tag_id);
        }
        self
    }

    //

    //

    /// Saves this data file to the location it was read from. The old file is overwritten.
    pub fn save(&mut self) -> Result<(), io::Error> {
        let mut tmp_path = self.file_path.clone();
        tmp_path.add_extension("tmp");

        let new_file = File::create(&tmp_path)?;
        let mut writer = BufWriter::new(new_file);

        for data_entry in self.entries.values() {
            data_entry.write(&mut writer)?;
        }
        writer.flush()?;

        fs::rename(tmp_path, &self.file_path)?;

        Ok(())
    }
}

//

//

/// Contains one statdiary data entry.
#[derive(Clone)]
pub struct DataEntry {
    pub hour: u8,
    pub mental_score: u8,
    pub physical_score: u8,
    pub tags: Vec<u16>,
}

impl DataEntry {
    pub fn new(hour: u8, mental_score: u8, physical_score: u8, tags: Vec<u16>) -> DataEntry {
        DataEntry {
            hour,
            mental_score,
            physical_score,
            tags,
        }
    }

    pub fn from_c_data(data: &[u16]) -> Result<DataEntry, Error> {
        if data.len() < 3 {
            log::error!("DataEntry::from_c_data(): Data array was too short! Len was {} when 3 is mandatory!", data.len())
        }

        let hour = {
            if !(0..24).contains(&data[0]) {
                log::error!(
                    "DataEntry::from_c_data(): Hour [{}] is out of range!",
                    data[0]
                );
                return Err(Error::InvalidData);
            }
            data[0] as u8
        };
        let m_score = Self::validate_score(data[1])?;
        let p_score = Self::validate_score(data[2])?;

        let mut tags = Vec::new();

        for tag in data.iter().skip(3) {
            tags.push(*tag);
        }

        Ok(DataEntry::new(hour, m_score, p_score, tags))
    }

    fn validate_score(score: u16) -> Result<u8, Error> {
        if !(0..=100).contains(&score) {
            log::error!(
                "DataEntry::from_c_data(): Score [{}] is out of range!",
                score
            );
            return Err(Error::InvalidData);
        }
        Ok(score as u8)
    }

    //

    //

    /// Merges the two provided tags into one in this entry.
    /// tag_2 is merged with tag_1, meaning that if tag_1 or/and tag_2 exists in this entry then
    /// only one tag_1 will be left. If tag_2 exists but not tag_1 then tag_2 will be replaced by
    /// tag_1. If only tag_1 exist no change is made.
    fn merge_tags(&mut self, tag_1: u16, tag_2: u16) {
        let mut i = 0;
        let mut tag_found = false;
        while i < self.tags.len() {
            if self.tags[i] == tag_1 || self.tags[i] == tag_2 {
                self.tags.remove(i);
                tag_found = true;
            } else {
                i += 1;
            }
        }
        if tag_found {
            self.tags.push(tag_2);
        }
    }

    //

    //

    /// Removes any occurance of `tag_id` from this entry.
    fn remove_tag(&mut self, tag_id: u16) {
        let mut i = 0;
        while i < self.tags.len() {
            if self.tags[i] == tag_id {
                self.tags.remove(i);
            } else {
                i += 1;
            }
        }
    }

    //

    //

    /// Writes this data_entry in bytes to the provided writer. Ending the write with a 2 byte
    /// u16::MAX marker.
    pub fn write(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        writer.write_all(&[self.hour, self.mental_score, self.physical_score])?;

        for tag_id in &self.tags {
            writer.write_all(&tag_id.to_be_bytes())?;
        }

        writer.write_all(&u16::MAX.to_be_bytes())?;
        Ok(())
    }
}
