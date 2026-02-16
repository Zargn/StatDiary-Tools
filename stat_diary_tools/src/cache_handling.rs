use std::{
    collections::HashSet,
    fmt::Display,
    fs::File,
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::{
    data_entry::DataEntry, read_sorted_directory, Overview, ScoreAverages, DATAFILEEXTENSION,
};

#[derive(Debug)]
pub enum RegenCachesError {
    InvalidRoot,
    IoError(io::Error),
    FoundUnknownFile(PathBuf),
    FoundUnknownFolder(PathBuf),
}

impl RegenCachesError {
    pub fn into_code(self) -> i32 {
        match self {
            Self::InvalidRoot => 1,
            Self::IoError(_) => 2,
            Self::FoundUnknownFile(_) => 3,
            Self::FoundUnknownFolder(_) => 4,
        }
    }
}

impl Display for RegenCachesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::InvalidRoot => "Root directory does not exist!".to_string(),
            Self::IoError(err) => format!("IoError: {err}"),
            Self::FoundUnknownFile(path) => format!("Found unknown file: {path:?}"),
            Self::FoundUnknownFolder(path) => format!("Found unknown folder: {path:?}"),
        };
        write!(f, "{}", s)
    }
}

impl From<io::Error> for RegenCachesError {
    fn from(io_err: io::Error) -> Self {
        println!("io_err: {}", io_err);
        Self::IoError(io_err)
    }
}

//

//

/// Regenerates all caches in the provided database.
pub fn regenerate_caches(db_path: &Path) -> Result<(), RegenCachesError> {
    if !db_path.exists() {
        return Err(RegenCachesError::InvalidRoot);
    }

    let data_path = Path::new(db_path).join("data");

    for year_folder in read_sorted_directory(&data_path)? {
        let mut result_writer = BufWriter::new(File::create(year_folder.join("year_cache.txt"))?);
        for month_folder in read_sorted_directory(&year_folder)? {
            if month_folder.is_file() {
                if month_folder != year_folder.join("year_cache.txt") {
                    return Err(RegenCachesError::FoundUnknownFile(month_folder));
                }
                continue;
            }

            let Ok(folder_id) = month_folder
                .file_name()
                .ok_or(RegenCachesError::FoundUnknownFolder(month_folder.clone()))?
                .to_string_lossy()
                .parse::<u8>()
            else {
                return Err(RegenCachesError::FoundUnknownFolder(month_folder));
            };
            if !(1..=12).contains(&folder_id) {
                return Err(RegenCachesError::FoundUnknownFolder(month_folder));
            }

            let avg_month_scores = create_month_cache(&month_folder)?;
            writeln!(
                result_writer,
                "{:?} | {}",
                folder_id,
                avg_month_scores.to_data_str(),
            )?;
        }
        result_writer.flush()?;
    }

    Ok(())
}

//

//

/// Creates a month cache in the provided month folder. Reads all available day items and saves
/// min max and avg m/p scores for each day in separate rows in a month_cache.txt file placed
/// inside the provided month folder.
///
/// If a month_cache.txt file already exists then it gets overwritten.
///
/// Returns the average m and p score for this month.
fn create_month_cache(month_folder: &Path) -> Result<ScoreAverages, RegenCachesError> {
    let mut result_writer = BufWriter::new(File::create(month_folder.join("month_cache.txt"))?);

    let mut month_count = 0;
    let mut month_m_score_sum = 0.0;
    let mut month_p_score_sum = 0.0;

    for file in read_sorted_directory(month_folder)? {
        if file.is_dir() {
            return Err(RegenCachesError::FoundUnknownFolder(file));
        }

        if file == month_folder.join("month_cache.txt") {
            continue;
        }

        if file
            .extension()
            .ok_or(RegenCachesError::FoundUnknownFile(file.clone()))?
            .to_string_lossy()
            != DATAFILEEXTENSION
        {
            return Err(RegenCachesError::FoundUnknownFile(file));
        }

        let mut overview = Overview::default();

        let data_entries = DataEntry::read_from_file(&file)?;
        let entry_count = data_entries.len();

        let mut m_score_sum: f32 = 0.0;
        let mut p_score_sum: f32 = 0.0;
        let mut tags = HashSet::new();
        for data in data_entries {
            m_score_sum += data.mental_score as f32;
            p_score_sum += data.physical_score as f32;

            overview.min_m_score = overview.min_m_score.min(data.mental_score);
            overview.max_m_score = overview.max_m_score.max(data.mental_score);
            overview.min_p_score = overview.min_p_score.min(data.physical_score);
            overview.max_p_score = overview.max_p_score.max(data.physical_score);

            for tag in data.tags {
                tags.insert(tag);
            }
        }

        overview.avg_m_score = m_score_sum / entry_count as f32;
        overview.avg_p_score = p_score_sum / entry_count as f32;

        overview.tags = Vec::from_iter(tags);

        month_m_score_sum += overview.avg_m_score;
        month_p_score_sum += overview.avg_p_score;
        month_count += 1;

        let filename = file
            .file_name()
            .ok_or(RegenCachesError::FoundUnknownFile(file.clone()))?;

        writeln!(
            result_writer,
            "{} | {}",
            filename.to_string_lossy(),
            overview.to_data_str(),
        )?;
    }

    result_writer.flush()?;

    Ok(ScoreAverages {
        avg_mental: month_m_score_sum / month_count as f32,
        avg_physical: month_p_score_sum / month_count as f32,
    })
}
