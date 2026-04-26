pub(crate) mod error;
pub mod header;
pub(crate) mod model_match;
pub(crate) mod particles;
pub(crate) mod plan;
pub(crate) mod read;
pub(crate) mod write;
pub(crate) mod xdr;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

pub use error::{HddmError, HddmResult};
#[cfg(feature = "derive")]
pub use hddm_derive::{HddmRead, HddmWrite};
pub use header::HddmModel;
pub use model_match::validate_models;
pub use plan::{ChildPlan, ElementPlan, ModelPlan, build_model_plan};
pub use read::{ElementReader, HddmPrimitiveRead, HddmRead, HddmReadPlanned, HddmReader};
pub use write::{HddmPrimitiveWrite, HddmWrite, HddmWriter};

use crate::{header::read_header_streaming, read::HddmRecordReader, write::HddmRecordWriter};

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
    plan_cache: HashMap<&'static str, ModelPlan>,
}
impl HddmFile<BufReader<File>> {
    pub fn open<P: AsRef<Path>>(path: P) -> HddmResult<Self> {
        let file = File::open(path)?;
        let mut raw = BufReader::new(file);
        let (header, _) = read_header_streaming(&mut raw)?;
        Ok(Self {
            header,
            records: HddmRecordReader::new(raw),
            plan_cache: HashMap::new(),
        })
    }
}

impl<R: BufRead> HddmFile<R> {
    pub fn header(&self) -> &HddmModel {
        &self.header
    }

    pub fn read_record<T>(&mut self) -> HddmResult<Option<T>>
    where
        T: HddmRead + HddmReadPlanned + HddmSchema,
    {
        let Some(payload) = self.records.next_record_payload()? else {
            return Ok(None);
        };
        let mut element = ElementReader::from_payload(payload);
        let value = if &self.header == T::model() {
            T::read_contents(&mut element)?
        } else {
            let key = T::hddm_class();
            if !self.plan_cache.contains_key(key) {
                let plan = build_model_plan(&self.header, T::model())?;
                self.plan_cache.insert(key, plan);
            }
            let plan = self.plan_cache.get(key).unwrap();
            T::read_contents_planned(&mut element, &plan.root)?
        };
        element.ensure_empty()?;
        Ok(Some(value))
    }
}

pub struct HddmFileWriter {
    records: HddmRecordWriter<BufWriter<File>>,
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

        let mut records = HddmRecordWriter::new(raw);

        if compression != Compression::None {
            records.switch_compression(compression)?;
        }

        Ok(Self { records })
    }

    pub fn write_record<T: HddmWrite>(&mut self, record: &T) -> HddmResult<()> {
        self.records.write_record(record)
    }

    pub fn switch_compression(&mut self, compression: Compression) -> HddmResult<()> {
        self.records.switch_compression(compression)
    }

    pub fn flush(&mut self) -> HddmResult<()> {
        self.records.flush()
    }

    pub fn finish(&mut self) -> HddmResult<()> {
        self.records.flush()
    }
}

impl Drop for HddmFileWriter {
    fn drop(&mut self) {
        self.flush().unwrap();
    }
}

pub trait HddmSchema {
    fn model() -> &'static HddmModel;
    fn model_text() -> &'static str;
    fn hddm_class() -> &'static str;
}
