use std::{
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};

use crate::db_path::DataBasePath;

//

//

#[derive(Debug)]
pub enum ActiveTask {
    None,
    RegenerateCaches,
    RegenerateTagSums,
    MergeTags(u16, u16),
    RenameTag(String, String),
    RemoveTag(u16),
}

impl ActiveTask {
    fn parse(data_str: &str) -> Result<ActiveTask> {
        let mut parts = data_str.split('|');
        match parts.next().ok_or(DBStatusError::UnknownTask)? {
            "0" => Ok(ActiveTask::None),
            "1" => Ok(ActiveTask::RegenerateCaches),
            "2" => Ok(ActiveTask::RegenerateTagSums),
            "3" => {
                let (tag_1, tag_2) = {
                    let data: Vec<u16> = parts
                        .next()
                        .ok_or(DBStatusError::CorruptedData)?
                        .split(|c: char| !c.is_ascii_digit())
                        .filter(|s| s.is_empty())
                        // The parse() below will never fail due to the way the string is created.
                        .map(|s| s.parse::<u16>().map_err(|_| DBStatusError::CorruptedData))
                        .collect::<Result<_>>()?;

                    if data.len() != 2 {
                        log::error!("db_status::ActiveTask::parse(): Failed to parse MergeTag. Found {} arguments but expected 2!", data.len());
                        return Err(DBStatusError::CorruptedData);
                    }
                    (data[0], data[1])
                };

                Ok(ActiveTask::MergeTags(tag_1, tag_2))
            }
            "4" => {
                let (old_tag, new_tag) = {
                    let data: Vec<&str> = parts
                        .next()
                        .ok_or(DBStatusError::CorruptedData)?
                        .split(' ')
                        .collect();
                    if data.len() != 2 {
                        log::error!("db_status::ActiveTask::parse(): Failed to parse RenameTag. Found {} arguments but expected 2!", data.len());
                        return Err(DBStatusError::CorruptedData);
                    }
                    (data[0], data[1])
                };
                Ok(ActiveTask::RenameTag(
                    old_tag.to_string(),
                    new_tag.to_string(),
                ))
            }
            "5" => {
                let data = parts.next().ok_or(DBStatusError::CorruptedData)?;
                let Ok(tag_id) = data.parse::<u16>() else {
                    log::error!("db_status::ActiveTask::parse(): Failed to parse RemoveTag. Found unepected characters in tag id! [{}]", data);
                    return Err(DBStatusError::CorruptedData);
                };
                Ok(ActiveTask::RemoveTag(tag_id))
            }
            _ => Err(DBStatusError::UnknownTask),
        }
    }

    //

    //

    fn into_data_string(self) -> String {
        let task_id = self.task_id();
        let task_data = match self {
            Self::None => "",
            Self::RegenerateCaches => "",
            Self::RegenerateTagSums => "",
            Self::MergeTags(s1, s2) => &format!("{} {}", s1, s2),
            Self::RenameTag(s1, s2) => &format!("{} {}", s1, s2),
            Self::RemoveTag(tag_id) => &format!("{}", tag_id),
        };
        format!("{}|{}", task_id, task_data)
    }

    //

    //

    fn task_id(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::RegenerateCaches => 1,
            Self::RegenerateTagSums => 2,
            Self::MergeTags(_, _) => 3,
            Self::RenameTag(_, _) => 4,
            Self::RemoveTag(_) => 5,
        }
    }
}

//

//

#[derive(Debug)]
pub enum DBStatusError {
    Io(io::Error),
    DataBaseBusy(ActiveTask, DBStatus),
    CorruptedData,
    UnknownTask,
}

impl From<io::Error> for DBStatusError {
    fn from(err: io::Error) -> Self {
        DBStatusError::Io(err)
    }
}

//

//

type Result<T> = std::result::Result<T, DBStatusError>;

const STATUSFILENAME: &str = ".status.txt";

//

//

#[derive(Debug)]
pub struct DBStatus {
    status_path: PathBuf,
}

impl DBStatus {
    /// Returns whether the database is locked or not.
    pub fn is_locked(db_path: &DataBasePath) -> bool {
        let filepath = db_path.root().join(STATUSFILENAME);
        filepath.exists()
    }

    /// Attempts to lock the database. If the database is already locked a
    /// DBStatusError::DataBaseBusy is returner.
    pub fn lock(db_path: &DataBasePath, task: ActiveTask) -> Result<DBStatus> {
        let filepath = db_path.root().join(STATUSFILENAME);

        let db_status = DBStatus {
            status_path: filepath.clone(),
        };
        match File::create_new(&filepath) {
            Ok(mut file) => {
                write!(file, "{}", task.into_data_string())?;
                Ok(db_status)
            }
            Err(_) => {
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

    //

    //

    /// Releases the lock of the database.
    pub fn unlock(self) {
        let _removal_result = std::fs::remove_file(self.status_path);
    }
}
