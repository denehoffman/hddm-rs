use crate::{HddmError, HddmResult};

use flate2::{Compression as FlateCompression, read::ZlibDecoder, write::ZlibEncoder};
use std::io::{self, Read, Write};

pub const K_NO_COMPRESSION: i32 = 0x00;
pub const K_Z_COMPRESSION: i32 = 0x10;
pub const K_BZ2_COMPRESSION: i32 = 0x20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    None,
    Zlib,
}

pub struct ZlibBlockWriter<W: Write> {
    inner: W,
}

impl<W: Write> ZlibBlockWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> Write for ZlibBlockWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut encoder = ZlibEncoder::new(Vec::new(), FlateCompression::default());
        encoder.write_all(buf)?;
        let compressed = encoder.finish()?;

        let size = compressed.len() as u32;
        self.inner.write_all(&size.to_be_bytes())?;
        self.inner.write_all(&compressed)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

pub struct ZlibBlockReader<R: Read> {
    inner: R,
    current: std::io::Cursor<Vec<u8>>,
}

impl<R: Read> ZlibBlockReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            current: std::io::Cursor::new(Vec::new()),
        }
    }

    fn refill(&mut self) -> io::Result<bool> {
        let mut size_buf = [0u8; 4];

        match self.inner.read_exact(&mut size_buf) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => return Ok(false),
            Err(err) => return Err(err),
        }

        let compressed_size = u32::from_be_bytes(size_buf) as usize;

        let mut compressed = vec![0u8; compressed_size];
        self.inner.read_exact(&mut compressed)?;

        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        self.current = std::io::Cursor::new(decompressed);

        Ok(true)
    }
}

impl<R: Read> Read for ZlibBlockReader<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        loop {
            let n = self.current.read(out)?;
            if n > 0 {
                return Ok(n);
            }

            if !self.refill()? {
                return Ok(0);
            }
        }
    }
}

pub enum HddmOutputStream<W: Write> {
    Plain(W),
    Zlib(ZlibBlockWriter<W>),
}

impl<W: Write> Write for HddmOutputStream<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Plain(w) => w.write(buf),
            Self::Zlib(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Plain(w) => w.flush(),
            Self::Zlib(w) => w.flush(),
        }
    }
}

pub enum HddmInputStream<R: Read> {
    Plain(R),
    Zlib(ZlibBlockReader<R>),
}

impl<R: Read> Read for HddmInputStream<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Plain(r) => r.read(buf),
            Self::Zlib(r) => r.read(buf),
        }
    }
}

pub fn compression_from_status_bits(bits: i32) -> HddmResult<Compression> {
    match bits & (K_Z_COMPRESSION | K_BZ2_COMPRESSION) {
        0 => Ok(Compression::None),
        K_Z_COMPRESSION => Ok(Compression::Zlib),
        K_BZ2_COMPRESSION => Err(HddmError::FormatError(
            "bzip2 HDDM compression is not implemented".to_string(),
        )),
        other => Err(HddmError::FormatError(format!(
            "unsupported HDDM compression status bits: {other:#x}"
        ))),
    }
}
