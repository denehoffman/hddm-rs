pub(crate) mod error;
pub mod header;
pub(crate) mod particles;
pub(crate) mod read;
pub(crate) mod write;
pub(crate) mod xdr;

use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

pub use error::{HddmError, HddmResult};
#[cfg(feature = "derive")]
pub use hddm_derive::{HddmRead, HddmWrite};
pub use header::HddmModel;
pub use read::{ElementReader, HddmPrimitiveRead, HddmRead, HddmReader};
pub use write::{HddmPrimitiveWrite, HddmWrite, HddmWriter};

use crate::{
    header::read_header_streaming,
    read::HddmRecordReader,
    write::{Bzip2BlockWriter, HddmOutputStream, ZlibBlockWriter, write_compression_token},
};

const K_NO_COMPRESSION: i32 = 0x00;
const K_Z_COMPRESSION: i32 = 0x10;
const K_BZ2_COMPRESSION: i32 = 0x20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    None,
    Zlib,
    Bzip2,
}

pub struct HddmFile<R: BufRead> {
    pub header: HddmModel,
    records: HddmRecordReader<R>,
}
impl HddmFile<BufReader<File>> {
    pub fn open<P: AsRef<Path>>(path: P) -> HddmResult<Self> {
        let file = File::open(path)?;
        let mut raw = BufReader::new(file);
        let (header, _) = read_header_streaming(&mut raw)?;
        Ok(Self {
            header,
            records: HddmRecordReader::new(raw),
        })
    }
}

impl<R: BufRead> HddmFile<R> {
    pub fn header(&self) -> &HddmModel {
        &self.header
    }

    pub fn read_record<T: HddmRead>(&mut self) -> HddmResult<Option<T>> {
        let Some(payload) = self.records.next_record_payload()? else {
            return Ok(None);
        };
        let mut element = ElementReader::from_payload(payload);
        let value = T::read_contents(&mut element)?;
        element.ensure_empty()?;
        Ok(Some(value))
    }
}

pub struct HddmFileWriter {
    writer: HddmWriter<HddmOutputStream<File>>,
}
impl HddmFileWriter {
    pub fn create<P: AsRef<Path>>(path: P, header: &str) -> HddmResult<Self> {
        Self::create_with_compression(path, header, Compression::None)
    }

    pub fn create_with_compression<P: AsRef<Path>>(
        path: P,
        header: &str,
        compression: Compression,
    ) -> HddmResult<Self> {
        let file = File::create(path)?;
        let mut raw = BufWriter::new(file);
        raw.write_all(header.as_bytes())?;
        {
            let mut token_writer = HddmWriter::new(&mut raw);
            write_compression_token(&mut token_writer, compression)?;
        }
        let stream = match compression {
            Compression::None => HddmOutputStream::None(raw),
            Compression::Zlib => HddmOutputStream::Zlib(ZlibBlockWriter::new(raw)),
            Compression::Bzip2 => HddmOutputStream::Bzip2(Bzip2BlockWriter::new(raw)),
        };
        Ok(Self {
            writer: HddmWriter::new(stream),
        })
    }

    pub fn write_record<T: HddmWrite>(&mut self, record: &T) -> HddmResult<()> {
        record.write_hddm(&mut self.writer)
    }

    pub fn writer(&mut self) -> &mut HddmWriter<HddmOutputStream<File>> {
        &mut self.writer
    }

    pub fn flush(&mut self) -> HddmResult<()> {
        self.writer.flush()
    }

    pub fn finish(mut self) -> HddmResult<()> {
        self.flush()
    }
}

impl Drop for HddmFileWriter {
    fn drop(&mut self) {
        self.flush().unwrap();
    }
}
