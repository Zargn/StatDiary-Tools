use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{self, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

use crate::data_entry;

pub struct DataFile {
    entries: Vec<DataEntry>,
    file_path: PathBuf,
}

impl DataFile {
    /// Reads all entries in the provided file and returns a list of assembled DataEntry structs
    pub fn read_from_file(file_path: PathBuf) -> Result<DataFile, io::Error> {
        let bytes: Vec<u8> = io::BufReader::new(File::open(&file_path)?)
            .bytes()
            .map_while(Result::ok)
            .collect();

        let mut i = 0;

        let mut entries = Vec::new();

        while i < bytes.len() {
            let hour = bytes[i];
            let mental_score = bytes[i + 1];
            let physical_score = bytes[i + 2];

            let mut tags = Vec::new();
            i += 3;
            loop {
                let tag_id = ((bytes[i] as u16) << 8) | bytes[i + 1] as u16;
                if tag_id == u16::MAX {
                    i += 2;
                    break;
                }
                i += 2;

                tags.push(tag_id);
            }

            let data_entry = DataEntry::new(hour, mental_score, physical_score, tags);
            entries.push(data_entry);
        }

        Ok(DataFile { entries, file_path })
    }

    pub fn merge_tags(&mut self, tag_1: u16, tag_2: u16) {
        for data_entry in &mut self.entries {
            data_entry.merge_tags(tag_1, tag_2);
        }
    }

    pub fn save(self) -> Result<(), io::Error> {
        let mut tmp_path = self.file_path.clone();
        tmp_path.add_extension("tmp");

        let new_file = File::create(&tmp_path)?;
        let mut writer = BufWriter::new(new_file);

        for data_entry in self.entries {
            data_entry.write(&mut writer)?;
        }
        writer.flush()?;

        fs::rename(tmp_path, self.file_path)?;

        Ok(())
    }
}

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

    //

    //

    /// Reads all entries in the provided file and returns a list of assembled DataEntry structs
    pub fn read_from_file(file_path: &Path) -> Result<Vec<DataEntry>, io::Error> {
        let bytes: Vec<u8> = io::BufReader::new(File::open(file_path)?)
            .bytes()
            .map_while(Result::ok)
            .collect();

        let mut i = 0;

        let mut data_entries = Vec::new();

        while i < bytes.len() {
            let hour = bytes[i];
            let mental_score = bytes[i + 1];
            let physical_score = bytes[i + 2];

            let mut tags = Vec::new();
            i += 3;
            loop {
                let tag_id = ((bytes[i] as u16) << 8) | bytes[i + 1] as u16;
                if tag_id == u16::MAX {
                    i += 2;
                    break;
                }
                i += 2;

                tags.push(tag_id);
            }

            let data_entry = DataEntry::new(hour, mental_score, physical_score, tags);

            data_entries.push(data_entry);
        }

        Ok(data_entries)
    }

    //

    //

    fn merge_tags(&mut self, tag_1: u16, tag_2: u16) {
        let mut i = 0;
        let mut tag_found = false;
        println!("Checking for tags");
        while i < self.tags.len() {
            if self.tags[i] == tag_1 || self.tags[i] == tag_2 {
                self.tags.remove(i);
                tag_found = true;
                println!("Tag found at index {i}");
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

    //

    //

    /// Prints the provided data entry
    pub fn temp_display_entry(&self, tags: &HashMap<u16, String>) {
        print!(
            "\n {}:00 | {} | {} | ",
            self.hour, self.mental_score, self.physical_score
        );
        for tag_id in &self.tags {
            if let Some(tag) = tags.get(&tag_id) {
                print!("{} ", tag);
            } else {
                print!("UNKNOWN_ID ");
            }
        }
        println!();
    }
}
