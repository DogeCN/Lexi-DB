use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Result, Seek, SeekFrom};
use std::path::PathBuf;

pub struct DBReader {
    path: PathBuf,
    keys: HashMap<String, u64>, // 存储键及其对应的值偏移量
    values_file: File,          // 值文件的句柄
}

impl DBReader {
    pub fn new(path: &str) -> Result<DBReader> {
        let path = PathBuf::from(path);
        let mut file = BufReader::new(File::open(&path)?);

        // 读取文件元数据（跳过长度字段）
        let mut len_buf = [0u8; 8];
        file.read_exact(&mut len_buf)?;

        // 解析键文件
        let mut keys = HashMap::new();
        let mut key_len_buf = [0u8; 1];
        while file.read_exact(&mut key_len_buf).is_ok() {
            let key_len = key_len_buf[0];
            let mut key_buf = vec![0u8; key_len as usize];
            file.read_exact(&mut key_buf)?;
            let key = String::from_utf8(key_buf).unwrap();

            let mut offset_buf = [0u8; 8];
            file.read_exact(&mut offset_buf)?;
            let offset = u64::from_le_bytes(offset_buf);

            keys.insert(key, offset);
        }

        // 打开值文件
        let values_file = File::open(&path)?;

        Ok(DBReader {
            path,
            keys,
            values_file,
        })
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        if let Some(&offset) = self.keys.get(key) {
            let mut file = BufReader::new(&self.values_file);
            file.seek(SeekFrom::Start(offset)).unwrap();

            // 读取值的长度
            let mut len_buf = [0u8; 2];
            file.read_exact(&mut len_buf).unwrap();
            let value_len = u16::from_le_bytes(len_buf) as usize;

            // 读取值的内容
            let mut value_buf = vec![0u8; value_len];
            file.read_exact(&mut value_buf).unwrap();

            return Some(String::from_utf8(value_buf).unwrap());
        }
        None
    }
}
