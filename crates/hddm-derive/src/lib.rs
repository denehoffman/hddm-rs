use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(HddmWrite, attributes(hddm))]
pub fn derive_hddm_write(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_hddm_write(input).into()
}

fn outer_type_name(ty: &syn::Type) -> Option<String> {
    let syn::Type::Path(path) = ty else {
        return None;
    };

    path.path.segments.last().map(|seg| seg.ident.to_string())
}

fn field_kind(field: &syn::Field) -> FieldKind {
    match outer_type_name(&field.ty).as_deref() {
        Some("Option") => FieldKind::Link,
        Some("Vec") => FieldKind::List,
        _ => FieldKind::Attr,
    }
}

enum FieldKind {
    Attr,
    Link,
    List,
}

fn expand_hddm_write(input: DeriveInput) -> proc_macro2::TokenStream {
    let name = input.ident;
    let Data::Struct(data) = input.data else {
        return quote! {
            compile_error!("HdmWrite can only be derived for structs");
        };
    };

    let Fields::Named(fields) = data.fields else {
        return quote! {
            compile_error!("HddmWrite can only be derived for structs with named fields");
        };
    };

    let writes = fields.named.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        match field_kind(field) {
            FieldKind::Attr => quote! {
                ::hddm::HddmPrimitiveWrite::write_primitive(&self.#ident, w)?;
            },
            FieldKind::Link => quote! {
                w.write_link(&self.#ident)?;
            },
            FieldKind::List => quote! {
                w.write_list(&self.#ident)?;
            },
        }
    });

    quote! {
        impl ::hddm::HddmWrite for #name {
            fn write_contents<W: std::io::Write>(&self, w: &mut ::hddm::HddmWriter<W>) -> ::hddm::HddmResult<()> {
                #(#writes)*
                Ok(())
            }
        }
    }
}

#[proc_macro_derive(HddmRead, attributes(hddm))]
pub fn derive_hddm_read(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_hddm_read(input).into()
}

fn expand_hddm_read(input: DeriveInput) -> proc_macro2::TokenStream {
    let name = input.ident;
    let Data::Struct(data) = input.data else {
        return quote! {
            compile_error!("HddmRead can only be derived for structs");
        };
    };

    let Fields::Named(fields) = data.fields else {
        return quote! {
            compile_error!("HddmWrite can only be derived for structs with named fields");
        };
    };

    let reads = fields.named.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        match field_kind(field) {
            FieldKind::Attr => quote! {
                #ident: ::hddm::HddmPrimitiveRead::read_primitive(r)?,
            },
            FieldKind::Link => quote! {
                #ident: r.read_link()?,
            },
            FieldKind::List => quote! {
                #ident: r.read_list()?,
            },
        }
    });

    quote! {
        impl ::hddm::HddmRead for #name {
            fn read_contents(r: &mut ::hddm::ElementReader) -> ::hddm::HddmResult<Self> {
                Ok(Self {
                    #(#reads)*
                })
            }
        }
    }
}
