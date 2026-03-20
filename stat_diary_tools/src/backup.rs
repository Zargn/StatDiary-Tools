use image::{ImageBuffer, ImageError, ImageReader};
use std::fs::File;
use std::io::{self, Cursor};
use std::path::Path;
use walkdir::WalkDir;

use zip::{result::ZipError, write::SimpleFileOptions, ZipArchive};

use crate::db_path::DataBasePath;

#[derive(Debug)]
pub enum BackupImageError {
    Io(io::Error),
    InvalidImage,
    UnableToZip,
    Image(ImageError),
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

impl From<ImageError> for BackupImageError {
    fn from(value: ImageError) -> Self {
        Self::Image(value)
    }
}

//

//

/// Compress the database at `db_path` into a image and save said image to `result_path`.
pub fn compress_database_to_image(
    db_path: &DataBasePath,
    result_path: &Path,
) -> Result<(), BackupImageError> {
    let method = zip::CompressionMethod::Ppmd;

    let data = zip_dir(db_path.root(), method)?;
    save_to_image(result_path, data)?;

    Ok(())
}

//

//

// Credit for the core of the function below goes to the zip2 example at this link:
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

fn save_to_image(target_path: &Path, data: Vec<u8>) -> Result<(), BackupImageError> {
    let byte_count = data.len() as u32;
    let img_size = ((byte_count as f64 + 8.0) / 4.0).sqrt().ceil() as u32;

    println!("Byte count: {}\nImg size: {}", byte_count, img_size);

    let mut imgbuf = ImageBuffer::new(img_size, img_size);

    let bytes = byte_count.to_be_bytes();
    let markers = [image::Rgba([255, 255, 255, 255]), image::Rgba(bytes)];

    // Mark the first pixel with 4 max bytes and the second pixel with the length of the data.
    for (i, (_, _, pixel)) in imgbuf.enumerate_pixels_mut().take(2).enumerate() {
        *pixel = markers[i];
    }

    let mut data_iter = data.into_iter();

    for (_, _, pixel) in imgbuf.enumerate_pixels_mut().skip(2) {
        *pixel = image::Rgba([
            data_iter.next().unwrap_or_default(), // Since the amount of data bytes is known it is
            data_iter.next().unwrap_or_default(), // fine to write a few 0 bytes if the data
            data_iter.next().unwrap_or_default(), // doesn't have the length to perfectly match the
            data_iter.next().unwrap_or_default(), // pixel count. They wont be read regardless.
        ]);
    }

    imgbuf.save(target_path)?;
    Ok(())
}

//

//

/// Attempt to unzip a database stored in the pixel data of the image at the provided `img_path`.
/// If successful the database is stored at the provided `db_path`.
pub fn load_image(img_path: &Path, db_path: &Path) -> Result<(), BackupImageError> {
    let data = get_data_from_image(img_path)?;
    extract_db(db_path, data)
}

//

//

/// Attempts to get the data hidden in the pixels from the image at the provided `img_path`.
fn get_data_from_image(img_path: &Path) -> Result<Vec<u8>, BackupImageError> {
    let img_reader = match ImageReader::open(img_path) {
        Ok(reader) => reader,
        Err(error) => {
            log::error!("Could not open reader for image at: {img_path:?} Error: {error:?}");
            return Err(BackupImageError::Image(error.into()));
        }
    };
    let data = match img_reader.decode() {
        Ok(data) => data,
        Err(error) => {
            log::error!("Could not decode image at: {img_path:?} Error: {error:?}");
            return Err(BackupImageError::Image(error));
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
            log::error!(
                "get_data_from_image(): Could not get byte count from image due to: {error:?}"
            );
            return Err(BackupImageError::InvalidImage);
        }
    });

    let data: Vec<u8> = image_data
        .iter()
        .take(byte_count as usize)
        .copied()
        .collect::<Vec<u8>>();

    println!("byte_count: {}\nimg data len: {}", byte_count, data.len());
    Ok(data)
}

//

//

/// Attempts to extract the database into `target_path` from the zip archive `data`.
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
