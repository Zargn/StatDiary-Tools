use std::{
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};

use crate::db_status;

#[derive(Debug)]
pub enum ActiveTask {
    None,
    RegenerateCaches,
    MergeTags(String, String),
    RenameTag(String, String),
}

impl ActiveTask {
    fn parse(data_str: &str) -> Result<ActiveTask> {
        let mut parts = data_str.split('|');
        match parts.next().ok_or(DBStatusError::UnknownTask)? {
            "0" => Ok(ActiveTask::None),
            "1" => Ok(ActiveTask::RegenerateCaches),
            _ => Err(DBStatusError::UnknownTask),
        }
    }

    fn to_data_string(self) -> String {
        let task_id = self.task_id();
        let task_data = match self {
            Self::None => "",
            Self::RegenerateCaches => "",
            Self::MergeTags(s1, s2) => &format!("{} {}", s1, s2),
            Self::RenameTag(s1, s2) => &format!("{} {}", s1, s2),
        };
        format!("{}|{}", task_id, task_data)
    }

    fn task_id(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::RegenerateCaches => 1,
            Self::MergeTags(_, _) => 2,
            Self::RenameTag(_, _) => 3,
        }
    }
}

#[derive(Debug)]
pub enum DBStatusError {
    InvalidDataBasePath,
    IoError(io::Error),
    DataBaseBusy(ActiveTask, DBStatus),
    UnknownTask,
}

impl From<io::Error> for DBStatusError {
    fn from(err: io::Error) -> Self {
        DBStatusError::IoError(err)
    }
}

type Result<T> = std::result::Result<T, DBStatusError>;

const STATUSFILENAME: &str = ".status.txt";

#[derive(Debug)]
pub struct DBStatus {
    status_path: PathBuf,
}

impl DBStatus {
    pub fn activate(db_path: PathBuf, task: ActiveTask) -> Result<DBStatus> {
        if !db_path.exists() {
            return Err(DBStatusError::InvalidDataBasePath);
        }
        let filepath = db_path.join(STATUSFILENAME);

        let db_status = DBStatus {
            status_path: filepath.clone(),
        };
        match File::create_new(&filepath) {
            Ok(mut file) => {
                write!(file, "{}", task.to_data_string())?;
                Ok(db_status)
            }
            Err(e) => {
                let mut data_str = String::new();
                let mut file = File::open(filepath)?;
                file.read_to_string(&mut data_str)?;
                Err(DBStatusError::DataBaseBusy(
                    ActiveTask::parse(&data_str)?,
                    db_status,
                ))
            }
        }
    }

    pub fn deactivate(self) {
        let _removal_result = std::fs::remove_file(self.status_path);
    }
}
