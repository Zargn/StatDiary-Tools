use std::{fmt::Display, io, path::Path};

pub enum Error {
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

type Result<T> = std::result::Result<T, Error>;

pub struct DiaryFile {}

impl DiaryFile {
    pub fn from_file(path: &Path) -> Result<DiaryFile> {
        todo!();
    }
    pub fn open(path: &Path) -> Result<DiaryFile> {
        todo!();
    }
    pub fn add_entry(&mut self, entry: DiaryEntry) -> Result<()> {
        todo!();
    }
    pub fn insert_entry(
        &mut self,
        entry_index: usize,
        entry: DiaryEntry,
    ) -> Result<Option<DiaryEntry>> {
        todo!();
    }
    pub fn remove_entry(&mut self, entry_index: usize) -> Result<()> {
        todo!();
    }
    pub fn read(&self) -> Result<&Vec<DiaryEntry>> {
        todo!();
    }
    pub fn save(self) -> Result<()> {
        todo!();
    }
}

pub struct DiaryEntry {}

impl DiaryEntry {
    pub fn new(timestamp: time::PrimitiveDateTime, text: String) -> Result<DiaryEntry> {
        todo!();
    }
}

impl Display for DiaryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!();
    }
}
