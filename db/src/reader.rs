use lz4::Decoder;
use serialization::Deserialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{Result, Seek, SeekFrom, copy},
    marker::PhantomData,
};

pub struct DBReader<T> {
    _marker: PhantomData<T>,
    pub name: String,
    pub name_zh: String,
    pub indexes: HashMap<String, usize>,
    temp: String,
    value: Option<File>,
}

impl<T: Deserialize> DBReader<T> {
    pub fn from(path: &str, temp: &str) -> Result<DBReader<T>> {
        let mut decoder = Decoder::new(File::open(&path)?)?;
        let mut indexes = HashMap::new();

        let name = String::deserialize(&mut decoder)?;
        let name_zh = String::deserialize(&mut decoder)?;

        for _ in 0..usize::deserialize(&mut decoder)? {
            indexes.insert(
                String::deserialize(&mut decoder)?,
                usize::deserialize(&mut decoder)?,
            );
        }

        copy(&mut decoder, &mut File::create(temp)?)?;

        Ok(DBReader::<T> {
            _marker: PhantomData,
            name,
            name_zh,
            indexes,
            temp: temp.to_owned(),
            value: Some(File::open(temp)?),
        })
    }

    pub fn get(&mut self, key: &str) -> Option<T> {
        match self.indexes.get(key) {
            Some(&offset) => self.read(offset as u64).ok(),
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
        let mut file = self.value.as_ref().unwrap();
        file.seek(SeekFrom::Start(offset))?;
        Ok(T::deserialize(&mut file)?)
    }
}

impl<T> Drop for DBReader<T> {
    fn drop(&mut self) {
        self.value.take();
        let _ = std::fs::remove_file(&self.temp);
    }
}
