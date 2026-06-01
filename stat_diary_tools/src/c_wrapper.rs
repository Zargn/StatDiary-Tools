use std::{
    ffi::{c_char, CStr},
    path::{Path, PathBuf},
};

use crate::{
    data_base::{self, DataBase},
    data_entry::DataEntry,
};

/// fn InitLogger(`logfile_path_ptr`);
///
/// WARNING: Function is incomplete. Currently logs to std::out meaning the logfile_path_ptr
/// isn't actually used yet.
/// Sets the logfile path to `logfile_path_ptr`.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
#[no_mangle]
pub unsafe extern "C" fn InitLogger(logfile_path_ptr: *const c_char) -> i32 {
    let logfile_path = match try_ptr_to_string(logfile_path_ptr) {
        Ok(str) => str,
        Err(ec) => return ec,
    };

    let result_code = match DataBase::init_logger(PathBuf::from(logfile_path)) {
        Ok(_) => 0,
        Err(error) => {
            log::error!("InitLogger error occured: {error:?}");
            error.code()
        }
    };

    log::logger().flush();
    result_code
}

//

//

/// fn CompressDBToImage(`db_path_ptr`, `result_path_ptr`);
///
/// Compresses the database at `db_path_ptr` into a image stored at `result_path_ptr`.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
#[no_mangle]
pub unsafe extern "C" fn CompressDBToImage(
    db_path_ptr: *const c_char,
    result_path_ptr: *const c_char,
) -> i32 {
    let data_base = match try_get_db(db_path_ptr) {
        Ok(db) => db,
        Err(ec) => return ec,
    };

    let result_path = match try_ptr_to_string(result_path_ptr) {
        Ok(str) => str,
        Err(ec) => return ec,
    };

    let result_code = match data_base.compress_to_image(Path::new(&result_path)) {
        Ok(_) => 0,
        Err(error) => {
            log::error!("CompressDBToImage error occured: {error:?}");
            error.code()
        }
    };

    log::logger().flush();
    result_code
}

//

//

/// fn ExtractDBFromImage(`db_image_path_ptr`, `db_path_ptr`);
///
/// Attempts to extract a `DataBase` from the provided `db_image_path_ptr` into `db_path_ptr`.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
#[no_mangle]
pub unsafe extern "C" fn ExtractDBFromImage(
    db_image_path_ptr: *const c_char,
    db_path_ptr: *const c_char,
) -> i32 {
    let db_path = match try_ptr_to_string(db_path_ptr) {
        Ok(str) => str,
        Err(ec) => return ec,
    };

    let db_image_path = match try_ptr_to_string(db_image_path_ptr) {
        Ok(str) => str,
        Err(ec) => return ec,
    };

    let result_code = match DataBase::load_from_image(
        Path::new(&db_image_path),
        Path::new(&db_path).to_path_buf(),
    ) {
        Ok(_) => 0,
        Err(error) => {
            log::error!("ExtractDBFromImage error occured: {error:?}");
            error.code()
        }
    };

    log::logger().flush();
    result_code
}

//

//

/// fn RegenerateCaches(`db_path_ptr`);
///
/// Attempts to regenerate the cache files in the `DataBase` at the provided `db_path_ptr`.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
#[no_mangle]
pub unsafe extern "C" fn RegenerateCaches(db_path_ptr: *const c_char) -> i32 {
    let data_base = match try_get_db(db_path_ptr) {
        Ok(db) => db,
        Err(ec) => return ec,
    };

    let result_code = match data_base.regen_caches() {
        Ok(_) => 0,
        Err(error) => {
            log::error!("RegenerateCaches error occured: {error:?}");
            error.code()
        }
    };

    log::logger().flush();
    result_code
}

//

//

/// fn ResumeTask(`db_path_ptr`);
///
/// Attempts to resume any non-finished tasks in the `DataBase` at the provided `db_path_ptr`.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
#[no_mangle]
pub unsafe extern "C" fn ResumeTask(db_path_ptr: *const c_char) -> i32 {
    let data_base = match try_get_db(db_path_ptr) {
        Ok(db) => db,
        Err(ec) => return ec,
    };

    let result_code = match data_base.resume_task() {
        Ok(_) => 0,
        Err(error) => {
            log::error!("ResumeTask error occured: {error:?}");
            error.code()
        }
    };

    log::logger().flush();
    result_code
}

//

//

/// fn MergeTags(`db_path_ptr`, `tag1`, `tag2`);
///
/// Attempts to merge `tag1` into `tag2` in the `DataBase` at the provided `db_path_ptr`.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
#[no_mangle]
pub unsafe extern "C" fn MergeTags(db_path_ptr: *const c_char, tag1: u16, tag2: u16) -> i32 {
    let data_base = match try_get_db(db_path_ptr) {
        Ok(db) => db,
        Err(ec) => return ec,
    };

    let result_code = match data_base.merge_tags(tag1, tag2) {
        Ok(_) => 0,
        Err(error) => {
            log::error!("MergeTags error occured: {error:?}");

            error.code()
        }
    };

    log::logger().flush();
    result_code
}

//

//

/// fn RegenerateTagSums(`db_path_ptr`);
///
/// Attempts to regenerate all tag sums in the `DataBase` at the provided `db_path_ptr`.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
#[no_mangle]
pub unsafe extern "C" fn RegenerateTagSums(db_path_ptr: *const c_char) -> i32 {
    let data_base = match try_get_db(db_path_ptr) {
        Ok(db) => db,
        Err(ec) => return ec,
    };

    let result_code = match data_base.regen_tag_sums() {
        Ok(_) => 0,
        Err(error) => {
            log::error!("RegenerateTagSums error occured: {error:?}");

            error.code()
        }
    };

    log::logger().flush();
    result_code
}

//

//

/// fn RenameTag(`db_path_ptr`, `old_tag_ptr`, `new_tag_ptr`);
///
/// Attempts to rename `old_tag_ptr` to `new_tag_ptr` in the `DataBase` at `db_path_ptr`.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
#[no_mangle]
pub unsafe extern "C" fn RenameTag(
    db_path_ptr: *const c_char,
    old_tag_ptr: *const c_char,
    new_tag_ptr: *const c_char,
) -> i32 {
    let data_base = match try_get_db(db_path_ptr) {
        Ok(db) => db,
        Err(ec) => return ec,
    };
    let Ok(old_tag) = try_ptr_to_string(old_tag_ptr) else {
        return -2;
    };
    let Ok(new_tag) = try_ptr_to_string(new_tag_ptr) else {
        return -3;
    };

    let result_code = match data_base.rename_tag(old_tag.to_string(), new_tag.to_string()) {
        Ok(_) => 0,
        Err(error) => {
            log::error!("RenameTag error occured: {error:?}");
            error.code()
        }
    };

    log::logger().flush();
    result_code
}

//

//

/// fn TemporaryUpdateDatabase(`db_path_ptr`) 6
///
/// Attempts to upgrade the `DataBase` at the proided `db_path_ptr` to the new format.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
#[no_mangle]
pub unsafe extern "C" fn TemporaryUpdateDatabase(db_path_ptr: *const c_char) -> i32 {
    let data_base_path = match try_ptr_to_string(db_path_ptr) {
        Ok(db_path) => db_path,
        Err(ec) => return ec,
    };

    let result_code = match DataBase::upgrade_database(Path::new(&data_base_path)) {
        Ok(_) => 0,
        Err(error) => {
            println!("Error occured!\n{:?}", error);

            error.code()
        }
    };

    log::logger().flush();
    result_code
}

//

//

/// fn InsertDataEntry(`db_path_ptr`, `year`, `month`, `day`, `data`, `data_length`);
///
/// Attempts to insert the `DataEntry` stored in the `data` parameter to the `DataFile` matching the
/// provided `year`, `month` and `day` in the `DataBase` at the provided `db_path_ptr`.
/// Any existing entry at the target location is overwritten.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
#[no_mangle]
pub unsafe extern "C" fn InsertDataEntry(
    db_path_ptr: *const c_char,
    year: i32,
    month: i32,
    day: i32,
    data: *const u16,
    data_length: u32,
) -> i32 {
    if data.is_null() {
        return -3;
    }

    let data = unsafe { std::slice::from_raw_parts(data, data_length as usize) };

    let data_base = match try_get_db(db_path_ptr) {
        Ok(db) => db,
        Err(ec) => return ec,
    };

    let data_entry = match DataEntry::from_c_data(data) {
        Ok(data_entry) => data_entry,
        Err(error) => {
            log::error!("InsertDataEntry error occured! {error:?}");
            return data_base::Error::from(error).code();
        }
    };

    let result_code = match data_base.insert_data_entry(year, month, day, data_entry) {
        Ok(_) => 0,
        Err(error) => {
            log::error!("InsertDataEntry error occured: {error:?}");
            error.code()
        }
    };

    log::logger().flush();
    result_code
}

//

//

/// fn AddDataEntry(`db_path_ptr`, `year`, `month`, `day`, `data`, `data_length`);
///
/// Attempts to add the `DataEntry` stored in the `data` parameter to the `DataFile` matching the
/// provided `year`, `month` and `day` in the `DataBase` at the provided `db_path_ptr`.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
#[no_mangle]
pub unsafe extern "C" fn AddDataEntry(
    db_path_ptr: *const c_char,
    year: i32,
    month: i32,
    day: i32,
    data: *const u16,
    data_length: u32,
) -> i32 {
    if data.is_null() {
        return -3;
    }

    let data = unsafe { std::slice::from_raw_parts(data, data_length as usize) };

    let data_base = match try_get_db(db_path_ptr) {
        Ok(db) => db,
        Err(ec) => return ec,
    };

    let data_entry = match DataEntry::from_c_data(data) {
        Ok(data_entry) => data_entry,
        Err(error) => {
            log::error!("AddDataEntry error occured! {error:?}");
            log::logger().flush();
            return data_base::Error::from(error).code();
        }
    };

    let result_code = match data_base.add_data_entry(year, month, day, data_entry) {
        Ok(_) => 0,
        Err(error) => {
            log::error!("AddDataEntry error occured: {error:?}");
            error.code()
        }
    };

    log::logger().flush();
    result_code
}

//

//

/// fn AddTag(`db_path_ptr`, `tag_name_ptr`);
///
/// Attempts to add the tag name provided with `tag_name_ptr` to the database at the path specified
/// by `db_path_ptr`.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
#[no_mangle]
pub unsafe extern "C" fn AddTag(db_path_ptr: *const c_char, tag_name_ptr: *const c_char) -> i32 {
    let data_base = match try_get_db(db_path_ptr) {
        Ok(db) => db,
        Err(ec) => return ec,
    };
    let Ok(tag_name) = try_ptr_to_string(tag_name_ptr) else {
        return -2;
    };

    let result_code = match data_base.add_tag(tag_name) {
        Ok(_) => 0,
        Err(error) => {
            log::error!("AddTag error occured: {error:?}");

            error.code()
        }
    };

    log::logger().flush();
    result_code
}

//

//

/// fn RemoveTag(`db_path_ptr`, `tag_id`);
///
/// Attempts to remove the tag with the provided `tag_id` from the database at the path specified
/// by `db_path_ptr`.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
#[no_mangle]
pub unsafe extern "C" fn RemoveTag(db_path_ptr: *const c_char, tag_id: u16) -> i32 {
    let data_base = match try_get_db(db_path_ptr) {
        Ok(db) => db,
        Err(ec) => return ec,
    };

    let result_code = match data_base.remove_tag(tag_id) {
        Ok(_) => 0,
        Err(error) => {
            log::error!("RemoveTag error occured: {error:?}");
            error.code()
        }
    };

    log::logger().flush();
    result_code
}

//

//

/// Attempts to create a rust `String` using the provided `ptr`.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
unsafe fn try_ptr_to_string(ptr: *const c_char) -> Result<String, i32> {
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

/// Attempts to create a `DataBase` from the string path provided in `ptr`.
///
/// # Safety
///
/// Any parameter mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocation!
///     * `ptr` must be non-null even for a zero-length cstr.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
unsafe fn try_get_db(db_path_ptr: *const c_char) -> Result<DataBase, i32> {
    let db_path_str = try_ptr_to_string(db_path_ptr)?;
    match DataBase::load(PathBuf::from(db_path_str)) {
        Ok(db_path) => Ok(db_path),
        Err(err) => Err(err.code()),
    }
}
