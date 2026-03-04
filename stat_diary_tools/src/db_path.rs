use std::{
    ffi::{c_char, CStr},
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum DataBasePathError {
    DoesNotExist,
    IsNotDataBase,
}

#[derive(Debug)]
pub enum PtrToDBPathError {
    NullPtr,
    InvalidUTF8,
    DataBasePath(DataBasePathError),
}

impl From<DataBasePathError> for PtrToDBPathError {
    fn from(dbp_error: DataBasePathError) -> Self {
        PtrToDBPathError::DataBasePath(dbp_error)
    }
}

#[derive(Debug, Clone)]
pub struct DataBasePath {
    db_root: PathBuf,
}

impl DataBasePath {
    pub unsafe fn try_ptr_to_data_base_path(
        ptr: *const c_char,
    ) -> Result<DataBasePath, PtrToDBPathError> {
        if ptr.is_null() {
            return Err(PtrToDBPathError::NullPtr);
        }
        let cstr = unsafe { CStr::from_ptr(ptr) };
        let Ok(str) = cstr.to_str() else {
            return Err(PtrToDBPathError::InvalidUTF8);
        };
        Ok(DataBasePath::new(Path::new(str).to_path_buf())?)
    }

    pub fn new(db_path: PathBuf) -> Result<DataBasePath, DataBasePathError> {
        let Ok(true) = db_path.try_exists() else {
            return Err(DataBasePathError::DoesNotExist);
        };

        let db_marker_path = db_path.join(".db_marker");
        let Ok(true) = db_marker_path.try_exists() else {
            return Err(DataBasePathError::IsNotDataBase);
        };

        Ok(DataBasePath { db_root: db_path })
    }

    pub fn root(&self) -> &Path {
        &self.db_root
    }

    pub fn data(&self) -> PathBuf {
        self.db_root.join("data")
    }

    pub fn stat_sums(&self) -> PathBuf {
        self.db_root.join("stat_sums")
    }
}
