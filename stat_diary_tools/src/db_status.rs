use std::path::PathBuf;

pub enum ActiveTask {}

pub struct DBStatus {
    db_path: PathBuf,
    active: bool,
}

impl DBStatus {
    pub fn new(db_path: PathBuf) -> Result<DBStatus, ActiveTask> {
        todo!();
    }

    pub fn activate(&mut self, task: ActiveTask) {
        todo!();
    }

    pub fn deactivate(self) {
        todo!();
    }
}
