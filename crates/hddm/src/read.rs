use crate::{HddmError, HddmResult, xdr::XdrReader};
use std::io::{Cursor, Read};

pub struct HddmReader<R: Read> {
    xdr: XdrReader<R>,
}

impl<R: Read> HddmReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            xdr: XdrReader::new(inner),
        }
    }

    pub fn read_i32(&mut self) -> HddmResult<i32> {
        Ok(self.xdr.read_i32()?)
    }

    pub fn read_u32(&mut self) -> HddmResult<u32> {
        Ok(self.xdr.read_u32()?)
    }

    pub fn read_i64(&mut self) -> HddmResult<i64> {
        Ok(self.xdr.read_i64()?)
    }

    pub fn read_u64(&mut self) -> HddmResult<u64> {
        Ok(self.xdr.read_u64()?)
    }

    pub fn read_f32(&mut self) -> HddmResult<f32> {
        Ok(self.xdr.read_f32()?)
    }

    pub fn read_f64(&mut self) -> HddmResult<f64> {
        Ok(self.xdr.read_f64()?)
    }

    pub fn read_bool(&mut self) -> HddmResult<bool> {
        Ok(self.read_i32()? != 0)
    }

    pub fn read_string(&mut self) -> HddmResult<String> {
        Ok(self.xdr.read_string()?)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> HddmResult<()> {
        Ok(self.xdr.read_exact(buf)?)
    }

    pub fn read_element<T, F>(&mut self, f: F) -> HddmResult<T>
    where
        F: FnOnce(&mut ElementReader) -> HddmResult<T>,
    {
        let size = self.read_i32()?;

        if size < 0 {
            return Err(HddmError::FormatError(format!(
                "negative HDDM element size: {size}"
            )));
        }

        let size = size as usize;
        let mut payload = vec![0u8; size];
        self.read_exact(&mut payload)?;

        let mut element = ElementReader {
            reader: HddmReader::new(Cursor::new(payload)),
            size,
        };

        let value = f(&mut element)?;

        if !element.is_empty() {
            return Err(HddmError::FormatError(
                "HDDM element size mismatch: unread bytes remain".to_string(),
            ));
        }

        Ok(value)
    }

    pub fn read_link<T: HddmRead>(&mut self) -> HddmResult<Option<T>> {
        self.read_element(|e| {
            if e.is_empty() {
                Ok(None)
            } else {
                Ok(Some(T::read_contents(e)?))
            }
        })
    }

    pub fn read_list<T: HddmRead>(&mut self) -> HddmResult<Vec<T>> {
        self.read_element(|e| {
            if e.is_empty() {
                return Ok(Vec::new());
            }

            let count = e.read_i32()?;

            if count < 0 {
                return Err(HddmError::FormatError(format!(
                    "negative HDDM list count: {count}"
                )));
            }

            let mut items = Vec::with_capacity(count as usize);

            for _ in 0..count {
                items.push(T::read_contents(e)?);
            }

            Ok(items)
        })
    }

    pub fn read_record<T: HddmRead>(&mut self) -> HddmResult<Option<T>> {
        match T::read_hddm(self) {
            Ok(record) => Ok(Some(record)),
            Err(HddmError::IoError(err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                Ok(None)
            }

            Err(err) => Err(err),
        }
    }
}

pub struct ElementReader {
    reader: HddmReader<Cursor<Vec<u8>>>,
    size: usize,
}

impl ElementReader {
    fn position(&self) -> usize {
        self.reader.xdr.get_ref().position() as usize
    }

    fn is_empty(&self) -> bool {
        self.position() == self.size
    }

    pub fn read_i32(&mut self) -> HddmResult<i32> {
        self.reader.read_i32()
    }

    pub fn read_u32(&mut self) -> HddmResult<u32> {
        self.reader.read_u32()
    }

    pub fn read_i64(&mut self) -> HddmResult<i64> {
        self.reader.read_i64()
    }

    pub fn read_u64(&mut self) -> HddmResult<u64> {
        self.reader.read_u64()
    }

    pub fn read_f32(&mut self) -> HddmResult<f32> {
        self.reader.read_f32()
    }

    pub fn read_f64(&mut self) -> HddmResult<f64> {
        self.reader.read_f64()
    }

    pub fn read_bool(&mut self) -> HddmResult<bool> {
        self.reader.read_bool()
    }

    pub fn read_string(&mut self) -> HddmResult<String> {
        self.reader.read_string()
    }

    pub fn read_link<T: HddmRead>(&mut self) -> HddmResult<Option<T>> {
        self.reader.read_link()
    }

    pub fn read_list<T: HddmRead>(&mut self) -> HddmResult<Vec<T>> {
        self.reader.read_list()
    }
}

pub trait HddmRead: Sized {
    fn read_contents(r: &mut ElementReader) -> HddmResult<Self>;

    fn read_hddm<R: Read>(r: &mut HddmReader<R>) -> HddmResult<Self> {
        r.read_element(Self::read_contents)
    }
}

pub trait HddmPrimitiveRead: Sized {
    fn read_primitive(r: &mut ElementReader) -> HddmResult<Self>;
}

impl HddmPrimitiveRead for i32 {
    fn read_primitive(r: &mut ElementReader) -> HddmResult<i32> {
        r.read_i32()
    }
}

impl HddmPrimitiveRead for u32 {
    fn read_primitive(r: &mut ElementReader) -> HddmResult<u32> {
        r.read_u32()
    }
}

impl HddmPrimitiveRead for i64 {
    fn read_primitive(r: &mut ElementReader) -> HddmResult<i64> {
        r.read_i64()
    }
}

impl HddmPrimitiveRead for u64 {
    fn read_primitive(r: &mut ElementReader) -> HddmResult<u64> {
        r.read_u64()
    }
}

impl HddmPrimitiveRead for f32 {
    fn read_primitive(r: &mut ElementReader) -> HddmResult<f32> {
        r.read_f32()
    }
}

impl HddmPrimitiveRead for f64 {
    fn read_primitive(r: &mut ElementReader) -> HddmResult<f64> {
        r.read_f64()
    }
}

impl HddmPrimitiveRead for bool {
    fn read_primitive(r: &mut ElementReader) -> HddmResult<bool> {
        r.read_bool()
    }
}

impl HddmPrimitiveRead for String {
    fn read_primitive(r: &mut ElementReader) -> HddmResult<String> {
        r.read_string()
    }
}
