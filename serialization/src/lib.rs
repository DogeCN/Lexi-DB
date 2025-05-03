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

macro_rules! impl_serialize_for_int {
    ($($t:ty),*) => {
        $(
            impl Serialize for $t {
                fn serialize(&self) -> Vec<u8> {
                    self.to_le_bytes().to_vec()
                }
            }
            impl Deserialize for $t {
                fn deserialize(data: &[u8]) -> Self {
                    <$t>::from_le_bytes(data.try_into().unwrap())
                }
            }
        )*
    };
}

impl_serialize_for_int!(u64, u32, u16);

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self) -> Vec<u8> {
        self.iter()
            .map(|item| item.serialize())
            .collect::<Vec<_>>()
            .join(&[0u8][..])
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize(data: &[u8]) -> Self {
        data.split(|b| b.eq(&0))
            .map(|data| T::deserialize(data))
            .collect()
    }
}
