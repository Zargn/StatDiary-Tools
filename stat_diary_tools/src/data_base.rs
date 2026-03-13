use std::{
    io,
    path::{Path, PathBuf},
};

use log::error;

use crate::{
    backup, cache_handling,
    db_path::{DataBasePath, DataBasePathError},
    db_status::{ActiveTask, DBStatus, DBStatusError},
    merge_tags,
    stat_sums::{self, StatSumsError},
    tags::{TagList, TagsError},
    MergeTagsError,
};

pub struct DataBase {
    path: DataBasePath,
}

type Result<T> = std::result::Result<T, Error>;

// Public functions
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
        //let db = DataBase::load(PathBuf::new()).unwrap();

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
            return Err(activate_error.into());
        };

        match active_task {
            ActiveTask::None => {}
            ActiveTask::RegenerateCaches => cache_handling::regenerate_caches(&self.path)?,
            ActiveTask::RegenerateTagSums => stat_sums::regenerate_tag_sums(&self.path)?,

            ActiveTask::MergeTags(tag_1, tag_2) => {
                if let Err(e) = self.intr_merge_tags(tag_1, tag_2) {
                    error!("merge_tags() failed due to: {e:?}");
                    return Err(e.into());
                }
            }
            ActiveTask::RenameTag(old_name, new_name) => {
                if let Err(e) = self.intr_rename_tag(old_name, new_name) {
                    error!("rename_tag() failed due to: {e:?}");
                    return Err(e.into());
                }
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

    /// Merges `tag_1` into `tag_2`. Any existing reference to `tag_1` will be changed to `tag_2` if
    /// `tag_2` doesn't already exist in that context.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is not limited to just
    /// these cases:
    ///
    /// * The database is busy.
    /// * An io error occured.
    /// * `tag_1` or `tag_2` does not exist.
    pub fn merge_tags(&self, tag_1: u16, tag_2: u16) -> Result<()> {
        let db_status = DBStatus::activate(&self.path, ActiveTask::RegenerateTagSums)?;

        // TODO Error handling...
        if let Err(e) = self.intr_merge_tags(tag_1, tag_2) {
            error!("merge_tags() failed due to: {e:?}");
            db_status.deactivate();
            return Err(e.into());
        }

        db_status.deactivate();

        Ok(())
    }

    /// Renames `old_tag` to `new_tag`.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is not limited to just
    /// these cases:
    ///
    /// * The database is busy.
    /// * An io error occured.
    /// * `old_tag` doesn't exist.
    /// * `new_tag` already exists. (Meaning a merge is required instead.)
    pub fn rename_tag(&self, old_tag: String, new_tag: String) -> Result<()> {
        let db_status = DBStatus::activate(&self.path, ActiveTask::RegenerateTagSums)?;

        self.intr_rename_tag(old_tag, new_tag)?;

        db_status.deactivate();

        Ok(())
    }

    ///
    pub fn compress_to_image() {}
    pub fn upgrade_database() {}
}

// Private functions
impl DataBase {
    fn intr_rename_tag(&self, old_tag: String, new_tag: String) -> Result<()> {
        let mut tags = TagList::from_file(&self.path)?;
        tags.rename_tag(old_tag, new_tag)?;
        tags.save()?;
        Ok(())
    }

    fn intr_merge_tags(&self, tag_1: u16, tag_2: u16) -> Result<()> {
        // TODO Error handling...
        if let Err(e) = merge_tags(&self.path, tag_1, tag_2) {
            error!("merge_tags() failed due to: {e:?}");
            return Err(e.into());
        }

        if let Err(e) = stat_sums::regenerate_tag_sums(&self.path) {
            error!(
                "merge_tags() received {:?} when attempting to regenerate tag sums!",
                e
            );
        }

        if let Err(e) = cache_handling::regenerate_caches(&self.path) {
            error!(
                "merge_tags() received {:?} when attempting to regenerate caches!",
                e
            );
        }

        Ok(())
    }
}

//

//

// Error struct
// ###############################################################################################

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
    /// Wrapper for a unexpected io error.
    Io(io::Error),
    /// Wrapper for a unexpected walkdir error.
    WalkDir(walkdir::Error),
    /// No object exists at the provided filepath.
    PathDoesNotExist,
    /// The provided path points to a existing object, but said object is not marked as a database.
    IsNotDataBase,
    /// The database is in the middle of a unfinished operation.
    /// This should only happen if the program is terminated mid-operation.
    DataBaseBusy,
    /// The database is marked as busy, but the data required by the active task is missing.
    MissingData,
    /// The database is marked as busy, but the task id does not match any known task.
    UnknownTask,
    /// The tags file is corrupted. The file might contain duplicate ids or tag names, or a line
    /// might have a unexected format.
    CorruptedTagsFile,
    /// The provided tag name does not exist in the database.
    UnknownTag(String),
    /// The provided tag id does not exist in the database.
    UnknownTagId(u16),
    /// The proivded tag name already exists in the database.
    TagAlreadyExists,
}

/*

*/

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
            kind: match value {
                DataBasePathError::IsNotDataBase => ErrorKind::IsNotDataBase,
                DataBasePathError::DoesNotExist => ErrorKind::PathDoesNotExist,
            },
        }
    }
}

impl From<DBStatusError> for Error {
    fn from(value: DBStatusError) -> Self {
        Self {
            kind: match value {
                DBStatusError::Io(e) => ErrorKind::Io(e),
                DBStatusError::MissingData => ErrorKind::MissingData,
                DBStatusError::UnknownTask => ErrorKind::UnknownTask,
                DBStatusError::DataBaseBusy(_, _) => ErrorKind::DataBaseBusy,
            },
        }
    }
}

impl From<StatSumsError> for Error {
    fn from(value: StatSumsError) -> Self {
        Self {
            kind: match value {
                StatSumsError::Io(e) => ErrorKind::Io(e),
                StatSumsError::WalkDir(e) => ErrorKind::WalkDir(e),
            },
        }
    }
}

impl From<TagsError> for Error {
    fn from(value: TagsError) -> Self {
        Self {
            kind: match value {
                TagsError::Io(e) => ErrorKind::Io(e),
                TagsError::CorruptedTagsFile(_) => ErrorKind::CorruptedTagsFile,
                TagsError::UnknownTag(tag) => ErrorKind::UnknownTag(tag),
                TagsError::UnknownId(id) => ErrorKind::UnknownTagId(id),
                TagsError::TagAlreadyExists => ErrorKind::TagAlreadyExists,
            },
        }
    }
}

impl From<MergeTagsError> for Error {
    fn from(value: MergeTagsError) -> Self {
        match value {
            MergeTagsError::Io(e) => Self::with_kind(ErrorKind::Io(e)),
            MergeTagsError::WalkDir(e) => Self::with_kind(ErrorKind::WalkDir(e)),
            MergeTagsError::Tags(tags_error) => tags_error.into(),
        }
    }
}
