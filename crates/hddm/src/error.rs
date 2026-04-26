use thiserror::Error;

pub type HddmResult<T> = Result<T, HddmError>;

#[derive(Error, Debug)]
pub enum HddmError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    XmlError(#[from] quick_xml::Error),
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    AttrError(#[from] quick_xml::events::attributes::AttrError),
    #[error("invalid HDDM format: {0}")]
    FormatError(String),
    // #[error("unexpected EOF")]
    // Eof,
}
