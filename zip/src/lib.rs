use std::cmp::min;
use std::collections::VecDeque;
use std::io::{Error, ErrorKind, Read, Result, Write};

const WINDOW_SIZE: usize = 4096;
const LOOKAHEAD_BUF: usize = 258; // 允许最大匹配长度258

pub struct LZ77Compressor<Inner: Read> {
    window: VecDeque<u8>,
    buffer: Vec<u8>,
    output: Vec<u8>,
    inner: Inner,
    eof: bool,
}

impl<Inner: Read> LZ77Compressor<Inner> {
    pub fn new(inner: Inner) -> Self {
        Self {
            window: VecDeque::with_capacity(WINDOW_SIZE),
            buffer: Vec::with_capacity(LOOKAHEAD_BUF),
            output: Vec::new(),
            inner,
            eof: false,
        }
    }

    fn fill_buffer(&mut self) -> Result<usize> {
        if self.eof {
            return Ok(0);
        }
        let mut temp = [0u8; LOOKAHEAD_BUF];
        let n = self.inner.read(&mut temp)?;
        if n == 0 {
            self.eof = true;
        } else {
            self.buffer.extend_from_slice(&temp[..n]);
        }
        Ok(n)
    }

    fn process(&mut self) -> Result<()> {
        while self.buffer.len() >= LOOKAHEAD_BUF || (self.eof && !self.buffer.is_empty()) {
            let (offset, length) = self.find_longest_match();

            if length > 2 {
                self.emit_match(offset, length)?;
                self.update_window(length);
            } else {
                self.emit_literal(self.buffer[0]);
                self.update_window(1);
            }
            if self.buffer.len() < LOOKAHEAD_BUF && !self.eof {
                self.fill_buffer()?;
            }
        }
        Ok(())
    }

    fn find_longest_match(&self) -> (u16, usize) {
        let max_match = min(self.buffer.len(), LOOKAHEAD_BUF);
        let mut best = (0, 0);
        let (s1, s2) = self.window.as_slices();
        let window_len = self.window.len();

        for start in 0..window_len {
            let mut match_len = 0;
            while match_len < max_match && (start + match_len) < window_len {
                let pos = start + match_len;
                let window_byte = if pos < s1.len() {
                    s1[pos]
                } else {
                    s2[pos - s1.len()]
                };
                if window_byte != self.buffer[match_len] {
                    break;
                }
                match_len += 1;
            }
            if match_len > best.1 && match_len >= 3 {
                let distance = (window_len - start) as u16;
                best = (distance, match_len);
                if match_len == max_match {
                    return best; // 提前返回最优解
                }
            }
        }
        best
    }

    fn emit_match(&mut self, offset: u16, length: usize) -> Result<()> {
        if offset == 0 || length < 3 || length > LOOKAHEAD_BUF {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Invalid match: offset={} length={}", offset, length),
            ));
        }

        self.output.extend(offset.to_be_bytes());
        self.output.push((length - 3) as u8);
        Ok(())
    }

    fn emit_literal(&mut self, byte: u8) {
        self.output.push(0xFF);
        self.output.push(byte);
    }

    fn update_window(&mut self, advance: usize) {
        let advance = min(advance, self.buffer.len());
        let drained: Vec<_> = self.buffer.drain(..advance).collect();
        if drained.is_empty() {
            return;
        }

        for &b in &drained {
            if self.window.len() == WINDOW_SIZE {
                self.window.pop_front();
            }
            self.window.push_back(b);
        }
    }
}

impl<Inner: Read> Read for LZ77Compressor<Inner> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        while self.output.is_empty() && (!self.eof || !self.buffer.is_empty()) {
            if self.buffer.len() < LOOKAHEAD_BUF && !self.eof {
                self.fill_buffer()?;
            }
            self.process()?;
        }
        let len = self.output.len().min(buf.len());
        buf[..len].copy_from_slice(&self.output[..len]);
        self.output.drain(..len);
        Ok(len)
    }
}

pub struct LZ77Decompressor<Inner: Write> {
    window: VecDeque<u8>,
    buffer: Vec<u8>,
    inner: Inner,
}

impl<Inner: Write> LZ77Decompressor<Inner> {
    pub fn new(inner: Inner) -> Self {
        Self {
            window: VecDeque::with_capacity(WINDOW_SIZE),
            buffer: Vec::with_capacity(LOOKAHEAD_BUF * 2),
            inner,
        }
    }
}

impl<Inner: Write> Write for LZ77Decompressor<Inner> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        let mut cursor = 0;
        if self.buffer.len() % 3 != 0 && !self.buffer.contains(&0xFF) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Trailing incomplete data",
            ));
        }

        while cursor < self.buffer.len() {
            if self.buffer[cursor] == 0xFF {
                if cursor + 1 >= self.buffer.len() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Incomplete literal",
                    ));
                }
                let b = self.buffer[cursor + 1];
                self.window.push_back(b);
                self.inner.write_all(&[b])?;
                while self.window.len() > WINDOW_SIZE {
                    self.window.pop_front();
                }
                cursor += 2;
            } else {
                if cursor + 3 > self.buffer.len() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Incomplete match",
                    ));
                }
                let offset = u16::from_be_bytes([self.buffer[cursor], self.buffer[cursor + 1]]);
                let length = self.buffer[cursor + 2] as usize + 3;
                if length < 3 || length > 258 {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Invalid length: {}", length),
                    ));
                }
                cursor += 3;

                if offset == 0 || offset as usize > self.window.len() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid offset value",
                    ));
                }

                let window_len = self.window.len();
                for i in 0..length {
                    let pos = window_len - offset as usize + (i % offset as usize);
                    if pos >= window_len {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Invalid reference position",
                        ));
                    }
                    let b = self.window[pos];
                    self.window.push_back(b);
                    self.inner.write_all(&[b])?;
                    if self.window.len() > WINDOW_SIZE {
                        self.window.pop_front();
                    }
                }
            }
        }
        if cursor < self.buffer.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Trailing incomplete data",
            ));
        }
        self.buffer.drain(..cursor);
        self.inner.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn roundtrip_basic() -> std::io::Result<()> {
        let data = b"ABCDEFGHABCDEFGH";

        let mut compressor = LZ77Compressor::<&[u8]>::new(data.as_ref());
        let mut compressed = Vec::new();
        compressor.read_to_end(&mut compressed)?;

        let mut decompressed = Vec::new();
        {
            let mut decompressor = LZ77Decompressor::new(&mut decompressed);
            decompressor.write_all(&compressed)?;
            decompressor.flush()?;
        }

        assert_eq!(data.as_slice(), &decompressed);
        Ok(())
    }

    #[test]
    fn empty_data() -> std::io::Result<()> {
        let data = b"";

        let mut compressor = LZ77Compressor::<&[u8]>::new(data.as_ref());
        let mut compressed = Vec::new();
        compressor.read_to_end(&mut compressed)?;
        assert!(compressed.is_empty());

        let mut decompressed = Vec::new();
        {
            let mut decompressor = LZ77Decompressor::new(&mut decompressed);
            decompressor.write_all(&compressed)?;
            decompressor.flush()?;
        }

        assert!(decompressed.is_empty());
        Ok(())
    }

    #[test]
    fn repeated_pattern() -> std::io::Result<()> {
        let data = b"AAAAA".repeat(100);

        let mut compressor = LZ77Compressor::<&[u8]>::new(data.as_ref());
        let mut compressed = Vec::new();
        compressor.read_to_end(&mut compressed)?;
        assert!(compressed.len() < data.len() / 2);

        let mut decompressed = Vec::new();
        {
            let mut decompressor = LZ77Decompressor::new(&mut decompressed);
            decompressor.write_all(&compressed)?;
            decompressor.flush()?;
        }

        assert_eq!(data, decompressed);
        Ok(())
    }

    #[test]
    fn streaming_chunks() -> std::io::Result<()> {
        let chunks = [
            b"Part1: ABC".as_slice(),
            b"Part2: DEF".as_slice(),
            b"Part3: XYZ".as_slice(),
        ];

        let concat: Vec<u8> = chunks.concat();
        let mut compressed = Vec::new();
        {
            let mut compressor = LZ77Compressor::<&[u8]>::new(concat.as_slice());
            compressor.read_to_end(&mut compressed)?;
        }

        let mut decompressed = Vec::new();
        {
            let mut decompressor = LZ77Decompressor::new(&mut decompressed);
            decompressor.write_all(&compressed)?;
            decompressor.flush()?;
        }

        // 验证窗口状态
        assert!(
            decompressed.len() <= WINDOW_SIZE,
            "Window size overflow: {}",
            decompressed.len()
        );

        let expected: Vec<u8> = chunks.iter().flat_map(|c| c.iter().copied()).collect();
        assert_eq!(
            expected, decompressed,
            "Decompressed data mismatch\nExpected: {:?}\nActual: {:?}",
            expected, decompressed
        );

        Ok(())
    }

    #[test]
    fn large_random_data() -> std::io::Result<()> {
        let mut rng = rand::rng();
        let mut data = vec![0u8; 1000];
        rng.fill(&mut data[..]);

        let mut compressed = Vec::new();
        {
            let mut compressor = LZ77Compressor::<&[u8]>::new(data.as_slice());
            compressor.read_to_end(&mut compressed)?;
        }

        let mut decompressed = Vec::new();
        {
            let mut decompressor = LZ77Decompressor::new(&mut decompressed);
            decompressor.write_all(&compressed)?;
            decompressor.flush()?;
        }

        assert_eq!(data, decompressed);
        Ok(())
    }

    #[test]
    fn invalid_data_handling() {
        let corrupt_data = vec![0xFF, 0x00];

        let mut decompressed = Vec::new();
        let mut decompressor = LZ77Decompressor::new(&mut decompressed);
        decompressor.write_all(&corrupt_data).unwrap();
        let result = decompressor.flush();

        assert!(result.is_err());
    }
}
