use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::Path,
};

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
