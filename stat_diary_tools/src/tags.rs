use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufWriter, Read, Write},
};

use crate::{db_path::DataBasePath, utilities::read_lines};

#[derive(Debug)]
pub enum TagsError {
    Io(io::Error),
    CorruptedTagsFile(String),
    UnknownTag(String),
    UnknownId(u16),
    TagAlreadyExists,
}

impl From<io::Error> for TagsError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

type Result<T> = std::result::Result<T, TagsError>;

/// This is a in-memory representation of a tag list.
/// It provides a variety of functions, including getting the tag name from a id, or a id from a
/// tag name.
/// Furthermore it enables modifications to the tags list, like deleting, renaming, or merging
/// tags.
pub struct TagList {
    id_str_map: HashMap<u16, String>,
    str_id_map: HashMap<String, u16>,
    removed_ids: Vec<u16>,
    db_path: DataBasePath,
}

impl TagList {
    /// Attempts to create a TagList instance using the provided database.
    /// If said database is missing a tags.txt file then this will fail.
    /// It will also fail if said file doesn't follow the expected format of "{tag_id} {tag_name}".
    pub fn from_file(db_path: &DataBasePath) -> Result<TagList> {
        let filepath = db_path.root().join("tags.txt");

        let mut id_str_map = HashMap::new();
        let mut str_id_map = HashMap::new();
        for line in read_lines(filepath)? {
            let mut parts = line.split(' ');
            let (id, tag) = (
                parts
                    .next()
                    .ok_or(TagsError::CorruptedTagsFile(line.clone()))?
                    .parse::<u16>()
                    .map_err(|_| TagsError::CorruptedTagsFile(line.clone()))?,
                parts
                    .next()
                    .ok_or(TagsError::CorruptedTagsFile(line.clone()))?,
            );

            if str_id_map.insert(tag.to_string(), id).is_some() {
                return Err(TagsError::CorruptedTagsFile(
                    "Duplicate tags found in tags file!".to_string(),
                ));
            }

            if id_str_map.insert(id, tag.to_string()).is_some() {
                return Err(TagsError::CorruptedTagsFile(
                    "Duplicate tag ids found in tags file!".to_string(),
                ));
            }
        }

        Ok(TagList {
            id_str_map,
            str_id_map,
            removed_ids: Vec::new(),
            db_path: db_path.clone(),
        })
    }

    //

    //

    /// Returns the id linked to the provided tag name.
    /// If the tag name doesn't exist a TagsError::UnknownTag is returned.
    pub fn get_id(&self, tag: &str) -> Result<&u16> {
        self.str_id_map
            .get(tag)
            .ok_or(TagsError::UnknownTag(tag.to_string()))
    }

    //

    //

    /// Returns the tag name linked to the provided tag id.
    /// If the tag id doesn't exist a TagsError::UnknownId is returned.
    pub fn get_tag(&self, tag_id: u16) -> Result<&String> {
        self.id_str_map
            .get(&tag_id)
            .ok_or(TagsError::UnknownId(tag_id))
    }

    //

    //

    /// Returns if the provided tag_id exists or not.
    pub fn tag_exists(&self, tag_id: u16) -> bool {
        self.id_str_map.contains_key(&tag_id)
    }

    //

    //

    /// Removes the provided tag_id and its linked tag name from the tag list.
    /// Will fail with a TagsError if the provided tag doesn't exist.
    pub fn remove_tag(&mut self, tag_id: u16) -> Result<&mut Self> {
        let tag_str = self
            .id_str_map
            .remove(&tag_id)
            .ok_or(TagsError::UnknownId(tag_id))?;
        self.str_id_map
            .remove(&tag_str)
            .ok_or(TagsError::UnknownTag(tag_str))?;

        self.removed_ids.push(tag_id);
        Ok(self)
    }

    //

    //

    /// Attempts to rename old_tag to new_tag while keeping the same tag_id.
    /// If old_tag doesn't exist a TagsError::UnknownTag will be returned.
    /// If new_tag already exists this will fail with a TagsError::TagAlreadyExists.
    pub fn rename_tag(&mut self, old_tag: String, new_tag: String) -> Result<&mut Self> {
        if self.str_id_map.contains_key(&new_tag) {
            return Err(TagsError::TagAlreadyExists);
        }

        let Some(tag_id) = self.str_id_map.remove(&old_tag) else {
            return Err(TagsError::UnknownTag(old_tag));
        };

        self.str_id_map.insert(new_tag.clone(), tag_id);
        *self.id_str_map.entry(tag_id).or_default() = new_tag;

        Ok(self)
    }

    //

    //

    /// Attempts to merge the tag_1 into tag_2.
    /// If any of the tags doesn't exist then a TagsError::UnknownId is returned.
    /// Otherwise tag_1 is removed from the tag list, leaving only tag_2.
    pub fn merge_tags(&mut self, tag_1: u16, tag_2: u16) -> Result<&mut Self> {
        let _ = self.get_tag(tag_1)?;
        let _ = self.get_tag(tag_2)?;

        self.remove_tag(tag_1)?;
        Ok(self)
    }

    //

    //

    /// Saves the tags list to the file it was originally read from.
    /// The original file is overwritten by the new.
    ///
    /// Writes to a .tmp file which when completed is swapped with the original file, ensuring that
    /// no data is lost in the event of the program stopping mid-write.
    pub fn save(&mut self) -> Result<()> {
        let tmp_path = self.db_path.root().join("tags.txt.tmp");
        let filepath = self.db_path.root().join("tags.txt");

        let mut writer = BufWriter::new(File::create(&tmp_path)?);

        for (id, tag) in self.id_str_map.iter() {
            writeln!(writer, "{} {}", id, tag)?;
        }

        writer.flush()?;
        fs::rename(tmp_path, filepath)?;

        if !self.removed_ids.is_empty() {
            self.save_removed_ids()?;
        }

        Ok(())
    }

    //

    //

    /// Adds any removed id to a reclaimed.tags file.
    ///
    /// Writes to a .tmp file which when completed is swapped with the original file, ensuring that
    /// no data is lost in the event of the program stopping mid-write.
    fn save_removed_ids(&self) -> Result<()> {
        let tmp_path = self.db_path.root().join("reclaimed.tags.tmp");
        let filepath = self.db_path.root().join("reclaimed.tags");

        let mut writer = BufWriter::new(File::create(&tmp_path)?);

        if let Ok(mut file) = File::open(&filepath) {
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            writer.write_all(&bytes)?;
        }

        for tag_id in &self.removed_ids {
            writer.write_all(&tag_id.to_be_bytes())?;
        }

        writer.flush()?;
        fs::rename(tmp_path, filepath)?;

        Ok(())
    }
}
