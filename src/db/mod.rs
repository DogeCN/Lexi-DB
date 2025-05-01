mod creator;
mod reader;
mod serialization;

pub use creator::DBCreator;
pub use reader::DBReader;
pub use serialization::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db() {
        let path = "tests/test_db.db";
        let mut db = DBCreator::new(path);
        db.insert("key1", "value1".to_owned());
        db.insert("key2", "value2".to_owned());
        db.export().unwrap();

        let mut reader = DBReader::from(path).unwrap();
        assert_eq!(reader.get("key1"), Some("value1".to_owned()));
        assert_eq!(reader.get("key2"), Some("value2".to_owned()));
        assert_eq!(reader.get("key3"), None);
    }
}
