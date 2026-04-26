pub(crate) mod compression;
pub(crate) mod error;
pub(crate) mod header;
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
pub use header::HddmModel;
pub use read::{ElementReader, HddmPrimitiveRead, HddmRead, HddmReader};
pub use write::{HddmPrimitiveWrite, HddmWrite, HddmWriter};

use crate::header::read_header_streaming;

#[cfg(feature = "derive")]
pub use hddm_derive::{HddmRead, HddmWrite};

pub struct HddmFile<R: BufRead> {
    pub header: HddmModel,
    reader: HddmReader<R>,
}
impl HddmFile<BufReader<File>> {
    pub fn open<P: AsRef<Path>>(path: P) -> HddmResult<Self> {
        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(file);
        let (header, _) = read_header_streaming(&mut buf_reader)?;
        let reader = HddmReader::new(buf_reader);
        Ok(Self { header, reader })
    }
}

impl<R: BufRead> HddmFile<R> {
    pub fn header(&self) -> &HddmModel {
        &self.header
    }

    pub fn reader(&mut self) -> &mut HddmReader<R> {
        &mut self.reader
    }

    pub fn read_record<T: HddmRead>(&mut self) -> HddmResult<Option<T>> {
        self.reader.read_record()
    }
}

pub struct HddmFileWriter {
    writer: HddmWriter<BufWriter<File>>,
}
impl HddmFileWriter {
    pub fn create<P: AsRef<Path>>(path: P, header: &str) -> HddmResult<Self> {
        let file = File::create(path)?;
        let mut out = BufWriter::new(file);
        out.write_all(header.as_bytes())?;
        Ok(Self {
            writer: HddmWriter::new(out),
        })
    }

    pub fn write_record<T: HddmWrite>(&mut self, record: &T) -> HddmResult<()> {
        Ok(record.write_hddm(&mut self.writer)?)
    }

    pub fn writer(&mut self) -> &mut HddmWriter<BufWriter<File>> {
        &mut self.writer
    }

    pub fn flush(&mut self) -> HddmResult<()> {
        Ok(self.writer.flush()?)
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
