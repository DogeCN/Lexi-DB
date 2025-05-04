#![cfg(test)]
use db::{DBCreator, DBReader};

#[test]
fn test_db() {
    let mut db: DBCreator<String> = DBCreator::new("test.db");
    db.insert("key1", "value1");
    db.export().unwrap();

    let mut reader: DBReader<String> = DBReader::from("test.db", "test.values").unwrap();

    assert_eq!(reader.get("key1"), Some("value1".to_owned()));
    assert_eq!(reader.get("key2"), None);
}
