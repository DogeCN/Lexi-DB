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
    name: String,
    name_zh: String,
}

impl<T: Serialize> DBCreator<T> {
    pub fn new(path: &str, name: &str, name_zh: &str) -> DBCreator<T> {
        DBCreator {
            path: PathBuf::from(path),
            data: HashMap::new(),
            name: name.to_owned(),
            name_zh: name_zh.to_owned(),
        }
    }

    pub fn insert(&mut self, key: &str, value: impl Into<T>) {
        self.data.insert(key.to_owned(), value.into());
    }

    pub fn export(&self) -> Result<()> {
        let keys = self.path.with_extension("keys");
        let values = self.path.with_extension("values");
        let mut count: usize = 0;
        {
            let mut file_key = File::create(&keys)?;
            let mut file_values = File::create(&values)?;
            let mut total: usize = 0;

            for (key, value) in &self.data {
                let mut buf: Vec<u8> = Vec::new();
                buf.extend(key.serialize());
                buf.extend(total.serialize());
                file_key.write_all(&buf)?;

                let buf: Vec<u8> = value.serialize();
                total += buf.len();
                file_values.write_all(&buf)?;
                count += 1;
            }
        }
        {
            let mut encoder = EncoderBuilder::new()
                .level(4)
                .build(File::create(&self.path)?)?;
            encoder.write_all(&self.name.serialize())?;
            encoder.write_all(&self.name_zh.serialize())?;
            encoder.write_all(&count.serialize())?;
            copy(&mut File::open(&keys)?, &mut encoder)?;
            copy(&mut File::open(&values)?, &mut encoder)?;
            encoder.flush()?;
        }
        remove_file(&keys)?;
        remove_file(&values)?;
        Ok(())
    }
}
