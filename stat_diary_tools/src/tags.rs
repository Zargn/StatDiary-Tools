use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

use crate::read_lines;

#[derive(Debug)]
pub enum DBError {
    IoError(io::Error),
    CorruptedTagsFile(String),
    UnknownTag(String),
    UnknownId(u16),
    TagAlreadyExists,
}

impl From<io::Error> for DBError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl DBError {
    pub fn into_code(self) -> i32 {
        match self {
            Self::IoError(_) => 1,
            Self::CorruptedTagsFile(_) => 2,
            Self::UnknownTag(_) => 3,
            Self::UnknownId(_) => 4,
            Self::TagAlreadyExists => 5,
        }
    }
}

type Result<T> = std::result::Result<T, DBError>;

pub struct TagList {
    id_str_map: HashMap<u16, String>,
    str_id_map: HashMap<String, u16>,
}

impl TagList {
    pub fn from_file(db_path: &Path) -> Result<TagList> {
        let filepath = db_path.join("tags.txt");

        let mut id_str_map = HashMap::new();
        let mut str_id_map = HashMap::new();
        for line in read_lines(filepath)? {
            let mut parts = line.split(' ');
            let (id, tag) = (
                parts
                    .next()
                    .ok_or(DBError::CorruptedTagsFile(line.clone()))?
                    .parse::<u16>()
                    .map_err(|_| DBError::CorruptedTagsFile(line.clone()))?,
                parts
                    .next()
                    .ok_or(DBError::CorruptedTagsFile(line.clone()))?,
            );

            if str_id_map.insert(tag.to_string(), id).is_some() {
                return Err(DBError::CorruptedTagsFile(
                    "Duplicate tags found in tags file!".to_string(),
                ));
            }

            if id_str_map.insert(id, tag.to_string()).is_some() {
                return Err(DBError::CorruptedTagsFile(
                    "Duplicate tag ids found in tags file!".to_string(),
                ));
            }
        }
        Ok(TagList {
            id_str_map,
            str_id_map,
        })
    }

    //

    //

    pub fn get_id(&self, tag: &str) -> Result<&u16> {
        self.str_id_map
            .get(tag)
            .ok_or(DBError::UnknownTag(tag.to_string()))
    }

    //

    //

    pub fn get_tag(&self, tag_id: u16) -> Result<&String> {
        self.id_str_map
            .get(&tag_id)
            .ok_or(DBError::UnknownId(tag_id))
    }

    //

    //

    pub fn rename_tag(&mut self, old_tag: String, new_tag: String) -> Result<()> {
        println!("str-id map: \n{:?}\n\n", self.str_id_map);
        println!("id-str map: \n{:?}\n\n", self.id_str_map);

        println!("old-tag: {}, new-tag: {}", old_tag, new_tag);
        if self.str_id_map.contains_key(&new_tag) {
            return Err(DBError::TagAlreadyExists);
        }

        let Some(tag_id) = self.str_id_map.remove(&old_tag) else {
            return Err(DBError::UnknownTag(old_tag));
        };

        self.str_id_map.insert(new_tag.clone(), tag_id);
        *self.id_str_map.entry(tag_id).or_default() = new_tag;

        Ok(())
    }

    //

    //

    pub fn save_to_file(self, db_path: &Path) -> Result<()> {
        let filepath = db_path.join("tags.txt");

        let mut writer = BufWriter::new(File::create(filepath)?);

        for (id, tag) in self.id_str_map.iter() {
            writeln!(writer, "{} {}", id, tag)?;
        }

        writer.flush()?;
        Ok(())
    }
}
