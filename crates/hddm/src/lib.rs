pub(crate) mod error;
pub mod header;
pub(crate) mod model_match;
pub(crate) mod particles;
pub(crate) mod plan;
pub(crate) mod read;
pub(crate) mod write;
pub(crate) mod xdr;

pub mod prelude {
    pub use crate::{
        ChildPlan, Compression, ElementPlan, ElementReader, HddmError, HddmFileReader,
        HddmFileWriter, HddmModel, HddmPrimitiveRead, HddmPrimitiveWrite, HddmRead,
        HddmReadPlanned, HddmReader, HddmResult, HddmSchema, HddmWrite, HddmWriter, ModelPlan,
        build_model_plan, validate_models,
    };
}

use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};

pub use error::{HddmError, HddmResult};
#[cfg(feature = "derive")]
pub use hddm_derive::{HddmRead, HddmWrite};
pub use header::HddmModel;
pub use model_match::validate_models;
pub use particles::Particle;
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

pub struct HddmFile;
impl HddmFile {
    pub fn open<P: AsRef<Path>>(path: P) -> HddmResult<HddmFileReader> {
        HddmFileReader::open(path)
    }
    pub fn create<P: AsRef<Path>, S: AsRef<str>>(path: P, model: S) -> HddmResult<HddmFileWriter> {
        HddmFileWriter::new(
            path,
            WriteMode::Create {
                model: model.as_ref().to_string(),
            },
            Compression::Zlib,
        )
    }
    pub fn append<P: AsRef<Path>>(path: P) -> HddmResult<HddmFileWriter> {
        HddmFileWriter::new(path, WriteMode::Append, Compression::Zlib)
    }
}

pub struct HddmFileReader {
    pub header: HddmModel,
    records: HddmRecordReader<BufReader<File>>,
    plan_cache: HashMap<&'static str, ModelPlan>,
}
impl HddmFileReader {
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

impl HddmFileReader {
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
pub enum WriteMode {
    Create { model: String },
    Append,
}

impl HddmFileWriter {
    pub fn new<P: AsRef<Path>>(
        path: P,
        mode: WriteMode,
        compression: Compression,
    ) -> HddmResult<Self> {
        let file = match mode {
            WriteMode::Create { .. } => File::create(path)?,
            WriteMode::Append => File::options().append(true).open(path)?,
        };
        let mut raw = BufWriter::new(file);
        if let WriteMode::Create { model: header } = mode {
            raw.write_all(header.as_bytes())?;
        }
        let mut records = HddmRecordWriter::new(raw);
        if compression != Compression::None {
            records.switch_compression(compression)?;
        }
        Ok(Self { records })
    }

    pub fn write_record<T: HddmWrite>(&mut self, record: &T) -> HddmResult<()> {
        self.records.write_record(record)
    }

    pub fn with_compression(mut self, compression: Compression) -> HddmResult<Self> {
        self.records.switch_compression(compression)?;
        Ok(self)
    }

    pub fn set_compression(&mut self, compression: Compression) -> HddmResult<()> {
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
