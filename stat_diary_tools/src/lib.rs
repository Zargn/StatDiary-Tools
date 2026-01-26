use std::ffi::{c_char, CStr};

#[no_mangle]
pub unsafe extern "C" fn compress_db_to_image(path: *const c_char) -> i32 {
    if path.is_null() {
        return -1;
    }
    let path = unsafe { CStr::from_ptr(path).to_string_lossy() };

    local_compress_db_to_image(&path);

    1
}

fn local_compress_db_to_image(db_path: &str) -> Result<(), ()> {
    println!("Success! Using path: {}", db_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::local_compress_db_to_image;

    #[test]
    fn test_compress_db_to_image() {
        local_compress_db_to_image("none");
    }
}
