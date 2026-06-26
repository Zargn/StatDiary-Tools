use std::{
    fmt::Display,
    fs::{self, File},
    io::{self, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

use time::OffsetDateTime;

use crate::{DIARYFILEEXTENSION, TIMEFORMAT};

pub enum Error {
    Io(io::Error),
    NotADiaryFile,
    CorruptedDiaryFile,
    EntryIndexDoesNotExist,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

const ENTRYSEPARATOR: &str = "\n\nTHIS IS A SEPARATOR! DO NOT WRITE THE SAME LINES MANUALLY!
----------------------------------------------------\n";

type Result<T> = std::result::Result<T, Error>;

pub struct DiaryFile {
    entries: Vec<DiaryEntry>,
    file_path: PathBuf,
}

impl DiaryFile {
    pub fn from_file(file_path: &Path) -> Result<DiaryFile> {
        if !DiaryFile::is_diary_file(file_path) {
            log::error!("DiaryFile::from_file(): {file_path:?} does not point to a diary file!");
            return Err(Error::NotADiaryFile);
        }

        let mut file_text = String::new();
        io::BufReader::new(File::open(file_path)?).read_to_string(&mut file_text)?;

        let mut entries = Vec::new();
        for text_block in file_text.split(ENTRYSEPARATOR) {
            entries.push(DiaryEntry::from_block(text_block)?);
        }

        Ok(DiaryFile {
            entries,
            file_path: file_path.to_path_buf(),
        })
    }

    pub fn is_diary_file(path: &Path) -> bool {
        let Some(file_extension) = path.extension() else {
            return false;
        };
        file_extension == DIARYFILEEXTENSION
    }

    pub fn open(file_path: &Path) -> Result<DiaryFile> {
        if file_path.exists() && DiaryFile::is_diary_file(file_path) {
            DiaryFile::from_file(file_path)
        } else if file_path.exists() {
            log::error!("DiaryFile::open(): {file_path:?} does not point to a diary file!");
            Err(Error::NotADiaryFile)
        } else {
            let Some(file_dir) = file_path.parent() else {
                log::warn!("DiaryFile::open(): {:?}.parent() was None!", file_path);
                return Err(Error::NotADiaryFile);
            };
            fs::create_dir_all(file_dir)?;
            Ok(DiaryFile {
                entries: Vec::new(),
                file_path: file_path.to_path_buf(),
            })
        }
    }

    pub fn add_entry(&mut self, entry: DiaryEntry) -> &mut DiaryFile {
        self.entries.push(entry);
        self
    }

    pub fn remove_entry(&mut self, entry_index: usize) -> Result<()> {
        if entry_index >= self.entries.len() {
            log::error!("DiaryFile.remove_entry(): Attempted to remove an entry but the index was out of range!");
            return Err(Error::EntryIndexDoesNotExist);
        }
        self.entries.remove(entry_index);
        Ok(())
    }

    /// Returns the internal Vec holding all diary entries of this file.
    /// NOTE: This vec is not sorted automatically after changes!
    ///       This means that any newly added entry
    pub fn read(&self) -> &Vec<DiaryEntry> {
        &self.entries
    }

    pub fn save(&mut self) -> Result<()> {
        self.sort_entries();

        let mut tmp_path = self.file_path.clone();
        tmp_path.add_extension("tmp");

        let mut writer = BufWriter::new(File::create(&tmp_path)?);

        let s = self
            .entries
            .iter()
            .flat_map(|s| [s.to_string(), ENTRYSEPARATOR.to_string()])
            .take(self.entries.iter().len() * 2 - 1)
            .collect::<String>();

        write!(writer, "{}", s)?;

        writer.flush()?;

        fs::rename(tmp_path, &self.file_path)?;

        Ok(())
    }

    pub fn sort_entries(&mut self) {
        self.entries.sort_by_key(|a| a.timestamp);
    }
}

const TIMETITLESEPARATOR: &str = "   ";

pub struct DiaryEntry {
    timestamp: OffsetDateTime,
    title: String,
    text: String,
}

impl DiaryEntry {
    pub fn new(title: String, text: String, timestamp: OffsetDateTime) -> DiaryEntry {
        DiaryEntry {
            timestamp,
            title,
            text,
        }
    }

    pub fn from_block(text_block: &str) -> Result<DiaryEntry> {
        let mut text_block_parts = text_block.splitn(2, "\n\n");
        /*
        println!("{:?}", text_block_parts);
        let t = text_block_parts.next().unwrap();
        println!("t: {t}");
        println!("{:?}", text_block_parts); // */
        let mut time_and_title_line = text_block_parts
            .next()
            .expect("The first element always exists.")
            //.ok_or(Error::CorruptedDiaryFile)?
            .split(TIMETITLESEPARATOR);
        let time_block = time_and_title_line
            .next()
            .expect("The first element always exists.");
        let Ok(timestamp) = OffsetDateTime::parse(time_block, &TIMEFORMAT) else {
            log::error!(
                "DiaryEntry::from_block(): Could not parse timestamp from block: [{time_block}]"
            );
            return Err(Error::CorruptedDiaryFile);
        };
        let title = time_and_title_line.next().unwrap_or_default();
        let text = text_block_parts.next().ok_or(Error::CorruptedDiaryFile)?;

        Ok(DiaryEntry {
            timestamp,
            title: title.to_string(),
            text: text.to_string(),
        })
    }
}

impl Display for DiaryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}\n\n{}",
            self.timestamp
                .format(TIMEFORMAT)
                .expect("This should never fail"),
            TIMETITLESEPARATOR,
            self.title,
            self.text
        )
    }
}
