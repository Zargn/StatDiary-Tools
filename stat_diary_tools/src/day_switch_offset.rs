use std::{fs, path::Path};

use crate::data_base::{self, DataBase, Error, ErrorKind};

type Result<T> = std::result::Result<T, Error>;

const TEMPDBPATH: &str = "data_base_tmp";

pub fn change_day_switch_offset(database: DataBase, new_offset: i8) -> Result<()> {
    if !(-12..=12).contains(&new_offset) {
        log::error!("Attempted to set day_switch_offset to a value outside of -12..=12!");
        return Err(Error::with_kind(ErrorKind::OffsetOutOfRange));
    }

    if database.settings().day_switch_offset == new_offset {
        log::warn!("Attempted to set day_switch_offset to the same value it already is! Nothing was changed.");
        return Ok(());
    }

    let orignal_path = database.database_path().to_path_buf();
    let dir_path = match orignal_path.parent() {
        None => {
            log::error!("The database had no parent path. This should not be possible.");
            return Err(Error::with_kind(ErrorKind::PathDoesNotExist));
        }
        Some(path) => path.to_path_buf(),
    };

    let database_copy = create_temp_copy(database, &dir_path)?;

    update_data_files(&database_copy)?;
    update_diary_files(&database_copy)?;

    database_copy.regen_caches()?;
    database_copy.regen_tag_sums()?;

    fs::rename(dir_path.join(TEMPDBPATH), orignal_path)?;

    Ok(())
}

fn create_temp_copy(database: DataBase, dir_path: &Path) -> Result<DataBase> {
    log::info!("Creating database copy...");
    let img_path = dir_path.join("temp_backup.png");
    database.compress_to_image(&img_path)?;
    let database_copy = DataBase::load_from_image(&img_path, dir_path.join(TEMPDBPATH))?;
    fs::remove_file(img_path)?;
    log::info!("Finished copying database.");
    Ok(database_copy)
}

fn update_data_files(database: &DataBase) -> Result<()> {
    log::info!("Updating data files...");
    log::info!("Finished updating data files.");
    todo!();
}

fn update_diary_files(database: &DataBase) -> Result<()> {
    log::info!("Updating diary files...");
    log::info!("Finished updating diary files.");
    todo!();
}
