use std::io::BufRead;

use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};

use crate::error::{HddmError, HddmResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HddmModel {
    pub class_name: Option<String>,
    pub version: Option<String>,
    pub root: ElementDef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementDef {
    pub name: String,
    pub min_occurs: Option<String>,
    pub max_occurs: Option<String>,
    pub attributes: Vec<AttributeDef>,
    pub children: Vec<ElementDef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeDef {
    pub name: String,
    pub ty: String,
}

pub fn read_header_streaming<R: BufRead>(reader: &mut R) -> HddmResult<(HddmModel, String)> {
    let mut header = Vec::new();
    let needle = b"</HDDM>";

    loop {
        let buf = reader.fill_buf()?;
        if buf.is_empty() {
            return Err(HddmError::FormatError(
                "reached EOF before complete HDDM header".into(),
            ));
        }

        if let Some(pos) = buf.windows(needle.len()).position(|w| w == needle) {
            let end = pos + needle.len();

            // include closing tag
            header.extend_from_slice(&buf[..end]);
            reader.consume(end);

            // consume trailing XML whitespace
            loop {
                let buf = reader.fill_buf()?;
                if buf.is_empty() {
                    break;
                }
                let n = buf.iter().take_while(|b| b.is_ascii_whitespace()).count();
                if n == 0 {
                    break;
                }
                header.extend_from_slice(&buf[..n]);
                reader.consume(n);
            }

            let (model, _) = read_hddm_header_from_bytes(&header)?;
            let model_text =
                String::from_utf8(header).map_err(|e| HddmError::FormatError(e.to_string()))?;

            return Ok((model, model_text));
        } else {
            header.extend_from_slice(buf);
            let n = buf.len();
            reader.consume(n);
        }
    }
}
pub fn read_hddm_header_from_bytes(bytes: &[u8]) -> HddmResult<(HddmModel, usize)> {
    let mut xml = Reader::from_reader(bytes);
    xml.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut stack: Vec<ElementDef> = Vec::new();

    let mut class_name = None;
    let mut version = None;

    loop {
        buf.clear();

        match xml.read_event_into(&mut buf)? {
            Event::Start(e) => {
                if e.name().as_ref() == b"HDDM" {
                    class_name = get_attr(&e, b"class")?;
                    version = get_attr(&e, b"version")?;
                }

                stack.push(parse_element(e)?);
            }

            Event::Empty(e) => {
                let elem = parse_element(e)?;

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(elem);
                } else {
                    return Err(HddmError::FormatError(
                        "unexpected empty element outside root".to_string(),
                    ));
                }
            }

            Event::End(e) => {
                let closing_name = bytes_to_string(e.name().as_ref())?;

                let elem = stack.pop().ok_or_else(|| {
                    HddmError::FormatError(format!("unexpected closing tag </{}>", closing_name))
                })?;

                if elem.name != closing_name {
                    return Err(HddmError::FormatError(format!(
                        "mismatched closing tag: expected </{}>, got </{}>",
                        elem.name, closing_name
                    )));
                }

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(elem);
                } else {
                    let offset = xml.buffer_position() as usize;

                    return Ok((
                        HddmModel {
                            class_name,
                            version,
                            root: elem,
                        },
                        offset,
                    ));
                }
            }

            Event::Eof => {
                return Err(HddmError::FormatError(
                    "reached EOF before complete HDDM header".to_string(),
                ));
            }

            Event::Decl(_)
            | Event::Text(_)
            | Event::Comment(_)
            | Event::CData(_)
            | Event::PI(_)
            | Event::DocType(_)
            | Event::GeneralRef(_) => {}
        }
    }
}

fn parse_element(e: BytesStart<'_>) -> HddmResult<ElementDef> {
    let name = bytes_to_string(e.name().as_ref())?;

    let mut min_occurs = None;
    let mut max_occurs = None;
    let mut attributes = Vec::new();

    for attr in e.attributes() {
        let attr = attr.map_err(|err| HddmError::FormatError(err.to_string()))?;

        let key = bytes_to_string(attr.key.as_ref())?;
        let value = bytes_to_string(attr.value.as_ref())?;

        match key.as_str() {
            "minOccurs" => min_occurs = Some(value),
            "maxOccurs" => max_occurs = Some(value),
            "class" | "version" | "xmlns" => {}
            _ => attributes.push(AttributeDef {
                name: key,
                ty: value,
            }),
        }
    }

    Ok(ElementDef {
        name,
        min_occurs,
        max_occurs,
        attributes,
        children: Vec::new(),
    })
}

fn get_attr(e: &BytesStart<'_>, key: &[u8]) -> HddmResult<Option<String>> {
    for attr in e.attributes() {
        let attr = attr.map_err(|err| HddmError::FormatError(err.to_string()))?;

        if attr.key.as_ref() == key {
            return Ok(Some(bytes_to_string(attr.value.as_ref())?));
        }
    }

    Ok(None)
}

fn bytes_to_string(bytes: &[u8]) -> HddmResult<String> {
    Ok(String::from_utf8(bytes.to_vec())?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<HDDM class="x" version="1.0" xmlns="http://www.gluex.org/hddm">
  <student minOccurs="0" name="string">
    <enrolled maxOccurs="unbounded" semester="int" year="int">
      <course credits="int" maxOccurs="unbounded" title="string">
        <result Pass="boolean" grade="string" />
      </course>
    </enrolled>
  </student>
</HDDM>
"#;

    #[test]
    fn parses_header() {
        let (model, offset) = read_hddm_header_from_bytes(MODEL.as_bytes()).unwrap();

        assert_eq!(offset + 1, MODEL.len());
        assert_eq!(model.class_name.as_deref(), Some("x"));
        assert_eq!(model.version.as_deref(), Some("1.0"));
        assert_eq!(model.root.name, "HDDM");
        assert_eq!(model.root.children.len(), 1);

        let student = &model.root.children[0];
        assert_eq!(student.name, "student");
        assert_eq!(student.min_occurs.as_deref(), Some("0"));
        assert_eq!(student.attributes[0].name, "name");
        assert_eq!(student.attributes[0].ty, "string");

        let enrolled = &student.children[0];
        assert_eq!(enrolled.name, "enrolled");
        assert_eq!(enrolled.max_occurs.as_deref(), Some("unbounded"));
    }

    #[test]
    fn returns_offset_before_binary_payload() {
        let mut bytes = MODEL.as_bytes().to_vec();
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]);

        let (_model, offset) = read_hddm_header_from_bytes(&bytes).unwrap();

        assert_eq!(&bytes[offset + 1..], &[0x00, 0x00, 0x00, 0x04]);
    }
}
