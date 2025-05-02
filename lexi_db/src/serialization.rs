pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

pub trait Deserialize: Sized {
    fn deserialize(data: &[u8]) -> Self;
}

impl Serialize for String {
    fn serialize(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl Serialize for &String {
    fn serialize(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl Deserialize for String {
    fn deserialize(data: &[u8]) -> Self {
        String::from_utf8_lossy(data).into_owned()
    }
}

impl Serialize for u64 {
    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Deserialize for u64 {
    fn deserialize(data: &[u8]) -> Self {
        let mut buf = [0; 8];
        buf.copy_from_slice(data);
        u64::from_le_bytes(buf)
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self) -> Vec<u8> {
        self.iter()
            .flat_map(|item| item.serialize().into_iter().chain(std::iter::once(0u8)))
            .collect()
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize(data: &[u8]) -> Self {
        data.split(|b| b.eq(&0))
            .map(|data| T::deserialize(data))
            .collect()
    }
}
