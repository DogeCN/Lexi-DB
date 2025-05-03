use std::cmp::min;
use std::collections::VecDeque;
use std::io::{Error, ErrorKind, Read, Result, Write};

const WINDOW_SIZE: usize = 4096;
const LOOKAHEAD_BUF: usize = 258;

pub struct LZ77Compressor<Inner: Write> {
    window: VecDeque<u8>,
    buffer: Vec<u8>,
    inner: Inner,
}

impl<Inner: Write> LZ77Compressor<Inner> {
    pub fn new(inner: Inner) -> Self {
        Self {
            window: VecDeque::with_capacity(WINDOW_SIZE),
            buffer: Vec::with_capacity(LOOKAHEAD_BUF),
            inner,
        }
    }

    fn process(&mut self) -> Result<()> {
        while !self.buffer.is_empty() {
            let (offset, length) = self.find_longest_match();
            if length > 2 {
                self.inner.write_all(&offset.to_be_bytes())?;
                self.inner.write_all(&[(length - 3) as u8])?;
                self.update_window(length);
            } else {
                self.inner.write_all(&[0xFF, self.buffer[0]])?;
                self.update_window(1);
            }
        }
        Ok(())
    }

    fn find_longest_match(&self) -> (u16, usize) {
        let max_match = min(self.buffer.len(), LOOKAHEAD_BUF);
        let window_len = self.window.len();
        let mut best = (0, 0);
        for start in 0..window_len {
            let mut match_len = 0;
            while match_len < max_match && start + match_len < window_len {
                if self.window[start + match_len] != self.buffer[match_len] {
                    break;
                }
                match_len += 1;
            }
            if match_len > best.1 && match_len >= 3 {
                let distance = (window_len - start) as u16;
                best = (distance, match_len);
                if match_len == max_match {
                    break;
                }
            }
        }
        best
    }

    fn update_window(&mut self, advance: usize) {
        let take = min(advance, self.buffer.len());
        for b in self.buffer.drain(..take) {
            if self.window.len() == WINDOW_SIZE {
                self.window.pop_front();
            }
            self.window.push_back(b);
        }
    }
}

impl<Inner: Write> Write for LZ77Compressor<Inner> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.buffer.extend_from_slice(buf);
        self.process()?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        while !self.buffer.is_empty() {
            self.process()?;
        }
        self.inner.flush()
    }
}

impl<Inner: Write> Drop for LZ77Compressor<Inner> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

pub struct LZ77Decompressor<Inner: Read> {
    window: VecDeque<u8>,
    input_buffer: Vec<u8>,
    output_buffer: Vec<u8>,
    inner: Inner,
    eof: bool, // 标记输入流是否结束
}

impl<Inner: Read> LZ77Decompressor<Inner> {
    pub fn new(inner: Inner) -> Self {
        Self {
            window: VecDeque::with_capacity(WINDOW_SIZE),
            input_buffer: Vec::with_capacity(LOOKAHEAD_BUF * 2),
            output_buffer: Vec::with_capacity(WINDOW_SIZE),
            inner,
            eof: false,
        }
    }

    fn process(&mut self) -> Result<()> {
        let mut cursor = 0;
        while cursor < self.input_buffer.len() {
            if self.input_buffer[cursor] == 0xFF {
                if cursor + 1 >= self.input_buffer.len() {
                    if self.eof {
                        return Err(Error::new(ErrorKind::UnexpectedEof, "Incomplete literal"));
                    }
                    break;
                }
                let b = self.input_buffer[cursor + 1];
                self.window.push_back(b);
                self.output_buffer.push(b);
                while self.window.len() > WINDOW_SIZE {
                    self.window.pop_front();
                }
                cursor += 2;
            } else {
                if cursor + 3 > self.input_buffer.len() {
                    if self.eof {
                        return Err(Error::new(ErrorKind::UnexpectedEof, "Incomplete match"));
                    }
                    break;
                }
                let offset =
                    u16::from_be_bytes([self.input_buffer[cursor], self.input_buffer[cursor + 1]]);
                let length = self.input_buffer[cursor + 2] as usize + 3;
                if length < 3 || length > 258 {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Invalid length: {}", length),
                    ));
                }
                cursor += 3;

                if offset == 0 || offset as usize > self.window.len() {
                    return Err(Error::new(ErrorKind::InvalidData, "Invalid offset value"));
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
                    self.output_buffer.push(b);
                    if self.window.len() > WINDOW_SIZE {
                        self.window.pop_front();
                    }
                }
            }
        }
        self.input_buffer.drain(..cursor);
        Ok(())
    }
}

impl<Inner: Read> Read for LZ77Decompressor<Inner> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        while self.output_buffer.is_empty() {
            let mut temp = [0; 1024];
            let n = self.inner.read(&mut temp)?;
            if n == 0 {
                self.eof = true; // 标记输入已结束
                self.process()?; // 最后一次尝试处理残留数据
                if !self.input_buffer.is_empty() {
                    return Err(Error::new(ErrorKind::InvalidData, "Trailing data after EOF"));
                }
                break;
            }
            self.input_buffer.extend_from_slice(&temp[..n]);
            self.process()?;
        }
        let len = min(self.output_buffer.len(), buf.len());
        buf[..len].copy_from_slice(&self.output_buffer[..len]);
        self.output_buffer.drain(..len);
        Ok(len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use std::io::ErrorKind;

    #[test]
    fn roundtrip_basic() -> Result<()> {
        let data = b"ABCDEFGHABCDEFGH";

        let mut compressed = Vec::new();
        {
            let mut compressor = LZ77Compressor::new(&mut compressed);
            compressor.write_all(data)?;
            compressor.flush()?;
        }

        let mut decompressed = Vec::new();
        {
            let mut decompressor = LZ77Decompressor::new(compressed.as_slice());
            decompressor.read_to_end(&mut decompressed)?;
        }

        assert_eq!(data.as_slice(), &decompressed);
        Ok(())
    }

    #[test]
    fn empty_data() -> Result<()> {
        let data = b"";

        let mut compressed = Vec::new();
        {
            let mut compressor = LZ77Compressor::new(&mut compressed);
            compressor.write_all(data)?;
            compressor.flush()?;
        }
        assert!(compressed.is_empty());

        let mut decompressed = Vec::new();
        {
            let mut decompressor = LZ77Decompressor::new(compressed.as_slice());
            decompressor.read_to_end(&mut decompressed)?;
        }

        assert!(decompressed.is_empty());
        Ok(())
    }

    #[test]
    fn repeated_pattern() -> Result<()> {
        let data = b"AAAAA".repeat(100);

        let mut compressed = Vec::new();
        {
            let mut compressor = LZ77Compressor::new(&mut compressed);
            compressor.write_all(&data)?;
            compressor.flush()?;
        }
        assert!(compressed.len() < data.len() / 2);

        let mut decompressed = Vec::new();
        {
            let mut decompressor = LZ77Decompressor::new(compressed.as_slice());
            decompressor.read_to_end(&mut decompressed)?;
        }

        assert_eq!(data, decompressed);
        Ok(())
    }

    #[test]
    fn streaming_chunks() -> Result<()> {
        let chunks = [
            b"Part1: ABC".as_slice(),
            b"Part2: DEF".as_slice(),
            b"Part3: XYZ".as_slice(),
        ];

        let mut compressed = Vec::new();
        {
            let mut compressor = LZ77Compressor::new(&mut compressed);
            for chunk in &chunks {
                compressor.write_all(chunk)?;
            }
            compressor.flush()?;
        }

        let mut decompressed = Vec::new();
        {
            let mut decompressor = LZ77Decompressor::new(compressed.as_slice());
            decompressor.read_to_end(&mut decompressed)?;
        }

        // 验证窗口状态
        assert!(
            decompressed.len() <= WINDOW_SIZE,
            "Window size overflow: {}",
            decompressed.len()
        );

        let expected: Vec<u8> = chunks.into_iter().flatten().cloned().collect();
        assert_eq!(
            expected, decompressed,
            "Decompressed data mismatch\nExpected: {:?}\nActual: {:?}",
            expected, decompressed
        );

        Ok(())
    }

    #[test]
    fn large_random_data() -> Result<()> {
        let mut rng = rand::rng();
        let mut data = vec![0u8; 1000];
        rng.fill(&mut data[..]);

        let mut compressed = Vec::new();
        {
            let mut compressor = LZ77Compressor::new(&mut compressed);
            compressor.write_all(&data)?;
            compressor.flush()?;
        }

        let mut decompressed = Vec::new();
        {
            let mut decompressor = LZ77Decompressor::new(compressed.as_slice());
            decompressor.read_to_end(&mut decompressed)?;
        }

        assert_eq!(data, decompressed);
        Ok(())
    }

    #[test]
    fn invalid_data_handling() -> Result<()> {
        // 测试用例1: 不完整的字面量
        let mut decompressor = LZ77Decompressor::new([0xFFu8].as_ref());
        let mut buf = Vec::new();
        assert!(decompressor.read_to_end(&mut buf).is_err());

        // 测试用例2: 无效的匹配标记（只有2字节）
        let mut decompressor = LZ77Decompressor::new([0x01, 0x02].as_ref());
        let mut buf = Vec::new();
        assert!(decompressor.read_to_end(&mut buf).is_err());

        // 测试用例3: 非法偏移量（offset=0）
        let mut decompressor = LZ77Decompressor::new([0x00, 0x00, 0x05].as_ref());
        let mut buf = Vec::new();
        let result = decompressor.read_to_end(&mut buf);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidData);

        // 测试用例4: 非法长度（length=258）
        let mut decompressor = LZ77Decompressor::new([0x10, 0x00, 0xFF].as_ref());
        let mut buf = Vec::new();
        let result = decompressor.read_to_end(&mut buf);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidData);

        Ok(())
    }

    #[test]
    fn streaming_without_flush() -> Result<()> {
        let data = b"ABCDEFGHABCDEFGH";
        let mut compressed = Vec::new();

        // 不调用flush，依赖Drop自动处理
        {
            let mut compressor = LZ77Compressor::new(&mut compressed);
            compressor.write_all(data)?;
        } // 此处自动调用drop，触发flush

        let mut decompressed = Vec::new();
        let mut decompressor = LZ77Decompressor::new(compressed.as_slice());
        decompressor.read_to_end(&mut decompressed)?;

        assert_eq!(data.as_slice(), &decompressed);
        Ok(())
    }
}
