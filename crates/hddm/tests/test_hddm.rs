use hddm::{HddmFile, HddmFileWriter, HddmRead, HddmWrite};

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
fn test_roundtrip() -> anyhow::Result<()> {
    let mut file = HddmFileWriter::create("rust-test.hddm", MODEL)?;

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
    let event1: Hddm = file.read_record()?.unwrap();
    let event2: Hddm = file.read_record()?.unwrap();
    assert_eq!(event1, event);
    assert_eq!(event2, event_alt);
    assert!(file.read_record::<Hddm>()?.is_none());

    Ok(())
}
