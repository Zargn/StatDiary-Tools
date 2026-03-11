use std::{
    io,
    path::{Path, PathBuf},
};

use log::error;

use crate::{
    backup, cache_handling,
    db_path::{DataBasePath, DataBasePathError},
    db_status::{self, ActiveTask, DBStatus, DBStatusError},
    merge_tags,
    stat_sums::{self, StatSumsError},
};

pub struct DataBase {
    path: DataBasePath,
}

type Result<T> = std::result::Result<T, Error>;

impl DataBase {
    /// Attempts to load the database at `db_path`.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is not limited to just
    /// these cases:
    ///
    /// * `db_path` does not lead to a existing directory.
    /// * `db_path` leads to a directory, but the directory is missing a `.db_marker` file
    pub fn load(db_path: PathBuf) -> Result<DataBase> {
        Ok(DataBase {
            path: DataBasePath::new(db_path)?,
        })
    }

    fn tmp() {
        //
        let db = DataBase::load(PathBuf::new()).unwrap();

        //
        todo!();
    }

    /// Attempts to extract a database from `img_path` into `db_path`.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is not limited to just
    /// these cases:
    ///
    /// * `img_path` does not lead to a image.
    /// * `img_path` leads to a image but said image is not a valid compressed database.
    pub fn load_from_image(img_path: &Path, db_path: PathBuf) -> Result<DataBase> {
        if let Err(e) = backup::load_image(&img_path, &db_path) {
            error!("DataBase::load_from_image could not load image! Error: {e:?}");
        }

        Ok(DataBase {
            path: DataBasePath::new(db_path)?,
        })
    }

    /// Attempts to resume any unfinished task.
    /// Will also return `Ok` if no task was active.  
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is not limited to just
    /// these cases:
    ///
    /// * The active task is unknown. (The task id does not match any known tasks.)
    /// * The active task is missing data. (The task id is known but the required data is missing.)
    /// * The task is resumed but encountered an error.
    pub fn resume_task(&self) -> Result<()> {
        let activate_error = match DBStatus::activate(&self.path, ActiveTask::None) {
            Ok(db_status) => {
                db_status.deactivate();
                return Ok(());
            }
            Err(db_error) => db_error,
        };

        let DBStatusError::DataBaseBusy(active_task, db_status) = activate_error else {
            return Err(Error::with_kind(ErrorKind::DBStatus(activate_error)));
        };

        match active_task {
            ActiveTask::None => {}
            ActiveTask::RegenerateCaches => cache_handling::regenerate_caches(&self.path)?,
            ActiveTask::RegenerateTagSums => stat_sums::regenerate_tag_sums(&self.path)?,

            ActiveTask::MergeTags(tag_1, tag_2) => {
                todo!();
            }
            ActiveTask::RenameTag(old_name, new_name) => {
                todo!();
            }
        }

        db_status.deactivate();

        Ok(())
    }

    /// Regenerates all the caches in the database.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is not limited to just
    /// these cases:
    ///
    /// * The database is busy.
    /// * An io error occured.
    ///
    /// If it encounters a unknown or corrupted file a warning or error is logged. The function
    /// will then continue on skipping the bad file.
    pub fn regen_caches(&self) -> Result<()> {
        let db_status = DBStatus::activate(&self.path, ActiveTask::RegenerateCaches)?;

        cache_handling::regenerate_caches(&self.path)?;

        db_status.deactivate();
        Ok(())
    }

    /// Regenerates all tag sums in the database.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is not limited to just
    /// these cases:
    ///
    /// * The database is busy.
    /// * An io error occured.
    pub fn regen_tag_sums(&self) -> Result<()> {
        let db_status = DBStatus::activate(&self.path, ActiveTask::RegenerateTagSums)?;

        stat_sums::regenerate_tag_sums(&self.path)?;

        db_status.deactivate();
        Ok(())
    }

    /// Merges `tag_1` into `tag_2`. Any existing reference to tag_1 will be changed to tag_2 if
    /// tag_2 doesn't already exist in that context.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is not limited to just
    /// these cases:
    ///
    pub fn merge_tags(&self, tag_1: u16, tag_2: u16) -> Result<()> {
        let db_status = DBStatus::activate(&self.path, ActiveTask::RegenerateTagSums)?;

        // TODO Error handling...
        merge_tags(&self.path, tag_1, tag_2)?;

        if let Err(e) = stat_sums::regenerate_tag_sums(&self.path) {
            error!(
                "MergeTags() received {:?} when attempting to regenerate tag sums!",
                e
            );
        }

        if let Err(e) = cache_handling::regenerate_caches(&self.path) {
            error!(
                "MergeTags() received {:?} when attempting to regenerate caches!",
                e
            );
        }

        Ok(())
    }
    pub fn rename_tag() {}
    pub fn compress_to_image() {}
    pub fn upgrade_database() {}
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    fn with_kind(kind: ErrorKind) -> Error {
        Self { kind }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Io(io::Error),
    DataBasePath(DataBasePathError),
    DBStatus(DBStatusError),
    StatSum(StatSumsError),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(value),
        }
    }
}

impl From<DataBasePathError> for Error {
    fn from(value: DataBasePathError) -> Self {
        Self {
            kind: ErrorKind::DataBasePath(value),
        }
    }
}

impl From<DBStatusError> for Error {
    fn from(value: DBStatusError) -> Self {
        Self {
            kind: ErrorKind::DBStatus(value),
        }
    }
}

impl From<StatSumsError> for Error {
    fn from(value: StatSumsError) -> Self {
        Self {
            kind: ErrorKind::StatSum(value),
        }
    }
}
