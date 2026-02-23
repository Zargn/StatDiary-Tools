use std::{
    error::Error,
    io::{Cursor, Read},
};

use image::{ImageBuffer, ImageReader};
use std::fs::File;
use std::path::Path;
use walkdir::WalkDir;

use zip::{result::ZipError, write::SimpleFileOptions};

pub fn compress_to_image(db_path: &Path, result_path: &Path) -> Result<(), Box<dyn Error>> {
    let method = zip::CompressionMethod::Ppmd;

    let data = zip_dir(db_path, method)?;
    convert_to_image(result_path, data);

    get_data_from_image(result_path)?;

    //todo!();
    Ok(())
}

//

//

fn convert_to_image(target_path: &Path, data: Vec<u8>) {
    let byte_count = data.len() as u32;
    let img_size = ((byte_count as f64 + 4.0) / 4.0).sqrt().ceil() as u32;

    println!("Byte count: {}\nImg size: {}", byte_count, img_size);

    let mut imgbuf = ImageBuffer::new(img_size, img_size);

    let bytes = byte_count.to_be_bytes();
    let mut i = 0;
    for (_, _, pixel) in imgbuf.enumerate_pixels_mut().take(1) {
        *pixel = image::Rgba([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
        i += 4;
    }

    let mut data_iter = data.iter();

    for (_, _, pixel) in imgbuf.enumerate_pixels_mut().skip(2) {
        *pixel = image::Rgba([
            get_byte(data_iter.next()),
            get_byte(data_iter.next()),
            get_byte(data_iter.next()),
            get_byte(data_iter.next()),
        ]);
    }

    imgbuf.save(target_path).unwrap();
}

fn get_byte(data: Option<&u8>) -> u8 {
    match data {
        Some(b) => *b,
        None => 0,
    }
}

fn get_data_from_image(img_path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let data = ImageReader::open(img_path)?.decode()?;
    let (int_bytes, image_data) = data.as_bytes().split_at(size_of::<u32>());
    let byte_count = u32::from_be_bytes(int_bytes.try_into()?);

    let data: Vec<&u8> = image_data.iter().take(byte_count as usize).collect();

    println!("byte_count: {}\nimg data len: {}", byte_count, data.len());
    Ok(Vec::new())
}

//

//

// Credit for most of the function below goes to the zip2 example at this link:
// https://github.com/zip-rs/zip2/blob/b19f6707111bdbcd76ddebcbe7cbee246683e2d2/examples/write_dir.rs
fn zip_dir(
    src_dir: &Path,
    method: zip::CompressionMethod,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound.into());
    }

    let walkdir = WalkDir::new(src_dir);

    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(src_dir)?;
        let path_as_string = name
            .to_str()
            .map(str::to_owned)
            .ok_or_else(|| format!("{name:?} is a Non UTF-8 Path"))?;

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            //println!("adding file {path:?} as {name:?} ...");
            zip.start_file(path_as_string, options)?;
            let mut f = File::open(path)?;

            std::io::copy(&mut f, &mut zip)?;
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            //println!("adding dir {path_as_string:?} as {name:?} ...");
            zip.add_directory(path_as_string, options)?;
        }
    }
    let result_data = zip.finish()?;

    Ok(result_data.into_inner())
}
