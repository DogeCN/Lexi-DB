use super::Deserialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Result, Seek, SeekFrom},
    marker::PhantomData,
    path::PathBuf,
};

pub struct DBReader<T> {
    _marker: PhantomData<T>,
    pub indexes: HashMap<String, u64>,
    offset: u64,
    file: File,
}

impl<T: Deserialize> DBReader<T> {
    pub fn from(path: &str) -> Result<DBReader<T>> {
        let path = PathBuf::from(path);
        let mut file = File::open(&path)?;

        let mut buf = [0u8; 8];
        file.read_exact(&mut buf)?;
        let lenth = u64::deserialize(&buf) as usize;
        let mut bin = vec![0u8; lenth];
        file.read_exact(&mut bin)?;

        let mut indexes = HashMap::new();
        let mut cursor = 0;
        while cursor < lenth {
            let key_len = bin[cursor] as usize;
            cursor += 1;
            let key = String::deserialize(&bin[cursor..cursor + key_len]);
            cursor += key_len;
            let offset = u64::deserialize(&bin[cursor..cursor + 8]);
            cursor += 8;

            indexes.insert(key, offset);
        }
        Ok(DBReader::<T> {
            _marker: PhantomData,
            indexes,
            offset: lenth as u64 + 8,
            file,
        })
    }

    pub fn get(&mut self, key: &str) -> Option<T> {
        match self.indexes.get(key) {
            Some(&offset) => self.read(offset).ok(),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        self.indexes.len()
    }

    pub fn keys(&self) -> Vec<&String> {
        self.indexes.keys().collect()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.indexes.contains_key(key)
    }

    pub fn read(&mut self, offset: u64) -> Result<T> {
        self.file.seek(SeekFrom::Start(self.offset + offset))?;
        let mut buf = [0u8; 2];
        self.file.read_exact(&mut buf)?;
        let len = u16::from_le_bytes(buf) as usize;
        let mut value = vec![0u8; len];
        self.file.read_exact(&mut value)?;
        Ok(T::deserialize(&value))
    }
}
