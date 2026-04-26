use hddm::{Compression, HddmFile, HddmFileWriter, HddmRead, HddmResult, HddmWrite};

#[derive(Debug, PartialEq, HddmRead, HddmWrite)]
pub struct ResultElement {
    pub pass: bool,
    pub grade: String,
}

#[derive(Debug, PartialEq, HddmRead, HddmWrite)]
pub struct Course {
    pub credits: i32,
    pub title: String,
    pub result: Option<ResultElement>,
}

#[derive(Debug, PartialEq, HddmRead, HddmWrite)]
pub struct Enrolled {
    pub semester: i32,
    pub year: i32,
    pub courses: Vec<Course>,
}

#[derive(Debug, PartialEq, HddmRead, HddmWrite)]
pub struct Student {
    pub name: String,
    pub enrolled: Vec<Enrolled>,
}

#[derive(Debug, PartialEq, HddmRead, HddmWrite)]
pub struct Hddm {
    pub student: Option<Student>,
}

const MODEL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<HDDM class="x">
  <student name="string">
    <enrolled semester="int" year="int" maxOccurs="unbounded">
      <course credits="int" title="string" maxOccurs="unbounded">
        <result Pass="boolean" grade="string" />
      </course>
    </enrolled>
  </student>
</HDDM>
"#;

pub fn generate_events() -> Vec<Hddm> {
    vec![
        Hddm {
            student: Some(Student {
                name: "Dene".to_string(),
                enrolled: vec![Enrolled {
                    semester: 1,
                    year: 2026,
                    courses: vec![Course {
                        credits: 3,
                        title: "HDDM 101".to_string(),
                        result: Some(ResultElement {
                            pass: true,
                            grade: "A".to_string(),
                        }),
                    }],
                }],
            }),
        },
        Hddm {
            student: Some(Student {
                name: "Aditi".to_string(),
                enrolled: vec![Enrolled {
                    semester: 1,
                    year: 2026,
                    courses: vec![
                        Course {
                            credits: 3,
                            title: "Science 101".to_string(),
                            result: Some(ResultElement {
                                pass: true,
                                grade: "A+".to_string(),
                            }),
                        },
                        Course {
                            credits: 10,
                            title: "Engineering 500".to_string(),
                            result: None,
                        },
                    ],
                }],
            }),
        },
    ]
}

#[test]
fn test_roundtrip() -> HddmResult<()> {
    let filename = "/tmp/rust-test-roundtrip.hddm";
    let mut file = HddmFileWriter::create(filename, MODEL)?;

    let events = generate_events();
    file.write_record(&events[0])?;
    file.write_record(&events[1])?;
    file.finish()?;

    let mut file = HddmFile::open(filename)?;
    assert_eq!(file.read_record::<Hddm>()?.as_ref(), Some(&events[0]));
    assert_eq!(file.read_record::<Hddm>()?.as_ref(), Some(&events[1]));
    assert!(file.read_record::<Hddm>()?.is_none());
    Ok(())
}

#[test]
fn test_roundtrip_zlib() -> HddmResult<()> {
    let filename = "/tmp/rust-test-roundtrip-zlib.hddm";
    let mut file = HddmFileWriter::create_with_compression(filename, MODEL, Compression::Zlib)?;

    let events = generate_events();
    file.write_record(&events[0])?;
    file.write_record(&events[1])?;
    file.finish()?;

    let mut file = HddmFile::open(filename)?;
    assert_eq!(file.read_record::<Hddm>()?.as_ref(), Some(&events[0]));
    assert_eq!(file.read_record::<Hddm>()?.as_ref(), Some(&events[1]));
    assert!(file.read_record::<Hddm>()?.is_none());
    Ok(())
}

#[test]
fn test_roundtrip_bzip2() -> HddmResult<()> {
    let filename = "/tmp/rust-test-roundtrip-bzip2.hddm";
    let mut file = HddmFileWriter::create_with_compression(filename, MODEL, Compression::Bzip2)?;

    let events = generate_events();
    file.write_record(&events[0])?;
    file.write_record(&events[1])?;
    file.finish()?;

    let mut file = HddmFile::open(filename)?;
    assert_eq!(file.read_record::<Hddm>()?.as_ref(), Some(&events[0]));
    assert_eq!(file.read_record::<Hddm>()?.as_ref(), Some(&events[1]));
    assert!(file.read_record::<Hddm>()?.is_none());
    Ok(())
}

#[test]
fn test_compression_switching() -> HddmResult<()> {
    let filename = "/tmp/rust-test-compression-switching.hddm";
    let mut out = HddmFileWriter::create(filename, MODEL)?;
    let events = generate_events();

    out.write_record(&events[0])?;

    out.switch_compression(Compression::Zlib)?;
    out.write_record(&events[1])?;

    out.switch_compression(Compression::None)?;
    out.write_record(&events[0])?;

    out.switch_compression(Compression::Bzip2)?;
    out.write_record(&events[1])?;

    out.finish()?;

    let mut input = HddmFile::open(filename)?;

    assert_eq!(input.read_record::<Hddm>()?.as_ref(), Some(&events[0]));
    assert_eq!(input.read_record::<Hddm>()?.as_ref(), Some(&events[1]));
    assert_eq!(input.read_record::<Hddm>()?.as_ref(), Some(&events[0]));
    assert_eq!(input.read_record::<Hddm>()?.as_ref(), Some(&events[1]));
    assert_eq!(input.read_record::<Hddm>()?, None);
    Ok(())
}

#[test]
fn debug_manual_zlib_payload_decode() -> HddmResult<()> {
    use std::{convert::TryInto, io::Read};

    use flate2::read::ZlibDecoder;

    let path = "/tmp/rust-test.hddm";

    let mut out = HddmFileWriter::create_with_compression(path, MODEL, Compression::Zlib)?;

    let events = generate_events();
    out.write_record(&events[0])?;
    out.write_record(&events[1])?;
    out.finish()?;

    let bytes = std::fs::read(path)?;
    let (_header, mut offset) = hddm::header::read_hddm_header_from_bytes(&bytes)?;

    while offset < bytes.len() && bytes[offset].is_ascii_whitespace() {
        offset += 1;
    }

    let mut p = offset;

    fn read_i32_at(bytes: &[u8], p: &mut usize) -> i32 {
        let value = i32::from_be_bytes(bytes[*p..*p + 4].try_into().unwrap());
        *p += 4;
        value
    }

    let marker = read_i32_at(&bytes, &mut p);
    let token_size = read_i32_at(&bytes, &mut p);
    let format = read_i32_at(&bytes, &mut p);
    let status_bits = read_i32_at(&bytes, &mut p);

    println!(
        "marker={marker}, token_size={token_size}, format={format}, status_bits={status_bits:#x}"
    );

    let compressed_size = read_i32_at(&bytes, &mut p);
    println!("compressed_size={compressed_size}");

    let compressed = &bytes[p..p + compressed_size as usize];

    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    println!("decompressed_len={}", decompressed.len());
    println!(
        "decompressed bytes: {:02x?}",
        &decompressed[..decompressed.len().min(128)]
    );

    let mut q = 0;

    let rec1_size = read_i32_at(&decompressed, &mut q);
    let rec1_payload = decompressed[q..q + rec1_size as usize].to_vec();
    q += rec1_size as usize;

    let rec2_size = read_i32_at(&decompressed, &mut q);
    let rec2_payload = decompressed[q..q + rec2_size as usize].to_vec();
    q += rec2_size as usize;

    println!("rec1_size={rec1_size}, rec2_size={rec2_size}, final_q={q}");

    let mut e1 = hddm::ElementReader::from_payload(rec1_payload);
    let decoded1 = Hddm::read_contents(&mut e1)?;
    e1.ensure_empty()?;

    let mut e2 = hddm::ElementReader::from_payload(rec2_payload);
    let decoded2 = Hddm::read_contents(&mut e2)?;
    e2.ensure_empty()?;

    println!("decoded1={decoded1:?}");
    println!("decoded2={decoded2:?}");

    assert_eq!(decoded1, events[0]);
    assert_eq!(decoded2, events[1]);
    assert_eq!(q, decompressed.len());

    Ok(())
}

#[test]
fn debug_hddmfile_zlib_read_steps() -> HddmResult<()> {
    let path = "/tmp/rust-test.hddm";

    let mut out = HddmFileWriter::create_with_compression(path, MODEL, Compression::Zlib)?;

    let events = generate_events();
    out.write_record(&events[0])?;
    out.write_record(&events[1])?;
    out.finish()?;

    let mut input = HddmFile::open(path)?;

    let r1 = input.read_record::<Hddm>();
    println!("r1 = {r1:#?}");
    assert_eq!(r1?.as_ref(), Some(&events[0]));

    let r2 = input.read_record::<Hddm>();
    println!("r2 = {r2:#?}");
    assert_eq!(r2?.as_ref(), Some(&events[1]));

    let r3 = input.read_record::<Hddm>();
    println!("r3 = {r3:#?}");
    assert!(r3?.is_none());

    Ok(())
}
