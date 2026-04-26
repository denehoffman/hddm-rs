#![allow(dead_code)]
use std::io::{self, Cursor, Read, Write};

pub(crate) struct XdrWriter<W: Write> {
    pub(crate) inner: W,
}
impl<W: Write> XdrWriter<W> {
    pub(crate) fn new(writer: W) -> Self {
        Self { inner: writer }
    }

    pub(crate) fn write_string(&mut self, value: &str) -> io::Result<()> {
        let bytes = value.as_bytes();
        let bytes_len = bytes.len() as u32;
        self.write_all(&bytes_len.to_be_bytes())?;
        self.write_all(bytes)?;
        self.write_all(&vec![0u8; (4 - (bytes_len as usize % 4)) % 4])?;
        Ok(())
    }

    pub(crate) fn write_u32(&mut self, value: u32) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }
    pub(crate) fn write_i32(&mut self, value: i32) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }
    pub(crate) fn write_u64(&mut self, value: u64) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }
    pub(crate) fn write_i64(&mut self, value: i64) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }
    pub(crate) fn write_f32(&mut self, value: f32) -> io::Result<()> {
        self.write_all(&value.to_bits().to_be_bytes())
    }
    pub(crate) fn write_f64(&mut self, value: f64) -> io::Result<()> {
        self.write_all(&value.to_bits().to_be_bytes())
    }
    pub(crate) fn write_bool(&mut self, x: bool) -> io::Result<()> {
        self.write_i32(if x { 1 } else { 0 })
    }
    pub(crate) fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.inner.write_all(bytes)
    }
    pub fn into_inner(self) -> W {
        self.inner
    }
}

pub(crate) struct XdrReader<R: Read> {
    pub(crate) inner: R,
}

impl<R: Read> XdrReader<R> {
    pub(crate) fn new(reader: R) -> Self {
        Self { inner: reader }
    }

    pub(crate) fn read_n<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        let mut buf = [0u8; N];
        self.inner.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub(crate) fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner.read_exact(buf)
    }

    pub(crate) fn read_string(&mut self) -> io::Result<String> {
        let string_len = self.read_u32()? as usize;
        let mut bytes = vec![0u8; string_len];
        self.inner.read_exact(&mut bytes)?;
        let padding = (4 - (string_len % 4)) % 4;

        let mut pad = [0u8; 3];

        self.inner.read_exact(&mut pad[..padding])?;

        String::from_utf8(bytes).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    pub(crate) fn read_u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_be_bytes(self.read_n::<4>()?))
    }
    pub(crate) fn read_i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_be_bytes(self.read_n::<4>()?))
    }
    pub(crate) fn read_u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_be_bytes(self.read_n::<8>()?))
    }
    pub(crate) fn read_i64(&mut self) -> io::Result<i64> {
        Ok(i64::from_be_bytes(self.read_n::<8>()?))
    }
    pub(crate) fn read_f32(&mut self) -> io::Result<f32> {
        Ok(f32::from_bits(u32::from_be_bytes(self.read_n::<4>()?)))
    }
    pub(crate) fn read_f64(&mut self) -> io::Result<f64> {
        Ok(f64::from_bits(u64::from_be_bytes(self.read_n::<8>()?)))
    }
    pub(crate) fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.read_i32()? != 0)
    }
    pub fn into_inner(self) -> R {
        self.inner
    }
    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.inner
    }
}

impl XdrReader<Cursor<Vec<u8>>> {
    pub fn get_ref(&self) -> &Cursor<Vec<u8>> {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xdr_write_string() -> io::Result<()> {
        let cases = [
            ("", vec![0x00, 0x00, 0x00, 0x00]),
            ("a", vec![0x00, 0x00, 0x00, 0x01, b'a', 0x00, 0x00, 0x00]),
            ("ab", vec![0x00, 0x00, 0x00, 0x02, b'a', b'b', 0x00, 0x00]),
            ("abc", vec![0x00, 0x00, 0x00, 0x03, b'a', b'b', b'c', 0x00]),
            ("abcd", vec![0x00, 0x00, 0x00, 0x04, b'a', b'b', b'c', b'd']),
            (
                "abcde",
                vec![
                    0x00, 0x00, 0x00, 0x05, b'a', b'b', b'c', b'd', b'e', 0x00, 0x00, 0x00,
                ],
            ),
        ];
        let mut buf = Vec::new();
        for (string, expected) in cases {
            let mut writer = XdrWriter::new(&mut buf);
            writer.write_string(string)?;
            assert_eq!(buf, expected);
            buf.clear();
        }
        Ok(())
    }

    #[test]
    fn test_xdr_read_string() -> io::Result<()> {
        let cases = [
            ("", vec![0x00, 0x00, 0x00, 0x00]),
            ("a", vec![0x00, 0x00, 0x00, 0x01, b'a', 0x00, 0x00, 0x00]),
            ("ab", vec![0x00, 0x00, 0x00, 0x02, b'a', b'b', 0x00, 0x00]),
            ("abc", vec![0x00, 0x00, 0x00, 0x03, b'a', b'b', b'c', 0x00]),
            ("abcd", vec![0x00, 0x00, 0x00, 0x04, b'a', b'b', b'c', b'd']),
            (
                "abcde",
                vec![
                    0x00, 0x00, 0x00, 0x05, b'a', b'b', b'c', b'd', b'e', 0x00, 0x00, 0x00,
                ],
            ),
        ];
        for (expected, data) in cases {
            let mut reader = XdrReader::new(&data[..]);
            let string = reader.read_string()?;
            assert_eq!(string, expected);
        }
        Ok(())
    }
}
