use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use hddm_rs::generate_rust;

#[derive(Debug, Parser)]
#[command(name = "hddm-rs", version, about = "Generate Rust HDDM model bindings")]
pub struct Cli {
    /// Validate the HDDM model only; do not generate code
    #[arg(short = 'v', long = "validate")]
    pub validate: bool,

    /// Output basename or file path
    ///
    /// If omitted, generated Rust is written to stdout.
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    /// HDDM model/header file
    pub input: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let file = File::open(&cli.input)?;
    let mut reader = BufReader::new(file);
    let (model, model_text) = hddm::header::read_header_streaming(&mut reader)?;
    if cli.validate {
        unimplemented!("Validation is not yet implemented");
    }

    let generated = generate_rust(&model, &model_text)?;

    if let Some(output) = cli.output {
        std::fs::write(output, generated)?; // TODO: check if exists?
    } else {
        print!("{generated}");
    }
    Ok(())
}
