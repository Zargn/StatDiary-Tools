use std::ffi::CString;

fn main() {
    println!("Hello, world!");

    let path = "/data/data_base/";
    let c_path = CString::new(path).unwrap();
    let result_path = "/data/compressed.zip";
    let c_result_path = CString::new(result_path).unwrap();

    let rc =
        unsafe { stat_diary_tools::CompressDBToImage(c_path.as_ptr(), c_result_path.as_ptr()) };

    println!("Exit code: {rc}");
}
