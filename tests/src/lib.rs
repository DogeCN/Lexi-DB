#![cfg(test)]
use db::{DBCreator, DBReader};

#[test]
fn test_db() {
    let mut db: DBCreator<String> = DBCreator::new("test.db");
    for i in 0..100 {
        db.insert(&format!("key{}", i), format!("value{}", i));
    }
    db.export().unwrap();

    let mut reader: DBReader<String> = DBReader::from("test.db", "test.values").unwrap();

    assert_eq!(reader.get("key0"), Some("value0".to_owned()));
    assert_eq!(reader.get("key"), None);
}
