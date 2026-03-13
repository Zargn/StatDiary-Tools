use std::{
    ffi::{c_char, CStr},
    path::{Path, PathBuf},
};

use crate::{
    backup::{compress_to_image, load_image},
    cache_handling::regenerate_caches,
    db_path::{self, DataBasePath},
    db_status::{ActiveTask, DBStatus, DBStatusError},
    merge_tags, merge_tags_wrapper, rename_tag,
    stat_sums::regenerate_tag_sums,
    update_database::temporary_update_database,
};

//

//

#[no_mangle]
pub unsafe extern "C" fn CompressDBToImage(
    db_path_ptr: *const c_char,
    result_path: *const c_char,
) -> i32 {
    let db_path = match try_get_db_path(db_path_ptr) {
        Ok(db_path) => db_path,
        Err(ec) => return ec,
    };

    let result_path = match try_ptr_to_string(result_path) {
        Ok(str) => str,
        Err(ec) => return ec,
    };

    if let Err(error) = compress_to_image(&db_path, Path::new(&result_path)) {
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
    db_path_ptr: *const c_char,
    db_image_path: *const c_char,
) -> i32 {
    let db_path = match try_get_db_path(db_path_ptr) {
        Ok(db_path) => db_path,
        Err(ec) => return ec,
    };

    if db_image_path.is_null() {
        return -1;
    }
    let db_image_path = unsafe { CStr::from_ptr(db_image_path).to_string_lossy() };

    if let Err(error) = load_image(db_path.root(), Path::new(&db_image_path.to_string())) {
        println!("Error occured! [{:?}]", error);
        return -2;
    }

    //compress_db_to_image(&db_path, &result_path);

    1
}

//

//

#[no_mangle]
pub unsafe extern "C" fn RegenerateCaches(db_path_ptr: *const c_char) -> i32 {
    let db_path = match try_get_db_path(db_path_ptr) {
        Ok(db_path) => db_path,
        Err(ec) => return ec,
    };

    let Ok(db_status) = DBStatus::lock(&db_path, ActiveTask::RegenerateCaches) else {
        println!("Database is busy! Aborting...");
        return -3;
    };

    if let Err(error) = regenerate_caches(&db_path) {
        println!("Error occured!\n{:?}", error);

        db_status.unlock();
        return -3;
        //return error.into_code();
    }

    db_status.unlock();

    0
}

//

//

#[no_mangle]
pub unsafe extern "C" fn ResumeTask(db_path_ptr: *const c_char) -> i32 {
    let db_path = match try_get_db_path(db_path_ptr) {
        Ok(db_path) => db_path,
        Err(ec) => return ec,
    };

    let activate_error = match DBStatus::lock(&db_path, ActiveTask::None) {
        Ok(db_status) => {
            db_status.unlock();
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

                db_status.unlock();
                return -3;
                //return error.into_code();
            }
        }
        ActiveTask::RegenerateTagSums => {
            if let Err(error) = regenerate_tag_sums(&db_path) {
                println!("Error occured!\n{:?}", error);

                db_status.unlock();
                return -3;
                //return error.into_code();
            }
        }
        ActiveTask::MergeTags(tag_1, tag_2) => {
            if let Err(error) = merge_tags(&db_path, tag_1, tag_2) {
                println!("Error occured!\n{:?}", error);

                db_status.unlock();
                return -3;
                //return error.into_code();
            }
        }
        ActiveTask::RenameTag(old_tag, new_tag) => {
            if let Err(error) = rename_tag(&db_path, old_tag, new_tag) {
                println!("Error occured!\n{:?}", error);

                db_status.unlock();
                return -3;
                //return error.into_code();
            }
        }
        ActiveTask::None => {}
    }

    db_status.unlock();

    0
}

//

//
// TODO ################################################################## TODO
// Change tag ptr strings to plain u16 values.
//
//
#[no_mangle]
pub unsafe extern "C" fn MergeTags(db_path_ptr: *const c_char, tag1: u16, tag2: u16) -> i32 {
    let db_path = match try_get_db_path(db_path_ptr) {
        Ok(db_path) => db_path,
        Err(ec) => return ec,
    };
    /*
    let Ok(db_status) =
        DBStatus::lock(db_path.to_path_buf(), ActiveTask::MergeTags(tag1, tag2))
    else {
        println!("Database is busy! Aborting...");
        return -3;
    }; */

    if let Err(error) = merge_tags_wrapper(&db_path, tag1, tag2) {
        println!("Error occured!\n{:?}", error);

        //db_status.unlock();
        return -3;
        //return error.into_code();
    }

    //db_status.unlock();

    0
}

//

//

#[no_mangle]
pub unsafe extern "C" fn RegenerateTagSums(db_path_ptr: *const c_char) -> i32 {
    let db_path = match try_get_db_path(db_path_ptr) {
        Ok(db_path) => db_path,
        Err(ec) => return ec,
    };

    let Ok(db_status) = DBStatus::lock(&db_path, ActiveTask::RegenerateTagSums) else {
        println!("Database is busy! Aborting...");
        return -3;
    };

    if let Err(error) = regenerate_tag_sums(&db_path) {
        println!("Error occured! \n{:?}", error);
        db_status.unlock();
        return -3;
        //return error.into_code();
    }

    db_status.unlock();

    0
}

//

//

#[no_mangle]
pub unsafe extern "C" fn RenameTag(
    db_path_ptr: *const c_char,
    old_tag_ptr: *const c_char,
    new_tag_ptr: *const c_char,
) -> i32 {
    let db_path = match try_get_db_path(db_path_ptr) {
        Ok(db_path) => db_path,
        Err(ec) => return ec,
    };
    let Ok(old_tag) = try_ptr_to_string(old_tag_ptr) else {
        return -2;
    };
    let Ok(new_tag) = try_ptr_to_string(new_tag_ptr) else {
        return -3;
    };

    if let Err(error) = rename_tag(&db_path, old_tag.to_string(), new_tag.to_string()) {
        println!("Error occured!\n{:?}", error);
        return -3;
        //return error.into_code();
    }

    0
}

//

//

#[no_mangle]
pub unsafe extern "C" fn TemporaryUpdateDatabase(db_path_ptr: *const c_char) -> i32 {
    let db_path = match try_get_db_path(db_path_ptr) {
        Ok(db_path) => db_path,
        Err(ec) => return ec,
    };

    if let Err(error) = temporary_update_database(&db_path) {
        println!("Error occured!\n{:?}", error);
        //return error.into_code();
    }

    // TODO: Regenerate caches too!

    0
}

//

//

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

unsafe fn try_get_db_path(db_path_ptr: *const c_char) -> Result<DataBasePath, i32> {
    let db_path_str = try_ptr_to_string(db_path_ptr)?;
    match DataBasePath::new(PathBuf::from(db_path_str)) {
        Ok(db_path) => Ok(db_path),
        Err(db_path::DataBasePathError::DoesNotExist) => Err(-3),
        Err(db_path::DataBasePathError::IsNotDataBase) => Err(-4),
    }
}
