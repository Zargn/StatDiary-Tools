use std::{
    collections::HashMap,
    error::Error,
    ffi::{c_char, CStr},
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufWriter, Read, Write},
    path::Path,
};
use walkdir::WalkDir;

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

#[no_mangle]
pub unsafe extern "C" fn RegenerateCaches(db_path: *const c_char) -> i32 {
    todo!();
}

/*

Task:
Iterate through the entire database generating new caches for each day, month and year.

Should be done as a depth-first search. Start with the first day recorded, then the day after that, and so on.

Existing caches are to be overwritten by the new.

Caches are to be saved in the following locations:
All daily averages for each month should be saved in a month_cache.txt file inside that months folder.
All months averages for each year should be saved in a year_cache.txt file inside that years folder.


*/
fn regenerate_caches(db_path: &str) -> i32 {
    todo!();
}

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

/*
Fairly simple function. Only needs to edit the tags document.
*/
#[no_mangle]
pub unsafe extern "C" fn RenameTag(db_path: *const c_char) -> i32 {
    todo!();
}

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

fn temporary_update_database(db_path: &str) {
    let tags: HashMap<String, u16> = HashMap::new();

    // Iterate through data_base/data/
    // Call transform_data_file on each of them.

    todo!();
}

pub fn read_lines<P>(path: P) -> io::Result<impl Iterator<Item = String>>
where
    P: AsRef<Path>,
{
    Ok(io::BufReader::new(File::open(path)?)
        .lines()
        .map_while(Result::ok))
}

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

    let Ok(result_file) = File::create(format!("{}-{}", day_of_month, day_of_week)) else {
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

    Ok(())
}

pub fn temp_read_data_file(
    file_path: &str,
    tags: &HashMap<u16, String>,
) -> Result<(), Box<dyn Error>> {
    let bytes: Vec<u8> = io::BufReader::new(File::open(file_path)?)
        .bytes()
        .map_while(Result::ok)
        .collect();

    let mut i = 0;

    while i < bytes.len() {
        let hour = bytes[i];
        let m_score = bytes[i + 1];
        let p_score = bytes[i + 2];
        print!("\n {}:00 | {} | {} | ", hour, m_score, p_score);

        i += 3;
        loop {
            let id = ((bytes[i] as u16) << 8) | bytes[i + 1] as u16;
            if id == u16::MAX {
                i += 2;
                break;
            }
            i += 2;

            if let Some(tag) = tags.get(&id) {
                print!("{} ", tag);
            } else {
                print!("UNKNOWN_ID ");
            }
        }
    }

    Ok(())
}

fn parse_and_write(
    data_str: &str,
    split: char,
    writer: &mut impl io::Write,
) -> Result<(), Box<dyn Error>> {
    writer.write_all(&[data_str.split(split).next().unwrap().parse::<u8>()?])?;
    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::compress_db_to_image;

    #[test]
    fn test_compress_db_to_image() {
        compress_db_to_image("none", "none");
    }
}
