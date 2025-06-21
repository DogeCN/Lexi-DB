use serialization::Deserialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{Result, Seek, SeekFrom, copy},
    marker::PhantomData,
    path::PathBuf,
    sync::Arc,
};
use xz2::read::XzDecoder;

pub struct DBReader<T> {
    _marker: PhantomData<T>,
    pub name: String,
    pub name_zh: String,
    pub indexes: HashMap<Arc<String>, usize>,
    decoder: Option<XzDecoder<File>>,
    temp: PathBuf,
    value: Option<File>,
}

impl<T: Deserialize> DBReader<T> {
    pub fn from(path: &str, temp: &str) -> Result<DBReader<T>> {
        let mut decoder = XzDecoder::new(File::open(&path)?);
        let name = String::deserialize(&mut decoder)?;
        let name_zh = String::deserialize(&mut decoder)?;
        Ok(DBReader::<T> {
            _marker: PhantomData,
            name,
            name_zh,
            indexes: HashMap::new(),
            decoder: Some(decoder),
            temp: PathBuf::from(temp),
            value: None,
        })
    }

    pub fn load(&mut self) -> Result<()> {
        let mut decoder = self.decoder.take().unwrap();
        match self.load_with_decoder(&mut decoder) {
            Err(e) => {
                self.decoder = Some(decoder);
                Err(e)
            }
            _ => Ok(()),
        }
    }

    fn load_with_decoder(&mut self, decoder: &mut XzDecoder<File>) -> Result<()> {
        for _ in 0..usize::deserialize(decoder)? {
            self.indexes.insert(
                Arc::new(String::deserialize(decoder)?),
                usize::deserialize(decoder)?,
            );
        }
        if !self.temp.exists() {
            copy(decoder, &mut File::create(&self.temp)?)?;
        }
        self.value = Some(File::open(&self.temp)?);
        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Option<T> {
        match self.indexes.get(&key.to_owned()) {
            Some(&offset) => self.read(offset as u64).ok(),
            _ => None,
        }
    }

    pub fn filter_keys(&self, text: &str, seps: &[char]) -> Vec<String> {
        let mut result = Vec::new();
        for &sep in seps {
            let words: Vec<&str> = text.split(sep).collect();
            for k in self.keys() {
                let k = k.as_str();
                let keys: Vec<&str> = k.split(sep).collect();
                if text != k && keys.len() >= words.len() && words.iter().all(|p| keys.contains(p))
                {
                    result.push(k.to_owned());
                }
            }
        }
        result
    }

    pub fn len(&self) -> usize {
        self.indexes.len()
    }

    pub fn keys(&self) -> Vec<Arc<String>> {
        self.indexes.keys().cloned().collect()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.indexes.contains_key(&key.to_owned())
    }

    pub fn read(&mut self, offset: u64) -> Result<T> {
        let mut file = self.value.as_ref().unwrap();
        file.seek(SeekFrom::Start(offset))?;
        Ok(T::deserialize(&mut file)?)
    }
}
