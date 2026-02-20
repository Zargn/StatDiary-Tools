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
use zip::unstable::write;

use crate::{
    cache_handling::{regenerate_caches, RegenCachesError},
    data_entry::DataEntry,
    db_status::{ActiveTask, DBStatus, DBStatusError},
    tags::{DBError, TagList},
};
mod cache_handling;
mod data_entry;
mod db_status;
mod tags;

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

    let db_path = Path::new(path);

    let Ok(db_status) = DBStatus::activate(db_path.to_path_buf(), ActiveTask::RegenerateCaches)
    else {
        println!("Database is busy! Aborting...");
        return -3;
    };

    if let Err(error) = regenerate_caches(Path::new(path)) {
        println!("Error occured!\n{:?}", error);

        db_status.deactivate();
        return error.into_code();
    }

    db_status.deactivate();

    0
}

//

//

#[no_mangle]
pub unsafe extern "C" fn ResumeTask(db_path: *const c_char) -> i32 {
    if db_path.is_null() {
        return -1;
    }
    let db_path = unsafe { CStr::from_ptr(db_path) };

    let Ok(path) = db_path.to_str() else {
        return -2;
    };

    let db_path = Path::new(path);
    let activate_error = match DBStatus::activate(db_path.to_path_buf(), ActiveTask::None) {
        Ok(db_status) => {
            db_status.deactivate();
            return 0;
        }
        Err(db_error) => db_error,
    };

    let DBStatusError::DataBaseBusy(active_task, db_status) = activate_error else {
        return -3;
    };

    match active_task {
        ActiveTask::RegenerateCaches => {
            if let Err(error) = regenerate_caches(Path::new(path)) {
                println!("Error occured!\n{:?}", error);

                db_status.deactivate();
                return error.into_code();
            }
        }
        ActiveTask::MergeTags(tag_1, tag_2) => {}
        ActiveTask::RenameTag(old_tag, new_tag) => {
            if let Err(error) = rename_tag(Path::new(path), old_tag, new_tag) {
                println!("Error occured!\n{:?}", error);

                db_status.deactivate();
                return error.into_code();
            }
        }
        ActiveTask::None => {}
    }

    db_status.deactivate();

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

//

//

/*
/// Returns a hashmap representing all the tag ids and tag strings that are available in the
/// provided database.
///
/// Will return a DBError::CorruptedTagsFile if any fault is found with the tags file, and a
/// DBError::IoError if the tags file is missing.
fn get_tag_map(db_path: &Path) -> Result<(HashMap<u16, String>, HashMap<String, u16>, DBError> {
    let filepath = db_path.join("tags.txt");

    let mut tags = HashMap::new();
    for line in read_lines(filepath)? {
        let mut parts = line.split(' ');
        let (id, tag) = (
            parts
                .next()
                .ok_or(DBError::CorruptedTagsFile(line.clone()))?
                .parse::<u16>()
                .map_err(|_| DBError::CorruptedTagsFile(line.clone()))?,
            parts
                .next()
                .ok_or(DBError::CorruptedTagsFile(line.clone()))?,
        );

        if tags.insert(id, tag.to_string()).is_some() {
            return Err(DBError::CorruptedTagsFile(
                "Duplicate tag ids found in tags file!".to_string(),
            ));
        }
        todo!();
    }
    Ok(tags)
}
*/

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
pub unsafe extern "C" fn MergeTags(
    db_path_str: *const c_char,
    tag1: *const c_char,
    tag2: *const c_char,
) -> i32 {
    if db_path_str.is_null() {
        return -1;
    }
    if tag1.is_null() {
        return -2;
    }
    if tag2.is_null() {
        return -3;
    }

    let Ok(path_str) = unsafe { CStr::from_ptr(db_path_str) }.to_str() else {
        return -4;
    };
    let Ok(tag1) = unsafe { CStr::from_ptr(tag1) }.to_str() else {
        return -5;
    };

    let Ok(tag2) = unsafe { CStr::from_ptr(tag2) }.to_str() else {
        return -6;
    };

    let db_path = Path::new(path_str);

    let Ok(db_status) = DBStatus::activate(db_path.to_path_buf(), ActiveTask::RegenerateCaches)
    else {
        println!("Database is busy! Aborting...");
        return -3;
    };

    if let Err(error) = merge_tags(db_path, tag1, tag2) {
        println!("Error occured!\n{:?}", error);

        db_status.deactivate();
        return error.into_code();
    }

    db_status.deactivate();
    todo!();
}

fn merge_tags(db_path: &Path, tag1: &str, tag2: &str) -> Result<(), DBError> {
    let tags = TagList::from_file(db_path)?;

    /*
    Get tag ids for both tag1 and tag2.

    iterate through all .statdiary files in the data directory.
        for each data_entry in file
            if tag1 could be removed from data_entry.tags
                ensure tag2 is exists in data_entry.tags.
                (Since we are merging two tags we don't want to store two duplicate tags in a single entry)

    {
        Iterate through all .stat_avg files.
            read contents into a hashmap of ids and occurnaces.
            if tag1 does not exist in the map and tag2 does not exist in the map
                skip this file
            get occurnaces of tag1 from hashmap [tag_occurances]
                if tag1 does not exist then set [tag_occurances] to 0.
            get entry or default of tag2 from hashmap and add [tag_occurances] to that value.
    }
    OR
    {
        call regenerate_averages function.
    }


    call regenerate_caches function.
    */

    todo!();
}

//

//

#[no_mangle]
pub unsafe extern "C" fn RenameTag(
    db_path: *const c_char,
    old_tag: *const c_char,
    new_tag: *const c_char,
) -> i32 {
    if db_path.is_null() {
        return -1;
    }
    if old_tag.is_null() {
        return -2;
    }
    if new_tag.is_null() {
        return -3;
    }

    let Ok(path_str) = unsafe { CStr::from_ptr(db_path) }.to_str() else {
        return -4;
    };
    let Ok(old_tag) = unsafe { CStr::from_ptr(old_tag) }.to_str() else {
        return -5;
    };
    let Ok(new_tag) = unsafe { CStr::from_ptr(new_tag) }.to_str() else {
        return -6;
    };

    if let Err(error) = rename_tag(
        Path::new(path_str),
        old_tag.to_string(),
        new_tag.to_string(),
    ) {
        println!("Error occured!\n{:?}", error);

        return error.into_code();
    }

    0
}

fn rename_tag(db_path: &Path, old_tag: String, new_tag: String) -> Result<(), DBError> {
    let mut tags = TagList::from_file(db_path)?;
    tags.rename_tag(old_tag, new_tag)?;
    tags.save_to_file(db_path)
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
    if db_path.is_null() {
        return -1;
    }
    let db_path = unsafe { CStr::from_ptr(db_path) };

    let Ok(path) = db_path.to_str() else {
        return -2;
    };

    if let Err(error) = temporary_update_database(path) {
        println!("Error occured!\n{:?}", error);
        //return error.into_code();
    }

    0
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
            if let Some(e) = path.extension() {
                if e.to_str() != Some("txt") {
                    continue;
                }
            }
            println!("{:?}", path);
            transform_data_file(path.to_str().unwrap(), &mut tags)?;
        }
    }

    let Ok(tags_file) = File::create(format!("{}/tags.txt", db_path)) else {
        // Could not create a new file! It might already exist, or there is some other issue.
        return Err("Could not create a new file!".into());
    };

    let mut tags_writer = BufWriter::new(tags_file);

    for (k, v) in &tags {
        writeln!(tags_writer, "{} {}", v, k)?;
    }

    tags_writer.flush()?;
    update_averages(Path::new(db_path))?;

    Ok(())
}

pub fn update_averages(db_path: &Path) -> Result<(), Box<dyn Error>> {
    let taglist = TagList::from_file(db_path).unwrap();
    for path in WalkDir::new(db_path.join("averages")) {
        let path = path?;
        if path.path().is_file() {
            println!("{:?}", path);
            transform_average_file(&taglist, path.path())?;
        }
    }

    Ok(())
}

fn transform_average_file(taglist: &TagList, file_path: &Path) -> Result<(), Box<dyn Error>> {
    let new_file_path = file_path.with_extension("stat_avg");
    let mut result_writer = BufWriter::new(File::create(new_file_path)?);
    //println!("New avg file: {:?}", new_file_path);

    for line in read_lines(file_path)? {
        let mut parts = line.split(' ');
        let (occurances, tag) = (parts.next().unwrap(), parts.next().unwrap());
        println!("{} | {}", occurances, tag);
        writeln!(
            result_writer,
            "{} {}",
            occurances,
            taglist.get_id(tag).unwrap()
        )?;
        //writeln!(result_writer, "{} {}")
        //todo!();
    }

    result_writer.flush()?;

    std::fs::remove_file(file_path)?;

    Ok(())
    //todo!();
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
