use std::{
    error::Error,
    ffi::{c_char, CStr},
    fs::File,
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
while if it was translated to raw bytes it would only require 8 bytes.
1 > for mental score
1 > for physical score
6 > for 3 tags (2 bytes per tag to ensure the user doesn't run out of possible tag indexes.)
*/
#[no_mangle]
pub unsafe extern "C" fn TemporaryUpdateDatabase(db_path: *const c_char) -> i32 {
    todo!();
}

fn temporary_update_database(db_path: &str) {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::compress_db_to_image;

    #[test]
    fn test_compress_db_to_image() {
        compress_db_to_image("none", "none");
    }
}
