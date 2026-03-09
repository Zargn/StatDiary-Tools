use std::{
    collections::HashSet,
    ffi::OsStr,
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

use log::{error, warn};

use crate::{
    data_entry::{self, DataFile, ReadDataFileError},
    db_path::DataBasePath,
    utilities::read_sorted_directory,
    DATAFILEEXTENSION,
};

//

//

#[derive(Debug, Clone)]
struct ScoreAvg {
    min: u8,
    max: u8,
    total: u16,
    count: u16,
}

impl Default for ScoreAvg {
    fn default() -> Self {
        ScoreAvg {
            min: u8::MAX,
            max: 0,
            total: 0,
            count: 0,
        }
    }
}

impl ScoreAvg {
    fn add(&mut self, score: u8) {
        self.min = self.min.min(score);
        self.max = self.max.max(score);
        self.total += score as u16;
        self.count += 1;
    }
    fn avg(&self) -> f32 {
        self.total as f32 / self.count as f32
    }
    fn merge(&mut self, other: &ScoreAvg) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.total += other.total;
        self.count += other.count;
    }
}

#[derive(Debug, Default, Clone)]
struct Overview {
    m_score: ScoreAvg,
    p_score: ScoreAvg,
    tags: HashSet<u16>,
}

impl Overview {
    fn to_data_str(&self) -> String {
        let mut data_str = format!(
            "{} {} {} | {} {} {} |",
            self.m_score.min,
            self.m_score.max,
            self.m_score.avg(),
            self.p_score.min,
            self.p_score.max,
            self.p_score.avg()
        );
        for tag in &self.tags {
            data_str.push_str(&format!(" {}", tag));
        }
        data_str
    }

    fn merge(&mut self, other: &Overview) {
        for tag in &other.tags {
            self.tags.insert(*tag);
        }
        self.m_score.merge(&other.m_score);
        self.p_score.merge(&other.p_score);
    }
}

//

//

// TODO ############################################################ TODO
// Better checks to ensure a year folder is actually a valid year folder?

/// Regenerates all caches in the provided database.
pub fn regenerate_caches(db_path: &DataBasePath) -> Result<(), io::Error> {
    log::info!("Regenerating caches...");
    for year_path in read_sorted_directory(&db_path.data())? {
        if year_path.is_file() {
            warn!(
                "Encountered unexpected file in data folder! {:?}",
                year_path
            );
            continue;
        }

        let mut result_writer = BufWriter::new(File::create(year_path.join("year_cache.txt"))?);
        for month_path in read_sorted_directory(&year_path)? {
            let Ok(month_index) = is_month_folder(&month_path) else {
                continue;
            };

            let avg_month_scores = create_month_cache(&month_path)?;
            writeln!(
                result_writer,
                "{} | {}",
                month_index,
                avg_month_scores.to_data_str(),
            )?;
        }
        log::info!("Created year cache: {:?}", year_path.join("year_cache.txt"));
        result_writer.flush()?;
    }

    Ok(())
}

//

//

/// Returns a u8 representing the month index in the name of the provided folder IF it is a folder
/// and has a filename that can be parsed into a valid u8 month index. (To be valid it has to be
/// between 1..=12)
fn is_month_folder(month_path: &Path) -> Result<u8, ()> {
    if month_path.is_file() {
        if month_path.file_name() != Some(OsStr::new("year_cache.txt")) {
            warn!("Ignoring unexpected file in year folder: {:?}", month_path);
        }
        return Err(());
    }

    let Some(folder_name) = month_path.file_name() else {
        warn!("Ignoring folder without name. {:?}", month_path);
        return Err(());
    };

    let Ok(month_index) = folder_name.to_string_lossy().parse::<u8>() else {
        warn!("Ignoring folder with invalid name: {:?}", month_path);
        return Err(());
    };

    if !(1..=12).contains(&month_index) {
        warn!("Ignoring folder with invalid month index! {}", month_index);
        return Err(());
    }
    Ok(month_index)
}

//

//

/// Creates a month cache in the provided month folder. Reads all available day items and saves
/// min max and avg m/p scores for each day in separate rows in a month_cache.txt file placed
/// inside the provided month folder.
///
/// If a month_cache.txt file already exists then it gets overwritten.
///
/// Returns a overview over all days in this month.
fn create_month_cache(month_folder: &Path) -> Result<Overview, io::Error> {
    let mut result_writer = BufWriter::new(File::create(month_folder.join("month_cache.txt"))?);

    let mut month_overview = Overview::default();

    for file in read_sorted_directory(month_folder)? {
        if !DataFile::is_data_file(&file) {
            continue;
        }

        let Some(filename) = file.file_name() else {
            warn!("Skipping data file without name: {:?}", file);
            continue;
        };

        let mut overview = Overview::default();

        let data_file = match DataFile::read_from_file(&file) {
            Ok(data_file) => data_file,
            Err(ReadDataFileError::CorruptedDataFile) => {
                error!("Data file [{:?}] is corrupted! This file will not be represented in the cache!", file);
                continue;
            }
            Err(ReadDataFileError::Io(io_err)) => return Err(io_err),
        };

        for data in data_file.entries() {
            overview.m_score.add(data.mental_score);
            overview.p_score.add(data.physical_score);

            for tag in &data.tags {
                overview.tags.insert(*tag);
            }
        }

        month_overview.merge(&overview);

        writeln!(
            result_writer,
            "{} | {}",
            filename.to_string_lossy(),
            overview.to_data_str(),
        )?;
    }

    log::info!(
        "Created month cache: {:?}",
        month_folder.join("month_cache.txt")
    );
    result_writer.flush()?;

    Ok(month_overview)
}
