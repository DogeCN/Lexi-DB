use std::collections::HashMap;
use std::fs::{OpenOptions, remove_file};
use std::io::{BufWriter, Result, Write, copy};
use std::path::PathBuf;

pub struct DBCreator {
    path: PathBuf,
    data: HashMap<String, String>,
}

impl DBCreator {
    pub fn new(name: &str) -> DBCreator {
        DBCreator {
            path: PathBuf::from(name).with_extension("db"),
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_owned(), value.to_owned());
    }

    pub fn export(&self) -> Result<()> {
        let keys = self.path.with_extension("keys");
        let values = self.path.with_extension("values");
        let mut option = OpenOptions::new();
        option.write(true).truncate(true).create(true);
        {
            let mut file_key = option.open(&keys)?;
            let mut file_values = option.open(&values)?;

            let mut total: u64 = 0;

            for (key, value) in &self.data {
                let key = key.as_bytes();
                let mut buf: Vec<u8> = Vec::new();
                buf.push(key.len() as u8);
                buf.extend(key);
                buf.extend(total.to_le_bytes());
                file_key.write_all(&buf)?;

                let value = value.as_bytes();
                let mut buf: Vec<u8> = Vec::new();
                buf.extend((value.len() as u16).to_le_bytes());
                buf.extend(value);
                total += buf.len() as u64;
                file_values.write_all(&buf)?;
            }
        }
        {
            let mut file_keys = OpenOptions::new().read(true).open(&keys)?;
            let mut file_values = OpenOptions::new().read(true).open(&values)?;

            let mut file = BufWriter::new(option.open(&self.path)?);
            file.write_all(&(self.path.metadata()?.len() as u64).to_le_bytes())?;
            copy(&mut file_keys, &mut file)?;
            copy(&mut file_values, &mut file)?;
            file.flush()?;
        }
        remove_file(&keys)?;
        remove_file(&values)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut db = DBCreator::new("test");
        for i in 0..100 {
            db.insert(&format!("key{}", &i), &format!("value{}", &i));
        }
        db.export().unwrap();
        assert!(std::fs::exists(&db.path).unwrap());
    }
}
