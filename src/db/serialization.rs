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

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for s in self {
            result.extend(s.serialize());
            result.push(0);
        }
        result
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize(data: &[u8]) -> Self {
        let mut result = Vec::new();
        let mut buf = Vec::new();
        for &b in data {
            match b {
                0 => {
                    result.push(T::deserialize(&buf));
                    buf.clear();
                }
                _ => buf.push(b),
            }
        }
        result
    }
}
