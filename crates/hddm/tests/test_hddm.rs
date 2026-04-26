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

#[test]
fn test_roundtrip() -> HddmResult<()> {
    let mut file = HddmFileWriter::create("/tmp/rust-test.hddm", MODEL)?;

    let event = Hddm {
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
    };
    let event_alt = Hddm {
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
    };
    file.write_record(&event)?;
    file.write_record(&event_alt)?;
    file.finish()?;

    let bytes = std::fs::read("/tmp/rust-test.hddm")?;
    println!("Bytes: {:02x?}", &bytes[..bytes.len().min(64)]);
    println!(
        "{}",
        String::from_utf8_lossy(&bytes[..bytes.len().min(128)])
    );
    let mut file = HddmFile::open("/tmp/rust-test.hddm")?;
    assert_eq!(file.read_record::<Hddm>()?, Some(event));
    assert_eq!(file.read_record::<Hddm>()?, Some(event_alt));
    assert!(file.read_record::<Hddm>()?.is_none());
    Ok(())
}

#[test]
fn test_compression_roundtrip() -> HddmResult<()> {
    let mut file =
        HddmFileWriter::create_with_compression("/tmp/rust-test.hddm", MODEL, Compression::Zlib)?;
    let event = Hddm {
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
    };
    let event_alt = Hddm {
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
    };
    file.write_record(&event)?;
    file.write_record(&event_alt)?;
    file.finish()?;

    let mut file = HddmFile::open("/tmp/rust-test.hddm")?;
    assert_eq!(file.read_record::<Hddm>()?, Some(event));
    assert_eq!(file.read_record::<Hddm>()?, Some(event_alt));
    assert!(file.read_record::<Hddm>()?.is_none());
    Ok(())
}
