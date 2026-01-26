use std::ffi::CString;

fn main() {
    println!("Hello, world!");

    let path = "/tmp/my_db";
    let c_path = CString::new(path).unwrap();

    let rc = unsafe { stat_diary_tools::compress_db_to_image(c_path.as_ptr()) };

    println!("Exit code: {rc}");
}
