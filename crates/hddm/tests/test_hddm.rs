use hddm::{Compression, HddmFile, HddmRead, HddmResult, HddmSchema, HddmWrite};

const MODEL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<HDDM class="x">
  <student name="string">
    <enrolled semester="int" year="int" maxOccurs="unbounded">
      <course credits="int" title="string" maxOccurs="unbounded">
        <result minOccurs="0" Pass="boolean" grade="string" />
      </course>
    </enrolled>
  </student>
</HDDM>
"#;

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

impl HddmSchema for Hddm {
    fn model_text() -> &'static str {
        MODEL
    }
    fn hddm_class() -> &'static str {
        "x"
    }
    fn model() -> &'static ::hddm::HddmModel {
        static MODEL_PARSED: std::sync::OnceLock<::hddm::HddmModel> = std::sync::OnceLock::new();
        MODEL_PARSED.get_or_init(|| {
            ::hddm::header::read_hddm_header_from_bytes(MODEL.as_bytes())
                .expect("generated HDDM model should parse")
                .0
        })
    }
}

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
    let path = tempfile::NamedTempFile::new()?;
    let path = path.path();
    let mut file = HddmFile::create(path, MODEL)?.with_compression(Compression::None)?;

    let events = generate_events();
    file.write_record(&events[0])?;
    file.write_record(&events[1])?;
    file.finish()?;

    let mut file = HddmFile::open(path)?;
    assert_eq!(file.read_record::<Hddm>()?.as_ref(), Some(&events[0]));
    assert_eq!(file.read_record::<Hddm>()?.as_ref(), Some(&events[1]));
    assert!(file.read_record::<Hddm>()?.is_none());
    Ok(())
}

#[test]
fn test_roundtrip_zlib() -> HddmResult<()> {
    let path = tempfile::NamedTempFile::new()?;
    let path = path.path();
    let mut file = HddmFile::create(path, MODEL)?.with_compression(Compression::Zlib)?;

    let events = generate_events();
    file.write_record(&events[0])?;
    file.write_record(&events[1])?;
    file.finish()?;

    let mut file = HddmFile::open(path)?;
    assert_eq!(file.read_record::<Hddm>()?.as_ref(), Some(&events[0]));
    assert_eq!(file.read_record::<Hddm>()?.as_ref(), Some(&events[1]));
    assert!(file.read_record::<Hddm>()?.is_none());
    Ok(())
}

#[test]
fn test_roundtrip_bzip2() -> HddmResult<()> {
    let path = tempfile::NamedTempFile::new()?;
    let path = path.path();
    let mut file = HddmFile::create(path, MODEL)?.with_compression(Compression::Bzip2)?;

    let events = generate_events();
    file.write_record(&events[0])?;
    file.write_record(&events[1])?;
    file.finish()?;

    let mut file = HddmFile::open(path)?;
    assert_eq!(file.read_record::<Hddm>()?.as_ref(), Some(&events[0]));
    assert_eq!(file.read_record::<Hddm>()?.as_ref(), Some(&events[1]));
    assert!(file.read_record::<Hddm>()?.is_none());
    Ok(())
}

#[test]
fn test_compression_switching() -> HddmResult<()> {
    let path = tempfile::NamedTempFile::new()?;
    let path = path.path();
    let mut out = HddmFile::create(path, MODEL)?.with_compression(Compression::None)?;
    let events = generate_events();

    out.write_record(&events[0])?;

    out.set_compression(Compression::Zlib)?;
    out.write_record(&events[1])?;

    out.set_compression(Compression::None)?;
    out.write_record(&events[0])?;

    out.set_compression(Compression::Bzip2)?;
    out.write_record(&events[1])?;

    out.finish()?;

    let mut input = HddmFile::open(path)?;

    assert_eq!(input.read_record::<Hddm>()?.as_ref(), Some(&events[0]));
    assert_eq!(input.read_record::<Hddm>()?.as_ref(), Some(&events[1]));
    assert_eq!(input.read_record::<Hddm>()?.as_ref(), Some(&events[0]));
    assert_eq!(input.read_record::<Hddm>()?.as_ref(), Some(&events[1]));
    assert_eq!(input.read_record::<Hddm>()?, None);
    Ok(())
}

const MODEL_WITH_EXTRA: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<HDDM class="x">
  <student name="string">
    <advisor name="string" />
    <enrolled semester="int" year="int" maxOccurs="unbounded">
      <course credits="int" title="string" maxOccurs="unbounded">
        <result minOccurs="0" Pass="boolean" grade="string" />
      </course>
    </enrolled>
  </student>
</HDDM>
"#;

#[derive(Debug, PartialEq, HddmRead, HddmWrite)]
struct HddmExtra {
    student: Option<StudentExtra>,
}

#[derive(Debug, PartialEq, HddmRead, HddmWrite)]
struct StudentExtra {
    name: String,
    advisor: Option<Advisor>,
    enrolled: Vec<Enrolled>,
}

#[derive(Debug, PartialEq, HddmRead, HddmWrite)]
struct Advisor {
    name: String,
}

impl HddmSchema for HddmExtra {
    fn hddm_class() -> &'static str {
        "x"
    }
    fn model_text() -> &'static str {
        MODEL_WITH_EXTRA
    }
    fn model() -> &'static hddm::header::HddmModel {
        static MODEL_PARSED: std::sync::OnceLock<hddm::header::HddmModel> =
            std::sync::OnceLock::new();
        MODEL_PARSED.get_or_init(|| {
            hddm::header::read_hddm_header_from_bytes(MODEL_WITH_EXTRA.as_bytes())
                .unwrap()
                .0
        })
    }
}

#[test]
fn skips_unknown_child_from_file_schema() -> HddmResult<()> {
    let path = tempfile::NamedTempFile::new()?;
    let path = path.path();

    let extra = HddmExtra {
        student: Some(StudentExtra {
            name: "Dene".into(),
            advisor: Some(Advisor {
                name: "Dr. X".into(),
            }),
            enrolled: vec![Enrolled {
                semester: 1,
                year: 2026,
                courses: vec![Course {
                    credits: 3,
                    title: "HDDM 101".into(),
                    result: Some(ResultElement {
                        pass: true,
                        grade: "A".into(),
                    }),
                }],
            }],
        }),
    };

    let expected = Hddm {
        student: Some(Student {
            name: "Dene".into(),
            enrolled: vec![Enrolled {
                semester: 1,
                year: 2026,
                courses: vec![Course {
                    credits: 3,
                    title: "HDDM 101".into(),
                    result: Some(ResultElement {
                        pass: true,
                        grade: "A".into(),
                    }),
                }],
            }],
        }),
    };

    let mut out = HddmFile::create(path, MODEL_WITH_EXTRA)?;
    out.write_record(&extra)?;
    out.finish()?;

    let mut input = HddmFile::open(path)?;

    assert_eq!(input.read_record::<Hddm>()?, Some(expected));
    assert!(input.read_record::<Hddm>()?.is_none());

    Ok(())
}

const MODEL_MISSING_OPTIONAL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<HDDM class="x">
  <student name="string">
    <enrolled semester="int" year="int" maxOccurs="unbounded">
      <course credits="int" title="string" maxOccurs="unbounded">
      </course>
    </enrolled>
  </student>
</HDDM>
"#;

#[derive(Debug, PartialEq, HddmRead, HddmWrite)]
struct HddmMissingOptional {
    student: Option<StudentMissingOptional>,
}
#[derive(Debug, PartialEq, HddmRead, HddmWrite)]
struct StudentMissingOptional {
    name: String,
    enrolled: Vec<EnrolledMissingOptional>,
}
#[derive(Debug, PartialEq, HddmRead, HddmWrite)]
struct EnrolledMissingOptional {
    semester: i32,
    year: i32,
    courses: Vec<CourseMissingOptional>,
}
#[derive(Debug, PartialEq, HddmRead, HddmWrite)]
struct CourseMissingOptional {
    credits: i32,
    title: String,
}
impl HddmSchema for HddmMissingOptional {
    fn hddm_class() -> &'static str {
        "x"
    }
    fn model_text() -> &'static str {
        MODEL_MISSING_OPTIONAL
    }
    fn model() -> &'static hddm::header::HddmModel {
        static MODEL_PARSED: std::sync::OnceLock<hddm::header::HddmModel> =
            std::sync::OnceLock::new();
        MODEL_PARSED.get_or_init(|| {
            hddm::header::read_hddm_header_from_bytes(MODEL_MISSING_OPTIONAL.as_bytes())
                .unwrap()
                .0
        })
    }
}

#[test]
fn defaults_missing_optional_child() -> HddmResult<()> {
    let path = tempfile::NamedTempFile::new()?;
    let path = path.path();
    let written = HddmMissingOptional {
        student: Some(StudentMissingOptional {
            name: "Dene".into(),
            enrolled: vec![EnrolledMissingOptional {
                semester: 1,
                year: 2026,
                courses: vec![CourseMissingOptional {
                    credits: 3,
                    title: "HDDM 101".into(),
                }],
            }],
        }),
    };
    let expected = Hddm {
        student: Some(Student {
            name: "Dene".into(),
            enrolled: vec![Enrolled {
                semester: 1,
                year: 2026,
                courses: vec![Course {
                    credits: 3,
                    title: "HDDM 101".into(),
                    result: None,
                }],
            }],
        }),
    };
    let mut out = HddmFile::create(path, MODEL_MISSING_OPTIONAL)?;
    out.write_record(&written)?;
    out.finish()?;
    let mut input = HddmFile::open(path)?;
    assert_eq!(input.read_record::<Hddm>()?, Some(expected));
    Ok(())
}

const MODEL_ATTR_MISMATCH: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<HDDM class="x">
  <student full_name="string">
    <enrolled semester="int" year="int" maxOccurs="unbounded">
      <course credits="int" title="string" maxOccurs="unbounded">
        <result Pass="boolean" grade="string" />
      </course>
    </enrolled>
  </student>
</HDDM>
"#;

#[derive(Debug, PartialEq, HddmRead, HddmWrite)]
struct HddmAttrMismatch {
    student: Option<StudentAttrMismatch>,
}

#[derive(Debug, PartialEq, HddmRead, HddmWrite)]
struct StudentAttrMismatch {
    full_name: String,
    enrolled: Vec<Enrolled>,
}

impl HddmSchema for HddmAttrMismatch {
    fn hddm_class() -> &'static str {
        "x"
    }

    fn model_text() -> &'static str {
        MODEL_ATTR_MISMATCH
    }

    fn model() -> &'static hddm::header::HddmModel {
        static MODEL_PARSED: std::sync::OnceLock<hddm::header::HddmModel> =
            std::sync::OnceLock::new();

        MODEL_PARSED.get_or_init(|| {
            hddm::header::read_hddm_header_from_bytes(MODEL_ATTR_MISMATCH.as_bytes())
                .unwrap()
                .0
        })
    }
}

#[test]
fn errors_on_attribute_mismatch() -> HddmResult<()> {
    let path = tempfile::NamedTempFile::new()?;
    let path = path.path();

    let written = HddmAttrMismatch {
        student: Some(StudentAttrMismatch {
            full_name: "Dene".into(),
            enrolled: Vec::new(),
        }),
    };

    let mut out = HddmFile::create(path, MODEL_ATTR_MISMATCH)?;
    out.write_record(&written)?;
    out.finish()?;

    let mut input = HddmFile::open(path)?;
    let err = input.read_record::<Hddm>().unwrap_err();

    assert!(format!("{err}").contains("attribute mismatch"));

    Ok(())
}
