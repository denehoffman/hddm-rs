use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::bail;
use clap::Parser;
use hddm::header::{ElementDef, HddmModel};
use heck::{ToSnakeCase, ToUpperCamelCase};
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate_rust(model: &HddmModel, model_text: &str) -> anyhow::Result<String> {
    let mut structs: IndexMap<String, ElementDef> = IndexMap::new();
    collect_unique_structs(&model.root, &mut structs)?;

    let generated = structs.values().map(generate_struct);

    let hddm_class = model.class_name.as_deref().unwrap_or("");
    let model_lit = proc_macro2::Literal::string(model_text);
    let class_lit = proc_macro2::Literal::string(hddm_class);
    let root_ident = struct_ident(&model.root.name);

    let root_impl = quote! {
        impl #root_ident {
            pub fn writer<P: AsRef<std::path::Path>>(path: P) -> ::hddm::HddmResult<::hddm::HddmFileWriter> {
                ::hddm::HddmFileWriter::create(path, MODEL)
            }
        }
    };

    let tokens = quote! {
        #![allow(non_snake_case)]
        #![allow(non_camel_case_types)]

        pub const MODEL: &str = #model_lit;
        pub const HDDM_CLASS: &str = #class_lit;
        pub type Root = #root_ident;

        #(#generated)*

        #root_impl
    };

    let file = syn::parse_file(&tokens.to_string())?;
    Ok(prettyplease::unparse(&file))
}

fn collect_unique_structs(
    elem: &ElementDef,
    out: &mut IndexMap<String, ElementDef>,
) -> anyhow::Result<()> {
    for child in &elem.children {
        collect_unique_structs(child, out)?;
    }

    let name = struct_name_string(&elem.name);

    if let Some(existing) = out.get(&name) {
        if !same_struct_shape(existing, elem) {
            bail!("struct name collision for `{name}` with incompatible shapes");
        }
    } else {
        out.insert(name, elem.clone());
    }

    Ok(())
}

fn same_struct_shape(a: &ElementDef, b: &ElementDef) -> bool {
    a.name == b.name
        && a.attributes == b.attributes
        && a.children.len() == b.children.len()
        && a.children.iter().zip(&b.children).all(|(ac, bc)| {
            ac.name == bc.name && ac.min_occurs == bc.min_occurs && ac.max_occurs == bc.max_occurs
        })
}
fn generate_struct(elem: &ElementDef) -> TokenStream {
    let name = struct_ident(&elem.name);
    let attr_fields = elem.attributes.iter().map(|attr| {
        let field = field_ident(&attr.name);
        let ty = rust_type(&attr.ty);
        quote! {
            pub #field: #ty,
        }
    });

    let child_fields = elem.children.iter().map(|child| {
        let field = field_ident(&child.name);
        let child_ty = struct_ident(&child.name);
        let ty = if child.max_occurs.as_deref() == Some("unbounded") {
            quote! { Vec<#child_ty> }
        } else {
            quote! { Option<#child_ty> }
        };

        quote! {
            pub #field: #ty,
        }
    });

    quote! {
        #[derive(Debug, Clone, PartialEq, ::hddm::HddmRead, ::hddm::HddmWrite)]
        pub struct #name {
            #(#attr_fields)*
            #(#child_fields)*
        }
    }
}

fn struct_name_string(name: &str) -> String {
    if name == "HDDM" {
        "Hddm".to_string()
    } else {
        name.to_upper_camel_case()
    }
}

fn struct_ident(name: &str) -> proc_macro2::Ident {
    format_ident!("{}", struct_name_string(name))
}

fn field_ident(name: &str) -> proc_macro2::Ident {
    let field = name.to_snake_case();
    match field.as_str() {
        "type" => format_ident!("particle_type"),
        "self" => format_ident!("self_"),
        "match" => format_ident!("match_"),
        "ref" => format_ident!("ref_"),
        "crate" => format_ident!("crate_"),
        _ => format_ident!("{}", field),
    }
}

fn rust_type(ty: &str) -> TokenStream {
    match ty {
        "int" => quote! { i32 },
        "long" => quote! { i64 },
        "float" => quote! { f32 },
        "double" => quote! { f64 },
        "boolean" => quote! { bool },
        "string" | "anyURI" => quote! { String },
        "Particle_t" => quote! { ::gluex_core::Particle },
        other => quote! {
            compile_error!(concat!("unsupported HDDM type: ", #other))
        },
    }
}

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
