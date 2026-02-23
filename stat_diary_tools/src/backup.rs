use core::arch;
use std::{
    error::Error,
    fs,
    io::{copy, Cursor, Read},
    marker,
};

use image::{io, EncodableLayout, ImageBuffer, ImageReader};
use std::fs::File;
use std::path::Path;
use walkdir::WalkDir;

use zip::{result::ZipError, write::SimpleFileOptions, ZipArchive};

pub fn compress_to_image(db_path: &Path, result_path: &Path) -> Result<(), Box<dyn Error>> {
    let method = zip::CompressionMethod::Ppmd;

    let data = zip_dir(db_path, method)?;
    convert_to_image(result_path, data);

    //get_data_from_image(result_path)?;

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

pub fn load_image(db_path: &Path, img_path: &Path) -> Result<(), Box<dyn Error>> {
    let data = get_data_from_image(img_path)?;
    extract_db(db_path, data)
}

fn get_data_from_image(img_path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let data = ImageReader::open(img_path)?.decode()?;
    let bytes = data.as_bytes();
    if bytes.iter().take(4).any(|b| *b != 255) {
        return Err("This image does not hold a compressed database!".into());
    }
    let (int_bytes, image_data) = bytes.split_at(4).1.split_at(size_of::<u32>());
    let byte_count = u32::from_be_bytes(int_bytes.try_into()?);

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

//

//

fn extract_db(target_path: &Path, data: Vec<u8>) -> Result<(), Box<dyn Error>> {
    /*
    let mut archive = match fs::File::open(target_path)
        .map_err(ZipError::from)
        .and_then(ZipArchive::new)
    {
        Ok(archive) => archive,
        Err(e) => {
            eprintln!(
                "Error: unable to open archive {:?}: {e}",
                target_path.display()
            );
            return Err(e.into());
        }
    }; */

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

    /*
    let mut some_files_failed = false;
    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error: unable to open file {i} in archive: {e}");
                some_files_failed = true;
                continue;
            }
        };
        let out_path = match file.enclosed_name() {
            Some(path) => path,
            None => {
                eprintln!(
                    "Error: unable to extract file {:?} because it has an invalid path.",
                    file.name()
                );
                some_files_failed = true;
                continue;
            }
        };
        let comment = file.comment();
        if !comment.is_empty() {
            println!("File {i} comment: {comment:?}");
        }
        if file.is_dir() {
            if let Err(e) = fs::create_dir_all(&out_path) {
                eprintln!(
                    "Error: unable to extract directory {i} to {:?}: {e}",
                    out_path.display()
                );
                some_files_failed = true;
                continue;
            } else {
                println!("Directory {i} extracted to {:?}", out_path.display());
            }
        } else {
            if let Some(p) = out_path.parent() {
                if !p.exists() {
                    if let Err(e) = fs::create_dir_all(p) {
                        eprintln!(
                            "Error: unable to create parent directory {p:?} of file {}: {e}",
                            p.display()
                        );
                        some_files_failed = true;
                        continue;
                    }
                }
            }
            match fs::File::create(&out_path).and_then(|mut outfile| copy(&mut file, &mut outfile))
            {
                Ok(bytes_extracted) => {
                    println!(
                        "File {} extracted to {:?} ({bytes_extracted} bytes)",
                        i,
                        out_path.display(),
                    );
                }
                Err(e) => {
                    eprintln!(
                        "Error: unable to extract file {i} to {:?}: {e}",
                        out_path.display()
                    );
                    some_files_failed = true;
                    continue;
                }
            }
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                if let Err(e) = fs::set_permissions(&out_path, fs::Permissions::from_mode(mode)) {
                    eprintln!(
                        "Error: unable to change permissions of file {i} ({:?}): {e}",
                        out_path.display()
                    );
                    some_files_failed = true;
                }
            }
        }
    }

    if some_files_failed {
        eprintln!("Error: some files failed to extract; see above errors.");
        Err("Extraction partially failed".into())
    } else {
        Ok(())
    }*/
    Ok(())
}
