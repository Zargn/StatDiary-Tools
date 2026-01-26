use std::ffi::{c_char, CStr};

#[no_mangle]
pub unsafe extern "C" fn test_db(path: *const c_char) -> i32 {
    if path.is_null() {
        return -1;
    }
    let path = unsafe { CStr::from_ptr(path).to_string_lossy() };

    compress_db_to_image(&path);

    1
}

fn compress_db_to_image(db_path: &str) -> Result<(), ()> {
    todo!();
}

pub fn tester() {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::compress_db_to_image;

    #[test]
    fn test_compress_db_to_image() {
        compress_db_to_image("none");
    }
}
