pub struct DataEntry {
    pub hour: u8,
    pub mental_score: u8,
    pub physical_score: u8,
    pub tags: Vec<u16>,
}

impl DataEntry {
    pub fn new(hour: u8, mental_score: u8, physical_score: u8, tags: Vec<u16>) -> DataEntry {
        DataEntry {
            hour,
            mental_score,
            physical_score,
            tags,
        }
    }
}
