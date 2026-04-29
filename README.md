# hddm-rs

Rust tooling for reading, writing, and generating bindings for the **HDDM (Hierarchical Data Description Model)** format used in GlueX.

This project provides:

- `hddm` — core runtime (read/write, compression, schema handling)
- `hddm-derive` — derive macros for creating
- `hddm-rs` — code generator + CLI for producing Rust bindings from HDDM headers

---

## Acknowledgements

While not a direct port of the original HDDM C++ code, it is important to recognize the work of Jefferson Lab contributors to the original [HDDM repository](https://github.com/JeffersonLab/HDDM) as well as the original work from Richard Jones in [his repository](https://github.com/rjones30/HDDM).

---

## Features

- Read/write HDDM files in Rust
- Support for zlib and bzip2 compression
- Code generation from HDDM headers
- Derive macros for manual models

---

## Quick Start

### Writing a file

```rust
let mut file = hddm::HddmFile::create(
    "/tmp/example.hddm",
    MODEL,
)?;
file.write_record(&event)?;
file.finish()?;
```

### Reading a file

```rust
let mut file = hddm::HddmFile::open("/tmp/example.hddm")?;

while let Some(event) = file.read_record::<MyType>()? {
    println!("{event:?}");
}
```

### Derive Macros

You can define your own HDDM-compatible types:

```rust
#[derive(Debug, PartialEq, hddm::HddmRead, hddm::HddmWrite)]
struct Course {
    credits: i32,
    title: String,
    result: Option<ResultElement>,
}
```

### Code Generation (`hddm-rs`)

You can generate a file with Rust bindings directly from an HDDM model:
```bash
hddm-rs sample_mc.hddm -o hddm_s.rs
```

This will generate a new Rust file that contains structs for each field in the model as well as some convenience methods for reading/writing files.

#### Using `build.rs`

Rust allows for custom scripts to run at build time by defining a `build.rs` file. An example of such a crate is located in `demo/`. The basic idea is as follows:

##### `build.rs`
```rust
use std::{
    env,
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

use hddm::header::read_header_streaming;
use hddm_rs::generate_rust;

fn main() -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let model_path = manifest_dir.join("models/sample_mc.hddm");
    println!("cargo:rerun-if-changed={}", model_path.display());
    let file = File::open(&model_path)?;
    let mut reader = BufReader::new(file);
    let (model, model_text) = read_header_streaming(&mut reader)?;
    let generated = generate_rust(&model, &model_text)?;
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let out_file = out_dir.join("hddm_s.rs");
    fs::write(&out_file, generated)?;
    Ok(())
}
```

##### `main.rs`
```rust
mod hddm_s {
    include!(concat!(env!("OUT_DIR"), "/hddm_s.rs"));
}
fn main() -> anyhow::Result<()> {
    let event = demo_event();

    let mut out = hddm_s::create(path)?.with_compression(hddm::Compression::Bzip2)?;

    out.write_record(&event)?;
    out.finish()?;

    let mut input = hddm_s::open(path)?;
    let decoded = input.read_record::<hddm_s::Hddm>()?;
    assert_eq!(decoded.as_ref(), Some(&event));
}
```
