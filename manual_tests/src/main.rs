use std::{collections::HashMap, ffi::CString};

fn main() {
    println!("Hello, world!");

    /*
    let path = "/data/data_base/";
    let c_path = CString::new(path).unwrap();
    let result_path = "/data/compressed.zip";
    let c_result_path = CString::new(result_path).unwrap();

    let rc =
        unsafe { stat_diary_tools::CompressDBToImage(c_path.as_ptr(), c_result_path.as_ptr()) };

    println!("Exit code: {rc}");
    */

    let mut tags = HashMap::new();
    if let Err(e) = stat_diary_tools::transform_data_file("21-Wednesday.txt", &mut tags) {
        println!("transform error: {}", e);
        return;
    }

    let mut tags_list = HashMap::new();
    for (key, value) in tags.iter() {
        tags_list.insert(*value, key.clone());
    }
    if let Err(e) = stat_diary_tools::temp_read_data_file("21-2", &tags_list) {
        println!("read error: {}", e);
    }
}
