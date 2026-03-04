use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

use walkdir::WalkDir;

use crate::{
    db_path::DataBasePath, stat_sums::regenerate_tag_sums, tags::TagList, utilities::read_lines,
    DATAFILEEXTENSION,
};

pub fn temporary_update_database(db_path: &DataBasePath) -> Result<(), Box<dyn Error>> {
    let mut tags: HashMap<String, u16> = HashMap::new();

    // Iterate through data_base/data/
    // Call transform_data_file on each of them.

    for path in WalkDir::new(db_path.data()) {
        let path = path?;
        let path = path.path();
        if path.is_file() {
            if let Some(e) = path.extension() {
                if e.to_str() != Some("txt") {
                    continue;
                }
            }
            //println!("{:?}", path);
            transform_data_file(path.to_str().unwrap(), &mut tags)?;
        }
    }

    let Ok(tags_file) = File::create(db_path.root().join("tags.txt")) else {
        // Could not create a new file! It might already exist, or there is some other issue.
        return Err("Could not create a new file!".into());
    };

    let mut tags_writer = BufWriter::new(tags_file);

    for (k, v) in &tags {
        writeln!(tags_writer, "{} {}", v, k)?;
    }

    tags_writer.flush()?;
    //update_averages(Path::new(db_path))?;
    if let Err(e) = regenerate_tag_sums(db_path) {
        println!("regenerate_tag_sums error occured!\n{:?}", e);
    }

    Ok(())
}

//

//

pub fn transform_data_file(
    file_path: &str,
    tags: &mut HashMap<String, u16>,
) -> Result<(), Box<dyn Error>> {
    let Ok(lines) = read_lines(file_path) else {
        // File does not exist
        // Exit early with error
        return Err("File does not exist!".into());
    };

    let (day_of_month, day_of_week) = {
        let path = Path::new(file_path).file_stem().unwrap().to_os_string();
        let mut parts = path
            .to_str()
            .expect("The filename should always be valid a valid string, right?")
            .split('-');
        (
            parts.next().unwrap().to_string(),
            day_of_week(parts.next().unwrap()),
        )
    };

    let path = Path::new(file_path).parent().unwrap().to_str().unwrap();

    let Ok(result_file) = File::create(format!(
        "{}/{}-{}.{}",
        path, day_of_month, day_of_week, DATAFILEEXTENSION
    )) else {
        // Could not create a new file! It might already exist, or there is some other issue.
        return Err("Could not create a new file!".into());
    };

    let mut result_writer = BufWriter::new(result_file);

    for line in lines {
        let mut row_parts = line.split('|');
        row_parts.next();

        println!("Reading line: {}", line);

        // Hour
        parse_and_write(row_parts.next().unwrap(), ':', &mut result_writer)?;

        for _ in 0..2 {
            // Mental and physical score.
            parse_and_write(row_parts.next().unwrap(), ',', &mut result_writer)?;
        }

        // Tags
        for tag_str in row_parts.next().unwrap().split(' ').map(|s| s.to_string()) {
            let tags_len = tags.len() as u16;
            let id = tags.entry(tag_str).or_insert(tags_len);
            result_writer.write_all(&id.to_be_bytes())?;
        }

        // End of entry marker
        result_writer.write_all(&u16::MAX.to_be_bytes())?;
    }

    result_writer.flush()?;
    std::fs::remove_file(file_path)?;

    Ok(())
}

//

//

fn parse_and_write(
    data_str: &str,
    split: char,
    writer: &mut impl io::Write,
) -> Result<(), Box<dyn Error>> {
    writer.write_all(&[data_str.split(split).next().unwrap().parse::<u8>()?])?;
    Ok(())
}

//

//

fn day_of_week(day_name: &str) -> u8 {
    match day_name {
        "Monday" => 0,
        "Tuesday" => 1,
        "Wednesday" => 2,
        "Thursday" => 3,
        "Friday" => 4,
        "Saturday" => 5,
        "Sunday" => 6,
        _ => u8::MAX,
    }
}

//

//

pub fn update_averages(db_path: &DataBasePath) -> Result<(), Box<dyn Error>> {
    let taglist = TagList::from_file(db_path).unwrap();
    for path in WalkDir::new(db_path.root().join("averages")) {
        let path = path?;
        if path.path().is_file() {
            println!("{:?}", path);
            transform_average_file(&taglist, path.path())?;
        }
    }

    Ok(())
}

//

//

fn transform_average_file(taglist: &TagList, file_path: &Path) -> Result<(), Box<dyn Error>> {
    let new_file_path = file_path.with_extension("stat_avg");
    let mut result_writer = BufWriter::new(File::create(new_file_path)?);
    //println!("New avg file: {:?}", new_file_path);

    for line in read_lines(file_path)? {
        let mut parts = line.split(' ');
        let (occurances, tag) = (parts.next().unwrap(), parts.next().unwrap());
        println!("{} | {}", occurances, tag);
        writeln!(
            result_writer,
            "{} {}",
            occurances,
            taglist.get_id(tag).unwrap()
        )?;
        //writeln!(result_writer, "{} {}")
        //todo!();
    }

    result_writer.flush()?;

    std::fs::remove_file(file_path)?;

    Ok(())
    //todo!();
}
