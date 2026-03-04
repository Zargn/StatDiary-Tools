use std::{
    ffi::{c_char, CStr, OsStr},
    path::Path,
};

use walkdir::WalkDir;

use crate::{
    backup::{compress_to_image, load_image},
    cache_handling::regenerate_caches,
    data_entry::DataFile,
    db_path::{DataBasePath, PtrToDBPathError},
    db_status::{ActiveTask, DBStatus, DBStatusError},
    stat_sums::regenerate_tag_sums,
    tags::{TagList, TagsError},
    update_database::temporary_update_database,
    utilities::try_ptr_to_string,
};
mod backup;
mod cache_handling;
mod data_entry;
mod db_path;
mod db_status;
mod stat_diary_error;
mod stat_sums;
mod tags;
mod update_database;

const DATAFILEEXTENSION: &str = "statdiary";

//

//

#[no_mangle]
pub unsafe extern "C" fn CompressDBToImage(
    db_path_ptr: *const c_char,
    result_path: *const c_char,
) -> i32 {
    if result_path.is_null() {
        return -1;
    }
    let result_path = unsafe { CStr::from_ptr(result_path).to_string_lossy() };

    let db_path = match DataBasePath::try_ptr_to_data_base_path(db_path_ptr) {
        Ok(db_path) => db_path,
        Err(PtrToDBPathError::NullPtr) => return -1,
        Err(PtrToDBPathError::InvalidUTF8) => return -2,
        Err(PtrToDBPathError::DataBasePath(dbp_error)) => match dbp_error {
            db_path::DataBasePathError::DoesNotExist => return -3,
            db_path::DataBasePathError::IsNotDataBase => return -4,
        },
    };

    if let Err(error) = compress_to_image(&db_path, Path::new(&result_path.to_string())) {
        println!("Error occured! [{:?}]", error);
        return -2;
    }

    //compress_db_to_image(&db_path, &result_path);

    1
}

//

//

#[no_mangle]
pub unsafe extern "C" fn ExtractDBFromImage(
    result_db_path: *const c_char,
    db_image_path: *const c_char,
) -> i32 {
    if result_db_path.is_null() || db_image_path.is_null() {
        return -1;
    }
    let result_db_path = unsafe { CStr::from_ptr(result_db_path).to_string_lossy() };
    let db_image_path = unsafe { CStr::from_ptr(db_image_path).to_string_lossy() };

    if let Err(error) = load_image(
        Path::new(&result_db_path.to_string()),
        Path::new(&db_image_path.to_string()),
    ) {
        println!("Error occured! [{:?}]", error);
        return -2;
    }

    //compress_db_to_image(&db_path, &result_path);

    1
}

//

//

#[no_mangle]
pub unsafe extern "C" fn RegenerateCaches(db_path: *const c_char) -> i32 {
    if db_path.is_null() {
        return -1;
    }
    let db_path = unsafe { CStr::from_ptr(db_path) };

    let Ok(path_str) = db_path.to_str() else {
        return -2;
    };

    let Ok(db_path) = DataBasePath::new(Path::new(path_str).to_path_buf()) else {
        return -3;
    };

    let Ok(db_status) = DBStatus::activate(&db_path, ActiveTask::RegenerateCaches) else {
        println!("Database is busy! Aborting...");
        return -3;
    };

    if let Err(error) = regenerate_caches(&db_path) {
        println!("Error occured!\n{:?}", error);

        db_status.deactivate();
        return -3;
        //return error.into_code();
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

    let Ok(path_str) = db_path.to_str() else {
        return -2;
    };

    let Ok(db_path) = DataBasePath::new(Path::new(path_str).to_path_buf()) else {
        return -3;
    };

    let activate_error = match DBStatus::activate(&db_path, ActiveTask::None) {
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
            if let Err(error) = regenerate_caches(&db_path) {
                println!("Error occured!\n{:?}", error);

                db_status.deactivate();
                return -3;
                //return error.into_code();
            }
        }
        ActiveTask::RegenerateTagSums => {
            if let Err(error) = regenerate_tag_sums(&db_path) {
                println!("Error occured!\n{:?}", error);

                db_status.deactivate();
                return -3;
                //return error.into_code();
            }
        }
        ActiveTask::MergeTags(tag_1, tag_2) => {
            if let Err(error) = merge_tags(&db_path, tag_1, tag_2) {
                println!("Error occured!\n{:?}", error);

                db_status.deactivate();
                return -3;
                //return error.into_code();
            }
        }
        ActiveTask::RenameTag(old_tag, new_tag) => {
            if let Err(error) = rename_tag(Path::new(path_str), old_tag, new_tag) {
                println!("Error occured!\n{:?}", error);

                db_status.deactivate();
                return -3;
                //return error.into_code();
            }
        }
        ActiveTask::None => {}
    }

    db_status.deactivate();

    0
}

//

//

#[no_mangle]
pub unsafe extern "C" fn MergeTags(db_path: *const c_char, tag1: u16, tag2: u16) -> i32 {
    let Ok(path) = try_ptr_to_string(db_path) else {
        return -1;
    };

    let Ok(path_str) = db_path.to_str() else {
        return -2;
    };

    let Ok(db_path) = DataBasePath::new(Path::new(path_str).to_path_buf()) else {
        return -3;
    };

    /*
    let Ok(db_status) =
        DBStatus::activate(db_path.to_path_buf(), ActiveTask::MergeTags(tag1, tag2))
    else {
        println!("Database is busy! Aborting...");
        return -3;
    }; */

    if let Err(error) = merge_tags_wrapper(db_path, tag1, tag2) {
        println!("Error occured!\n{:?}", error);

        //db_status.deactivate();
        return -3;
        //return error.into_code();
    }

    //db_status.deactivate();

    0
}

//

//

fn merge_tags_wrapper(db_path: &DataBasePath, tag_1: u16, tag_2: u16) -> Result<(), TagsError> {
    let Ok(db_status) = DBStatus::activate(db_path, ActiveTask::MergeTags(tag_1, tag_2)) else {
        println!("Database is busy! Aborting...");
        return Err(TagsError::TagAlreadyExists);
        //return Err(DBError::DataBaseBusy);
    };

    println!("Merging tags");
    if let Err(error) = merge_tags(db_path, tag_1, tag_2) {
        //println!("Error occured!\n{:?}", error);

        db_status.deactivate();
        return Err(error);
    } // */
    db_status.deactivate();

    println!("Regenerating Tag sums");
    if let Err(error) = regenerate_tag_sums(db_path) {
        println!("Error occured! \n{:?}", error);
        //db_status.deactivate();
    }

    println!("Regenerating Caches");
    if let Err(error) = regenerate_caches(db_path) {
        println!("Error occured!\n{:?}", error);

        //db_status.deactivate();
    }

    Ok(())
}

//

//

/*
Make sure to add a check when adding tags to fill out any potential empty space left by a
merge.
*/
fn merge_tags(db_path: &DataBasePath, tag_1: u16, tag_2: u16) -> Result<(), TagsError> {
    let mut tags = TagList::from_file(db_path)?;
    tags.merge_tags(tag_1, tag_2)?;

    for path in WalkDir::new(db_path.data()) {
        let path = path.unwrap();
        let filepath = path.path();

        if !filepath.is_file() {
            continue;
        }

        if filepath.extension() != Some(OsStr::new("statdiary")) {
            continue;
        }

        let mut data_file = DataFile::read_from_file(filepath.to_path_buf())?;

        data_file.merge_tags(tag_1, tag_2);

        data_file.save()?;
    }

    //tags.save()?;

    /*
    Get tag ids for both tag1 and tag2.

    iterate through all .statdiary files in the data directory.
        for each data_entry in file
            if tag1 could be removed from data_entry.tags
                ensure tag2 is exists in data_entry.tags.
                (Since we are merging two tags we don't want to store two duplicate tags in a single entry)


    call regenerate_tag_sums function.


    call regenerate_caches function.
    */

    Ok(())
}

//

//

#[no_mangle]
pub unsafe extern "C" fn RegenerateTagSums(db_path: *const c_char) -> i32 {
    let Ok(path) = try_ptr_to_string(db_path) else {
        return -1;
    };

    let db_path = Path::new(&path);

    let Ok(db_status) = DBStatus::activate(db_path.to_path_buf(), ActiveTask::RegenerateTagSums)
    else {
        println!("Database is busy! Aborting...");
        return -3;
    };

    if let Err(error) = regenerate_tag_sums(db_path) {
        println!("Error occured! \n{:?}", error);
        db_status.deactivate();
        return -3;
        //return error.into_code();
    }

    db_status.deactivate();

    0
}

//

//

#[no_mangle]
pub unsafe extern "C" fn RenameTag(
    db_path: *const c_char,
    old_tag: *const c_char,
    new_tag: *const c_char,
) -> i32 {
    let Ok(path) = try_ptr_to_string(db_path) else {
        return -1;
    };
    let Ok(old_tag) = try_ptr_to_string(old_tag) else {
        return -2;
    };
    let Ok(new_tag) = try_ptr_to_string(new_tag) else {
        return -3;
    };

    if let Err(error) = rename_tag(Path::new(&path), old_tag.to_string(), new_tag.to_string()) {
        println!("Error occured!\n{:?}", error);
        return -3;
        //return error.into_code();
    }

    0
}

//

//

fn rename_tag(db_path: &Path, old_tag: String, new_tag: String) -> Result<(), TagsError> {
    let mut tags = TagList::from_file(db_path)?;
    tags.rename_tag(old_tag, new_tag)?;
    tags.save()
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
    let Ok(path) = try_ptr_to_string(db_path) else {
        return -1;
    };

    if let Err(error) = temporary_update_database(&path) {
        println!("Error occured!\n{:?}", error);
        //return error.into_code();
    }

    // TODO: Regenerate caches too!

    0
}

//

//

mod utilities {
    use std::{
        ffi::{c_char, CStr},
        fs::{self, File},
        io::{self, BufRead},
        path::{Path, PathBuf},
    };

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

    pub unsafe fn try_ptr_to_string(ptr: *const c_char) -> Result<String, i32> {
        if ptr.is_null() {
            return Err(-1);
        }
        let cstr = unsafe { CStr::from_ptr(ptr) };
        let Ok(str) = cstr.to_str() else {
            return Err(-2);
        };
        Ok(str.to_string())
    }

    //

    //

    /// Creates a sorted vec with paths visiting all items in the provided directory.
    pub fn read_sorted_directory(directory_path: &Path) -> Result<Vec<PathBuf>, io::Error> {
        let mut files = fs::read_dir(directory_path)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        files.sort();
        Ok(files)
    }
}

#[cfg(test)]
mod tests {}
