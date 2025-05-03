use lz4::Decoder;
use serialization::Deserialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Result, Seek, SeekFrom, copy},
    marker::PhantomData,
};

pub struct DBReader<T> {
    _marker: PhantomData<T>,
    pub indexes: HashMap<String, (u32, u16)>,
    temp: String,
    value: Option<File>,
}

impl<T: Deserialize> DBReader<T> {
    pub fn from(path: &str, temp: &str) -> Result<DBReader<T>> {
        let mut decoder = Decoder::new(File::open(&path)?)?;

        let mut buf = [0u8; 8];
        decoder.read_exact(&mut buf)?;
        let mut buf = vec![0u8; u64::deserialize(&buf) as usize];
        decoder.read_exact(&mut buf)?;

        let mut indexes = HashMap::new();
        let mut slice = &buf[..];
        while !slice.is_empty() {
            let (key, rest) = slice[1..].split_at(slice[0] as usize);
            let (meta, rest) = rest.split_at(6);
            indexes.insert(
                String::deserialize(key),
                (u32::deserialize(&meta[..4]), u16::deserialize(&meta[4..])),
            );
            slice = rest;
        }

        copy(&mut decoder, &mut File::create(temp)?)?;

        Ok(DBReader::<T> {
            _marker: PhantomData,
            indexes,
            temp: temp.to_owned(),
            value: Some(File::open(temp)?),
        })
    }

    pub fn get(&mut self, key: &str) -> Option<T> {
        match self.indexes.get(key) {
            Some(&(offset, lenth)) => self.read(offset as u64, lenth as usize).ok(),
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

    pub fn read(&mut self, offset: u64, lenth: usize) -> Result<T> {
        let mut file = self.value.as_ref().unwrap();
        file.seek(SeekFrom::Start(offset))?;
        let mut value = vec![0u8; lenth];
        file.read_exact(&mut value)?;
        Ok(T::deserialize(&value))
    }
}

impl<T> Drop for DBReader<T> {
    fn drop(&mut self) {
        self.value.take();
        let _ = std::fs::remove_file(&self.temp);
    }
}
