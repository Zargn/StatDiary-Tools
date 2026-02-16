use std::{
    collections::{HashMap, HashSet},
    error::Error,
    ffi::{c_char, CStr},
    fmt::{format, Display},
    fs::{self, File},
    io::{self, BufRead, BufWriter, Read, Write},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::data_entry::DataEntry;
mod data_entry;

const DATAFILEEXTENSION: &str = "statdiary";

//

//

#[no_mangle]
pub unsafe extern "C" fn CompressDBToImage(
    db_path: *const c_char,
    result_path: *const c_char,
) -> i32 {
    if db_path.is_null() || result_path.is_null() {
        return -1;
    }
    let db_path = unsafe { CStr::from_ptr(db_path).to_string_lossy() };
    let result_path = unsafe { CStr::from_ptr(result_path).to_string_lossy() };

    compress_db_to_image(&db_path, &result_path);

    1
}

//

//

fn compress_db_to_image(db_path: &str, result_path: &str) -> Result<(), Box<dyn Error>> {
    println!("Using path: {}", db_path);

    if !Path::new(db_path).is_dir() {
        return Err("".into());
    }

    let zipfile = File::create(result_path)?;
    let walkdir = WalkDir::new(db_path);
    let mut zip = zip::ZipWriter::new(zipfile);
    Ok(())
}

//

//

#[no_mangle]
pub unsafe extern "C" fn RegenerateCaches(db_path: *const c_char) -> i32 {
    if db_path.is_null() {
        return -1;
    }
    let db_path = unsafe { CStr::from_ptr(db_path) };

    let Ok(path) = db_path.to_str() else {
        return -2;
    };

    if let Err(error) = regenerate_caches(Path::new(path)) {
        println!("Error occured!\n{:?}", error);
        return error.into_code();
    }

    0
}

//

//

/*

Task:
Iterate through the entire database generating new caches for each day, month and year.

Should be done as a depth-first search. Start with the first day recorded, then the day after that, and so on.

Existing caches are to be overwritten by the new.

Caches are to be saved in the following locations:
All daily averages for each month should be saved in a month_cache.txt file inside that months folder.
All months averages for each year should be saved in a year_cache.txt file inside that years folder.

Cache format:
First byte:
Average mental score for this period.
Second byte:
Average physical score for this period.
Remaining bytes:
Every two bytes represent a u16 tag id.

??? Should we add a third byte for a user-defined score? ???
Doesn't need to actually contain anything yet, but we could reserve the third byte for it
just like we do for the mental and physical score.
Although if we want to add that in the future it shouldn't be very difficult to modify this function
at that time instead. Since it is made to regenerate all the caches, meaning the old gets deleted.

Update:
Current cache format is this:
year_cache: "{month_number} | {avg_mental_score} | {avg_physical_score}"
month_cache:
"{day_number} | {min_mental_score} {max_mental_score} {avg_mental_score} | {min_physical_score} {max_physical_score} {avg_physical_score} | {tag} ..."

*/

#[derive(Debug)]
enum RegenCachesError {
    InvalidRoot,
    IoError(io::Error),
    InvalidMonthFolder,
    FoundUnknownFile(PathBuf),
    FoundUnknownFolder(PathBuf),
}

impl RegenCachesError {
    fn into_code(self) -> i32 {
        match self {
            Self::InvalidRoot => 1,
            Self::IoError(_) => 2,
            Self::InvalidMonthFolder => 3,
            Self::FoundUnknownFile(_) => 4,
            Self::FoundUnknownFolder(_) => 5,
        }
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
fn regenerate_caches(db_path: &Path) -> Result<(), RegenCachesError> {
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
                .ok_or(RegenCachesError::InvalidMonthFolder)?
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

/// Creates a sorted vec with paths visiting all items in the provided directory.
fn read_sorted_directory(directory_path: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut files = fs::read_dir(directory_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    files.sort();
    Ok(files)
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
    /*
    let mut files = fs::read_dir(month_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    files.sort(); */

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

        let data_entries = get_entries_from_file(&file)?;
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
        /*
        println!(
            "Overview: \n{} | {}\n",
            file.file_name().unwrap().to_str().unwrap(),
            overview.to_data_str()
        );*/
    }

    result_writer.flush()?;

    /*
    println!(
        "Month avg:\nMental_score: {}\nPhysical_Score: {}",
        month_m_score_sum / month_count as f32,
        month_p_score_sum / month_count as f32
    );*/

    Ok(ScoreAverages {
        avg_mental: month_m_score_sum / month_count as f32,
        avg_physical: month_p_score_sum / month_count as f32,
    })
}

#[derive(Debug, Default)]
struct Overview {
    min_m_score: u8,
    max_m_score: u8,
    avg_m_score: f32,
    min_p_score: u8,
    max_p_score: u8,
    avg_p_score: f32,
    tags: Vec<u16>,
}

impl Overview {
    fn to_data_str(&self) -> String {
        let mut data_str = format!(
            "{} {} {} | {} {} {} |",
            self.min_m_score,
            self.max_m_score,
            self.avg_m_score,
            self.min_p_score,
            self.max_p_score,
            self.avg_p_score
        );
        for tag in &self.tags {
            data_str.push_str(&format!(" {}", tag));
        }
        data_str
    }
}

struct ScoreAverages {
    avg_mental: f32,
    avg_physical: f32,
}

impl ScoreAverages {
    fn to_data_str(&self) -> String {
        format!("{} | {}", self.avg_mental, self.avg_physical)
    }
}

//

//

/*
When merging two tags check the general averages and make sure to merge the less used tag
into the more used one. This will minimize the amount of edits to the data file.

Make sure to add a check when adding tags to fill out any potential empty space left by a
merge.
*/
#[no_mangle]
pub unsafe extern "C" fn MergeTags(db_path: *const c_char) -> i32 {
    todo!();
}

//

//

/*
Fairly simple function. Only needs to edit the tags document.
*/
#[no_mangle]
pub unsafe extern "C" fn RenameTag(db_path: *const c_char) -> i32 {
    todo!();
}

//

//

/*
This function is meant to eventually be used to update any old database to use a newer format.
No functionality should be implemented yet as it is not the current priority.

Some potential plans for changes that would require this is:
Better tag storage. Keep the string representations in a separate file and save the tag index
in each entry instead of the full string.

Later update could be to reduce the storage needed even further.
If tags are stored as numbers instead of strings then we are technically only storing numbers
in the entries. Meaning we could avoid using a text file entirely and just go with raw bytes.
1st byte would be a u8 representing the mental score, 2nd byte would be another u8 this time
representing the physical score. After that we could have any number of double bytes
representing a u16 tag id. Then the end of that entry is marked with a double byte u16 of value
u16::MAX.
No strings or text needed. This way the only thing stored in the data files would be the actual
data and one u16 used as a marker for each entry. Much better than storing the data in a human
readable format where a lot of space is taken by "," and "|".
Although for this we should probably make both scores integers instead of floats.

For example the following row in a txt document takes up 41 bytes. (1 byte per char)
|13:00|85,8|81,5|Lunch Geoguessr Youtube|
while if it was translated to raw bytes it would only require 11 bytes.
1 > for hour of day
1 > for mental score
1 > for physical score
6 > for 3 tags (2 bytes per tag to ensure the user doesn't run out of possible tag indexes.)
2 > for end marker u16::MAX

Make sure to call RegenerateCaches after this to create all the caches in the correct folders.
*/
#[no_mangle]
pub unsafe extern "C" fn TemporaryUpdateDatabase(db_path: *const c_char) -> i32 {
    todo!();
}

//

//

pub fn temporary_update_database(db_path: &str) -> Result<(), Box<dyn Error>> {
    let mut tags: HashMap<String, u16> = HashMap::new();

    // Iterate through data_base/data/
    // Call transform_data_file on each of them.

    for path in WalkDir::new(format!("{}/data", db_path)) {
        let path = path?;
        let path = path.path();
        if path.is_file() {
            println!("{:?}", path);
            transform_data_file(path.to_str().unwrap(), &mut tags)?;
        }
    }

    let Ok(tags_file) = File::create(format!("{}/tags.txt", db_path)) else {
        // Could not create a new file! It might already exist, or there is some other issue.
        return Err("Could not create a new file!".into());
    };

    let mut tags_writer = BufWriter::new(tags_file);

    for (k, v) in tags {
        writeln!(tags_writer, "{} {}", v, k)?;
    }

    tags_writer.flush()?;

    Ok(())
}

//

//

pub fn read_lines<P>(path: P) -> io::Result<impl Iterator<Item = String>>
where
    P: AsRef<Path>,
{
    Ok(io::BufReader::new(File::open(path)?)
        .lines()
        .map_while(Result::ok))
}

//

//

pub fn transform_data_file(
    file_path: &str,
    tags: &mut HashMap<String, u16>,
) -> Result<(), Box<dyn Error>> {
    //  split file name at '-'
    //      get the index of the day of week named in the result part 2.
    //      create a result_file with name: result part 1 + '-' + day index
    //
    //  For each row in file
    //      split row at '|'
    //          part1 is the hour
    //          part2 is the mental score
    //          part3 is the physical score
    //          part4 are the tags
    //      split part1 at ':'
    //          parse a u8 from the first result part
    //          write u8 to result_file
    //      for part2 and part3
    //          split at ','
    //              parse a u8 from the first result part
    //              write u8 to result_file
    //      split part4 at ' '
    //          for each result part
    //              if result part exists in tags
    //                  write tag u16 to result_file
    //              else
    //                  write tags.len() to result file
    //                  add result part to tags with value tags.len()
    //      add u16::MAX to result file
    //
    //  save result file
    //  delete original file

    let Ok(lines) = read_lines(file_path) else {
        // File does not exist
        // Exit early with error
        return Err("File does not exist!".into());
    };

    let (day_of_month, day_of_week) = {
        let mut path = Path::new(file_path).file_stem().unwrap().to_os_string();
        let mut parts = path
            .to_str()
            .expect("The filename should always be valid a valid string, right?")
            .split('-');
        (
            parts.next().unwrap().to_string(),
            day_of_week(parts.next().unwrap()),
        )
    };

    let path = Path::new(file_path).parent().unwrap().to_str().unwrap();

    let Ok(result_file) = File::create(format!(
        "{}/{}-{}.{}",
        path, day_of_month, day_of_week, DATAFILEEXTENSION
    )) else {
        // Could not create a new file! It might already exist, or there is some other issue.
        return Err("Could not create a new file!".into());
    };

    let mut result_writer = BufWriter::new(result_file);

    for line in lines {
        let mut row_parts = line.split('|');
        row_parts.next();

        println!("Reading line: {}", line);

        // Hour
        parse_and_write(row_parts.next().unwrap(), ':', &mut result_writer)?;

        for _ in 0..2 {
            // Mental and physical score.
            parse_and_write(row_parts.next().unwrap(), ',', &mut result_writer)?;
        }

        // Tags
        for tag_str in row_parts.next().unwrap().split(' ').map(|s| s.to_string()) {
            let tags_len = tags.len() as u16;
            let id = tags.entry(tag_str).or_insert(tags_len);
            result_writer.write_all(&id.to_be_bytes())?;
        }

        // End of entry marker
        result_writer.write_all(&u16::MAX.to_be_bytes())?;
    }

    result_writer.flush()?;
    std::fs::remove_file(file_path)?;

    Ok(())
}

//

//

/// Reads all entries in the provided file and returns a list of assembled DataEntry structs
pub fn get_entries_from_file(file_path: &Path) -> Result<Vec<DataEntry>, io::Error> {
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
pub fn temp_display_entry(entry: DataEntry, tags: &HashMap<u16, String>) {
    print!(
        "\n {}:00 | {} | {} | ",
        entry.hour, entry.mental_score, entry.physical_score
    );
    for tag_id in entry.tags {
        if let Some(tag) = tags.get(&tag_id) {
            print!("{} ", tag);
        } else {
            print!("UNKNOWN_ID ");
        }
    }
    println!();
}

//

//

fn parse_and_write(
    data_str: &str,
    split: char,
    writer: &mut impl io::Write,
) -> Result<(), Box<dyn Error>> {
    writer.write_all(&[data_str.split(split).next().unwrap().parse::<u8>()?])?;
    Ok(())
}

//

//

fn day_of_week(day_name: &str) -> u8 {
    match day_name {
        "Monday" => 0,
        "Tuesday" => 1,
        "Wednesday" => 2,
        "Thursday" => 3,
        "Friday" => 4,
        "Saturday" => 5,
        "Sunday" => 6,
        _ => u8::MAX,
    }
}

//

//

#[cfg(test)]
mod tests {
    use super::compress_db_to_image;

    #[test]
    fn test_compress_db_to_image() {
        compress_db_to_image("none", "none");
    }
}
