use lz4::EncoderBuilder;
use serialization::Serialize;
use std::{
    collections::HashMap,
    fs::{File, remove_file},
    io::{Result, Write, copy},
    path::PathBuf,
};

pub struct DBCreator<T> {
    path: PathBuf,
    data: HashMap<String, T>,
}

impl<T: Serialize> DBCreator<T> {
    pub fn new(path: &str) -> DBCreator<T> {
        DBCreator {
            path: PathBuf::from(path),
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: impl Into<T>) {
        self.data.insert(key.to_owned(), value.into());
    }

    pub fn export(&self) -> Result<()> {
        let keys = self.path.with_extension("keys");
        let values = self.path.with_extension("values");
        {
            let mut file_key = File::create(&keys)?;
            let mut file_values = File::create(&values)?;
            let mut total: u32 = 0;

            for (key, value) in &self.data {
                let key = key.as_bytes();
                let mut buf: Vec<u8> = Vec::new();
                buf.push(key.len() as u8);
                buf.extend(key);
                buf.extend(total.to_le_bytes());
                let value: Vec<u8> = value.serialize();
                buf.extend((value.len() as u16).to_le_bytes());
                file_key.write_all(&buf)?;
                total += value.len() as u32;
                file_values.write_all(&value)?;
            }
        }
        {
            let mut encoder = EncoderBuilder::new()
                .level(4)
                .build(File::create(&self.path)?)?;
            let mut reader = File::open(&keys)?;
            encoder.write_all(&reader.metadata()?.len().to_le_bytes())?;
            copy(&mut reader, &mut encoder)?;
            let mut reader = File::open(&values)?;
            copy(&mut reader, &mut encoder)?;
            encoder.flush()?;
        }
        remove_file(&keys)?;
        remove_file(&values)?;
        Ok(())
    }
}
