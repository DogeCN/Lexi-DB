pub use std::io::{Read, Result};

pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

pub trait Deserialize: Sized {
    fn deserialize<R: Read>(r: &mut R) -> Result<Self>;
}

impl Serialize for String {
    fn serialize(&self) -> Vec<u8> {
        let mut buf = self.len().serialize();
        buf.extend(self.as_bytes());
        buf
    }
}

impl Serialize for &String {
    fn serialize(&self) -> Vec<u8> {
        self.to_owned().serialize()
    }
}

impl Deserialize for String {
    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let mut buf = vec![0u8; usize::deserialize(r)?];
        r.read_exact(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }
}

macro_rules! impl_uint {
    ($($t:ty),*) => {
        $(

            impl Serialize for $t {
                fn serialize(&self) -> Vec<u8> {
                    let mut n = *self;
                    let bytes: Vec<u8> = std::iter::from_fn(|| {
                        if n == 0 { None } else {
                            let b = (n & 0xFF) as u8;
                            n >>= 8;
                            Some(b)
                        }
                    }).collect();
                    let len = bytes.len().max(1);
                    if len == 1 {
                        return vec![*self as u8];
                    }
                    let high5 = (bytes[len - 1] >> 3) & 0x1F;
                    let mut buf = vec![(high5 << 3) | ((len - 1) as u8 & 0x07)];
                    buf.extend(bytes.iter().take(len - 1).cloned());
                    buf.push(bytes[len - 1] & 0x07);
                    buf
                }
            }

            impl Deserialize for $t {
                fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
                    let mut first = [0u8; 1];
                    r.read_exact(&mut first)?;
                    if first[0] >> 3 == 0 {
                        return Ok(first[0] as $t);
                    }
                    let high5 = first[0] >> 3;
                    let len = (first[0] & 0x07) + 1;
                    let mut buf = vec![0u8; len as usize];
                    r.read_exact(&mut buf)?;
                    *buf.last_mut().unwrap() |= high5 << 3;
                    Ok(buf.into_iter().enumerate().map(|(i, b)| (b as $t) << (8 * i)).fold(0, |acc, x| acc | x))
                }
            }
        )*
    };
}

impl_uint!(u64, u32, usize, u16);

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self) -> Vec<u8> {
        let mut buf = self.len().serialize();
        for item in self {
            buf.extend(item.serialize());
        }
        buf
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let len = usize::deserialize(r)?;
        let mut buf = Vec::with_capacity(len);
        for _ in 0..len {
            buf.push(T::deserialize(r)?);
        }
        Ok(buf)
    }
}
