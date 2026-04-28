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

fn is_primitive_type(ty: &syn::Type) -> bool {
    let Some(name) = outer_type_name(ty) else {
        return false;
    };
    matches!(
        name.as_str(),
        "i32" | "u32" | "i64" | "u64" | "f32" | "f64" | "bool" | "String" | "Particle"
    )
}
fn field_kind(field: &syn::Field) -> FieldKind {
    match outer_type_name(&field.ty).as_deref() {
        Some("Option") => FieldKind::OptionalLink,
        Some("Vec") => FieldKind::List,
        _ if is_primitive_type(&field.ty) => FieldKind::Attr,
        _ => FieldKind::RequiredLink,
    }
}

enum FieldKind {
    Attr,
    OptionalLink,
    RequiredLink,
    List,
}

fn expand_hddm_write(input: DeriveInput) -> proc_macro2::TokenStream {
    let name = input.ident;
    let Data::Struct(data) = input.data else {
        return quote! {
            compile_error!("HddmWrite can only be derived for structs");
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
                ::hddm::HddmPrimitiveWrite::write_primitive(&self.#ident, __hddm_writer)?;
            },
            FieldKind::OptionalLink => quote! {
                __hddm_writer.write_link(&self.#ident)?;
            },
            FieldKind::RequiredLink => quote! {
                __hddm_writer.write_required_link(&self.#ident)?;
            },
            FieldKind::List => quote! {
                __hddm_writer.write_list(&self.#ident)?;
            },
        }
    });

    quote! {
        impl ::hddm::HddmWrite for #name {
            fn write_contents<W: std::io::Write>(&self, __hddm_writer: &mut ::hddm::HddmWriter<W>) -> ::hddm::HddmResult<()> {
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
                #ident: ::hddm::HddmPrimitiveRead::read_primitive(__hddm_reader)?,
            },
            FieldKind::OptionalLink => quote! {
                #ident: __hddm_reader.read_link()?,
            },
            FieldKind::RequiredLink => quote! {
                #ident: __hddm_reader.read_required_link()?,
            },
            FieldKind::List => quote! {
                #ident: __hddm_reader.read_list()?,
            },
        }
    });

    let planned_initializers = fields.named.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        match field_kind(field) {
            FieldKind::Attr => {
                quote! { let #ident = ::hddm::HddmPrimitiveRead::read_primitive(__hddm_reader)?; }
            }
            FieldKind::OptionalLink => quote! { let mut #ident = None; },
            FieldKind::RequiredLink => quote! { let mut #ident = None; },
            FieldKind::List => quote! { let mut #ident = Vec::new(); },
        }
    });
    let mut child_index = 0usize;
    let planned_matches = fields.named.iter().filter_map(|field| {
        let ident = field.ident.as_ref().unwrap();
        match field_kind(field) {
            FieldKind::Attr => None,
            FieldKind::OptionalLink => {
                let index = child_index;
                child_index += 1;
                Some(quote! {
                ::hddm::ChildPlan::Decode {
                    generated_index: #index,
                    plan: __hddm_child_plan,
                    ..
                } => {
                        #ident = __hddm_reader.read_link_planned(__hddm_child_plan)?;
                    }
                })
            }
            FieldKind::RequiredLink => {
                let index = child_index;
                child_index += 1;
                Some(quote! {
                ::hddm::ChildPlan::Decode {
                    generated_index: #index,
                    plan: __hddm_child_plan,
                    ..
                } => {
                        #ident = Some(__hddm_reader.read_required_link_planned(__hddm_child_plan)?);
                    }
                })
            }
            FieldKind::List => {
                let index = child_index;
                child_index += 1;
                Some(quote! {
                ::hddm::ChildPlan::Decode {
                    generated_index: #index,
                    plan: __hddm_child_plan,
                    ..
                } => {
                        #ident = __hddm_reader.read_list_planned(__hddm_child_plan)?;
                    }
                })
            }
        }
    });
    let field_names = fields.named.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        match field_kind(field) {
            FieldKind::RequiredLink => quote! {
                #ident: #ident.ok_or_else(|| {
                    ::hddm::HddmError::FormatError(
                        concat!("missing required HDDM child `", stringify!(#ident), "`").to_string())
                })?
            },
            _ => quote! {
                #ident
            },
        }
    });

    quote! {
        impl ::hddm::HddmRead for #name {
            fn read_contents(__hddm_reader: &mut ::hddm::ElementReader) -> ::hddm::HddmResult<Self> {
                Ok(Self {
                    #(#reads)*
                })
            }
        }
        impl ::hddm::HddmReadPlanned for #name {
            fn read_contents_planned(
                __hddm_reader: &mut ::hddm::ElementReader,
                __hddm_plan: &::hddm::ElementPlan,
            ) -> ::hddm::HddmResult<Self> {
                #(#planned_initializers)*

                for __hddm_child in &__hddm_plan.children {
                    match __hddm_child {
                        #(#planned_matches)*
                        ::hddm::ChildPlan::Skip { .. } => {
                            __hddm_reader.skip_element()?;
                        }
                        ::hddm::ChildPlan::Decode { generated_index, .. } => {
                            return Err(::hddm::HddmError::FormatError(format!("unexpected HDDM child index {generated_index}"
                            )));
                        }
                    }
                }

                Ok(Self {
                    #(#field_names,)*
                })

            }

        }
    }
}
