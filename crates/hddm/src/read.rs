use std::io::{BufRead, Cursor, Read};

use bzip2::read::BzDecoder;
use flate2::read::ZlibDecoder;

use crate::{
    Compression, HddmError, HddmResult, K_BZ2_COMPRESSION, K_Z_COMPRESSION, xdr::XdrReader,
};

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

    pub fn from_payload(payload: Vec<u8>) -> Self {
        let size = payload.len();
        Self {
            reader: HddmReader::new(Cursor::new(payload)),
            size,
        }
    }

    pub fn ensure_empty(&self) -> HddmResult<()> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(HddmError::FormatError(
                "HDDM payload has unread bytes".to_string(),
            ))
        }
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

pub fn read_i32<R: Read>(r: &mut R) -> HddmResult<i32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

pub fn read_compression<R: Read>(r: &mut R) -> HddmResult<Compression> {
    let token_size = read_i32(r)?;
    if token_size != 8 {
        return Err(HddmError::FormatError(format!(
            "invalid HDDM status token size: {token_size}"
        )));
    }
    let format = read_i32(r)?;
    if format != 0 {
        return Err(HddmError::FormatError(format!(
            "unsupported HDDM status token format: {format}"
        )));
    }
    let status_bits = read_i32(r)?;
    match status_bits & (K_Z_COMPRESSION | K_BZ2_COMPRESSION) {
        0 => Ok(Compression::None),
        K_Z_COMPRESSION => Ok(Compression::Zlib),
        K_BZ2_COMPRESSION => Ok(Compression::Bzip2),
        other => Err(HddmError::FormatError(format!(
            "unsupported HDDM compression bits: {other:#x}"
        ))),
    }
}

fn read_next_zlib_block_with_size<R: Read>(
    r: &mut R,
    compressed_size: i32,
) -> HddmResult<Cursor<Vec<u8>>> {
    if compressed_size < 0 {
        return Err(HddmError::FormatError(format!(
            "negative zlib block size: {compressed_size}"
        )));
    }

    let mut compressed = vec![0u8; compressed_size as usize];
    r.read_exact(&mut compressed)?;

    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    Ok(Cursor::new(decompressed))
}

fn read_next_bzip2_block_with_size<R: Read>(
    r: &mut R,
    compressed_size: i32,
) -> HddmResult<Cursor<Vec<u8>>> {
    if compressed_size < 0 {
        return Err(HddmError::FormatError(format!(
            "negative bzip2 block size: {compressed_size}"
        )));
    }

    let mut compressed = vec![0u8; compressed_size as usize];
    r.read_exact(&mut compressed)?;

    let mut decoder = BzDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    Ok(Cursor::new(decompressed))
}

pub fn read_payload<R: Read>(r: &mut R, size: i32) -> HddmResult<Vec<u8>> {
    if size < 0 {
        return Err(HddmError::FormatError(format!(
            "negative HDDM record size: {size}"
        )));
    }
    let mut payload = vec![0u8; size as usize];
    r.read_exact(&mut payload)?;
    Ok(payload)
}

pub(crate) struct HddmRecordReader<R: BufRead> {
    raw: R,
    compression: Compression,
    current_block: Option<Cursor<Vec<u8>>>,
}

impl<R: BufRead> HddmRecordReader<R> {
    pub fn new(raw: R) -> Self {
        Self {
            raw,
            compression: Compression::None,
            current_block: None,
        }
    }
    fn current_block_is_empty(&self) -> bool {
        match &self.current_block {
            Some(block) => block.position() as usize == block.get_ref().len(),
            None => true,
        }
    }
    pub fn next_record_payload(&mut self) -> HddmResult<Option<Vec<u8>>> {
        loop {
            match self.compression {
                Compression::None => {
                    let size = match read_i32(&mut self.raw) {
                        Ok(size) => size,
                        Err(HddmError::IoError(err))
                            if err.kind() == std::io::ErrorKind::UnexpectedEof =>
                        {
                            return Ok(None);
                        }
                        Err(err) => return Err(err),
                    };
                    if size == 1 {
                        self.compression = read_compression(&mut self.raw)?;
                        self.current_block = None;
                        continue;
                    }
                    return Ok(Some(read_payload(&mut self.raw, size)?));
                }
                Compression::Zlib => {
                    if self.current_block_is_empty() {
                        let size = match read_i32(&mut self.raw) {
                            Ok(size) => size,
                            Err(HddmError::IoError(err))
                                if err.kind() == std::io::ErrorKind::UnexpectedEof =>
                            {
                                return Ok(None);
                            }
                            Err(err) => return Err(err),
                        };

                        if size == 1 {
                            self.compression = read_compression(&mut self.raw)?;
                            self.current_block = None;
                            continue;
                        }

                        self.current_block =
                            Some(read_next_zlib_block_with_size(&mut self.raw, size)?);
                    }

                    let block = self.current_block.as_mut().unwrap();
                    let size = read_i32(block)?;

                    if size == 1 {
                        self.compression = read_compression(block)?;
                        self.current_block = None;
                        continue;
                    }

                    return Ok(Some(read_payload(block, size)?));
                }
                Compression::Bzip2 => {
                    if self.current_block_is_empty() {
                        let size = match read_i32(&mut self.raw) {
                            Ok(size) => size,
                            Err(HddmError::IoError(err))
                                if err.kind() == std::io::ErrorKind::UnexpectedEof =>
                            {
                                return Ok(None);
                            }
                            Err(err) => return Err(err),
                        };

                        if size == 1 {
                            self.compression = read_compression(&mut self.raw)?;
                            self.current_block = None;
                            continue;
                        }

                        self.current_block =
                            Some(read_next_bzip2_block_with_size(&mut self.raw, size)?);
                    }

                    let block = self.current_block.as_mut().unwrap();
                    let size = read_i32(block)?;

                    if size == 1 {
                        self.compression = read_compression(block)?;
                        self.current_block = None;
                        continue;
                    }

                    return Ok(Some(read_payload(block, size)?));
                }
            }
        }
    }
}
