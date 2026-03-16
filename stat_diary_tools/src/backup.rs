use core::panic;
use std::{
    error::Error,
    io::{self, Cursor},
};

use image::{ImageBuffer, ImageReader};
use std::fs::File;
use std::path::Path;
use walkdir::WalkDir;

use zip::{result::ZipError, write::SimpleFileOptions, ZipArchive};

use crate::db_path::DataBasePath;

pub fn compress_to_image(
    db_path: &DataBasePath,
    result_path: &Path,
) -> Result<(), BackupImageError> {
    let method = zip::CompressionMethod::Ppmd;

    let data = zip_dir(db_path.root(), method)?;
    convert_to_image(result_path, data);

    Ok(())
}

//

//

fn convert_to_image(target_path: &Path, data: Vec<u8>) {
    let byte_count = data.len() as u32;
    let img_size = ((byte_count as f64 + 8.0) / 4.0).sqrt().ceil() as u32;

    println!("Byte count: {}\nImg size: {}", byte_count, img_size);

    let mut imgbuf = ImageBuffer::new(img_size, img_size);

    let bytes = byte_count.to_be_bytes();
    let markers = [image::Rgba([255, 255, 255, 255]), image::Rgba(bytes)];

    for (i, (_, _, pixel)) in imgbuf.enumerate_pixels_mut().take(2).enumerate() {
        *pixel = markers[i];
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

#[derive(Debug)]
pub enum BackupImageError {
    Io(io::Error),
    InvalidImage,
    UnableToZip,
}

impl From<io::Error> for BackupImageError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<zip::result::ZipError> for BackupImageError {
    fn from(value: zip::result::ZipError) -> Self {
        log::error!("Could not zip database due to a [{value:?}] error!");
        Self::UnableToZip
    }
}

pub fn load_image(img_path: &Path, db_path: &Path) -> Result<(), BackupImageError> {
    let data = get_data_from_image(img_path)?;
    extract_db(db_path, data)
}

fn get_data_from_image(img_path: &Path) -> Result<Vec<u8>, BackupImageError> {
    let img_reader = match ImageReader::open(img_path) {
        Ok(reader) => reader,
        Err(error) => {
            log::error!("Could not open reader for image at: {img_path:?} Error: {error:?}");
            return Err(BackupImageError::InvalidImage);
        }
    };
    let data = match img_reader.decode() {
        Ok(data) => data,
        Err(error) => {
            log::error!("Could not decode image at: {img_path:?} Error: {error:?}");
            return Err(BackupImageError::InvalidImage);
        }
    };
    let bytes = data.as_bytes();
    if bytes.iter().take(4).any(|b| *b != 255) {
        return Err(BackupImageError::InvalidImage);
    }
    let (int_bytes, image_data) = bytes.split_at(4).1.split_at(size_of::<u32>());
    let byte_count = u32::from_be_bytes(match int_bytes.try_into() {
        Ok(uint) => uint,
        Err(error) => {
            log::error!("get_data_from_image(): Could not get byte count from image!");
            return Err(BackupImageError::InvalidImage);
        }
    });

    let data: Vec<u8> = image_data
        .iter()
        .take(byte_count as usize)
        .map(|b| *b)
        .collect::<Vec<u8>>();

    println!("byte_count: {}\nimg data len: {}", byte_count, data.len());
    Ok(data)
}

//

//

// Credit for most of the function below goes to the zip2 example at this link:
// https://github.com/zip-rs/zip2/blob/b19f6707111bdbcd76ddebcbe7cbee246683e2d2/examples/write_dir.rs
fn zip_dir(src_dir: &Path, method: zip::CompressionMethod) -> Result<Vec<u8>, BackupImageError> {
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
        let name = path.strip_prefix(src_dir)
            .expect(
                "This should never panic as this path is created by a WalkDir that traverses the same src_dir that is used by the strip_prefix method. I.e. the path should always start with src_dir, meaning strip_prefix(src_dir) will always succeed."
                );

        let path_as_string = name.to_str().map(str::to_owned).ok_or_else(|| {
            log::error!("Could not zip database due to [{name:?}] not being valid unicode!");
            BackupImageError::UnableToZip
        })?;

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

//

//

fn extract_db(target_path: &Path, data: Vec<u8>) -> Result<(), BackupImageError> {
    let mut archive = match ZipArchive::new(Cursor::new(data)) {
        Ok(archive) => archive,
        Err(e) => {
            eprintln!(
                "Error: unable to open archive {:?}: {e}",
                target_path.display()
            );
            return Err(e.into());
        }
    };
    archive.extract(target_path)?;
    Ok(())
}
