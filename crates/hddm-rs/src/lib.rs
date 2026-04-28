use anyhow::bail;
use hddm::header::{ElementDef, HddmModel};
use heck::{ToSnakeCase, ToUpperCamelCase};
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

fn raw_string_literal(s: &str) -> proc_macro2::TokenStream {
    let mut hashes = 1;
    while s.contains(&"#".repeat(hashes)) {
        hashes += 1;
    }
    let delim = "#".repeat(hashes);
    let lit = format!(r#"r{delim}"{s}"{delim}"#);
    lit.parse().unwrap()
}

pub fn generate_rust(model: &HddmModel, model_text: &str) -> anyhow::Result<String> {
    let mut structs: IndexMap<String, ElementDef> = IndexMap::new();
    collect_unique_structs(&model.root, &mut structs)?;

    let generated = structs.values().map(generate_struct);

    let hddm_class = model.class_name.as_deref().unwrap_or("");
    let model_lit = raw_string_literal(model_text);
    let class_lit = proc_macro2::Literal::string(hddm_class);
    let root_ident = struct_ident(&model.root.name);

    let tokens = quote! {
        use ::hddm::{HddmRead, HddmWrite};

        #[allow(dead_code)]
        pub const MODEL: &str = #model_lit;
        #[allow(dead_code)]
        pub const HDDM_CLASS: &str = #class_lit;
        #[allow(dead_code)]
        pub type Root = #root_ident;

        #(#generated)*

        #[allow(dead_code)]
        pub fn create<P: AsRef<std::path::Path>>(path: P) -> ::hddm::HddmResult<::hddm::HddmFileWriter> {
            ::hddm::HddmFileWriter::create(path, MODEL)
        }
        #[allow(dead_code)]
        pub fn create_with_compression<P: AsRef<std::path::Path>>(path: P, compression: ::hddm::Compression) -> ::hddm::HddmResult<::hddm::HddmFileWriter> {
            ::hddm::HddmFileWriter::create_with_compression(path, MODEL, compression)
        }
        #[allow(dead_code)]
        pub fn open<P: AsRef<std::path::Path>>(path: P) -> ::hddm::HddmResult<::hddm::HddmFile<std::io::BufReader<std::fs::File>>> {
            ::hddm::HddmFile::open(path)
        }

        impl ::hddm::HddmSchema for #root_ident {
            fn model_text() -> &'static str {
                MODEL
            }
            fn hddm_class() -> &'static str {
                HDDM_CLASS
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
        } else if child.min_occurs.as_deref() == Some("0") {
            quote! { Option<#child_ty> }
        } else {
            quote! { #child_ty }
        };

        quote! {
            pub #field: #ty,
        }
    });

    quote! {
        #[allow(dead_code)]
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, PartialEq, HddmRead, HddmWrite)]
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
    let mut field = name.to_snake_case();
    while syn::parse_str::<syn::Ident>(&field).is_err() {
        field.push('_');
    }
    format_ident!("{}", field)
}

fn rust_type(ty: &str) -> TokenStream {
    match ty {
        "int" => quote! { i32 },
        "long" => quote! { i64 },
        "float" => quote! { f32 },
        "double" => quote! { f64 },
        "boolean" => quote! { bool },
        "string" | "anyURI" => quote! { String },
        "Particle_t" => quote! { ::hddm::Particle },
        other => quote! {
            compile_error!(concat!("unsupported HDDM type: ", #other))
        },
    }
}
