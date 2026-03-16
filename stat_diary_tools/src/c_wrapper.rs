use std::{
    ffi::{c_char, CStr},
    path::{Path, PathBuf},
};

use crate::data_base::DataBase;

//

//

/// Compresses the database at the provided path into a image stored at the provided image path.
///
/// # Safety
///
/// Any arguemnt mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
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

    if let Err(error) = data_base.compress_to_image(Path::new(&result_path)) {
        log::error!("CompressDBToImage error occured: {error:?}");
        return error.code();
    }

    1
}

//

//

/// Attempts to extract a `DataBase` from the provided `db_image_path_ptr` into `db_path_ptr`.
///
/// # Safety
///
/// Any arguemnt mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
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
    db_path_ptr: *const c_char,
    db_image_path_ptr: *const c_char,
) -> i32 {
    let db_path = match try_ptr_to_string(db_path_ptr) {
        Ok(str) => str,
        Err(ec) => return ec,
    };

    let db_image_path = match try_ptr_to_string(db_image_path_ptr) {
        Ok(str) => str,
        Err(ec) => return ec,
    };

    if let Err(error) =
        DataBase::load_from_image(Path::new(&db_image_path), Path::new(&db_path).to_path_buf())
    {
        log::error!("ExtractDBFromImage error occured: {error:?}");
        return error.code();
    }

    1
}

//

//

/// Attempts to regenerate the cache files in the `DataBase` at the provided `db_path_ptr`.
///
/// # Safety
///
/// Any arguemnt mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
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

    if let Err(error) = data_base.regen_caches() {
        log::error!("RegenerateCaches error occured: {error:?}");
        return error.code();
    }

    0
}

//

//

/// Attempts to resume any non-finished tasks in the `DataBase` at the provided `db_path_ptr`.
///
/// # Safety
///
/// Any arguemnt mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
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

    if let Err(error) = data_base.resume_task() {
        log::error!("ResumeTask error occured: {error:?}");
        return error.code();
    }

    0
}

//

//

/// Attempts to merge `tag1` into `tag2` in the `DataBase` at the provided `db_path_ptr`.
///
/// # Safety
///
/// Any arguemnt mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
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

    if let Err(error) = data_base.merge_tags(tag1, tag2) {
        log::error!("MergeTags error occured: {error:?}");
        return error.code();
    }

    0
}

//

//

/// Attempts to regenerate all tag sums in the provided `DataBase` at the provide `db_path_ptr`.
///
/// # Safety
///
/// Any arguemnt mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
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

    if let Err(error) = data_base.regen_tag_sums() {
        log::error!("RegenerateTagSums error occured: {error:?}");
        return error.code();
    }

    0
}

//

//

/// Attempts to rename `old_tag_ptr` to `new_tag_ptr` in the `DataBase` at `db_path_ptr`.
///
/// # Safety
///
/// Any arguemnt mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
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

    if let Err(error) = data_base.rename_tag(old_tag.to_string(), new_tag.to_string()) {
        log::error!("RenameTag error occured: {error:?}");
        return error.code();
    }

    0
}

//

//

/// Attempts to upgrade the `DataBase` at the proided `db_path_ptr` to the new format.
///
/// # Safety
///
/// Any arguemnt mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
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
    let _data_base = match try_get_db(db_path_ptr) {
        Ok(db_path) => db_path,
        Err(ec) => return ec,
    };

    /*
    if let Err(error) = DataBase::upgrade_database() {
        println!("Error occured!\n{:?}", error);
        //return error.into_code();
    }*/

    todo!();

    // TODO: Regenerate caches too!
}

//

//

/// Attempts to create a rust `String` using the provided `ptr`.
///
/// # Safety
///
/// Any arguemnt mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
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
/// Any arguemnt mentioning `ptr` must satisfy the requirements of `CStr::from_ptr`:
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
