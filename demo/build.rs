use std::{env, fs, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let model_path = manifest_dir.join("models/sample_mc.hddm");
    println!("cargo:rerun-if-changed={}", model_path.display());
    let file = fs::File::open(&model_path)?;
    let mut reader = std::io::BufReader::new(file);
    let (model, model_text) = hddm::header::read_header_streaming(&mut reader)?;
    let generated = hddm_rs::generate_rust(&model, &model_text)?;
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let out_file = out_dir.join("hddm_s.rs");
    fs::write(&out_file, generated)?;
    Ok(())
}
