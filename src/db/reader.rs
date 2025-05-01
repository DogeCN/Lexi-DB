use super::Deserialize;
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{Read, Seek, SeekFrom},
    marker::PhantomData,
    path::PathBuf,
    result::Result,
};

pub struct DBReader<T> {
    _marker: PhantomData<T>,
    pub indexes: HashMap<String, u64>,
    offset: u64,
    file: File,
}

impl<T: Deserialize> DBReader<T> {
    pub fn from(path: &str) -> Result<DBReader<T>, Box<dyn Error>> {
        let path = PathBuf::from(path);
        let mut file = File::open(&path)?;

        let mut buf = [0u8; 8];
        file.read_exact(&mut buf)?;
        let lenth = u64::from_le_bytes(buf) as usize;
        let mut bin = vec![0u8; lenth];
        file.read_exact(&mut bin)?;

        let mut indexes = HashMap::new();
        let mut cursor = 0;
        while cursor < lenth {
            let key_len = bin[cursor] as usize;
            cursor += 1;
            let key = String::from_utf8(bin[cursor..cursor + key_len].to_vec())?;
            cursor += key_len;
            let offset = u64::from_le_bytes(bin[cursor..cursor + 8].try_into()?);
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

    pub fn read(&mut self, offset: u64) -> Result<T, Box<dyn Error>> {
        self.file.seek(SeekFrom::Start(self.offset + offset))?;
        let mut buf = [0u8; 2];
        self.file.read_exact(&mut buf)?;
        let len = u16::from_le_bytes(buf) as usize;
        let mut value = vec![0u8; len];
        self.file.read_exact(&mut value)?;
        Ok(T::deserialize(&value))
    }
}
