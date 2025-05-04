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
    #[inline]
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
                    let mut buf = vec![0];
                    let low = n as u8 & 0xF;
                    n >>= 4;
                    while n > 0 {
                        buf.push((n & 0xFF) as u8);
                        n >>= 8;
                    }
                    buf[0] = (buf.len() - 1) as u8 | low << 4;
                    buf
                }
            }

            impl Deserialize for $t {
                fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
                    let mut buf = [0];
                    r.read_exact(&mut buf)?;
                    let mut n = buf[0] as Self >> 4;
                    let mut buf = vec![0; buf[0] as usize & 0xF];
                    r.read_exact(&mut buf)?;
                    for (i, &b) in buf.iter().enumerate() {
                        n |= (b as Self) << (4 + 8 * i);
                    }
                    Ok(n)
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
