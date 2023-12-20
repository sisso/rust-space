extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, Data, Fields};

#[proc_macro_derive(SaveData)]
pub fn print_struct_info(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let struct_name = &input.ident;

    let Data::Struct(data_struct) = &input.data else {
        return syn::Error::new_spanned(&input, "SaveData macro only supports named structs")
            .to_compile_error()
            .into();
    };

    let Fields::Named(named_fields) = &data_struct.fields else {
        return syn::Error::new_spanned(&input, "SaveData macro only supports named structs")
            .to_compile_error()
            .into();
    };

    // collect fields and types
    let field_error = || {
        syn::Error::new_spanned(
            &data_struct.fields,
            "fail to parse Option<Component> type field",
        )
        .to_compile_error()
    };

    let mut fields = vec![];
    for field in &named_fields.named {
        let Some(field_name) = &field.ident else {
            return field_error().into();
        };

        // resolve component type from fields  Option<Component>
        let field_type = {
            let syn::Type::Path(type_path) = &field.ty else {
                return field_error().into();
            };
            let Some(segment) = type_path.path.segments.last() else {
                return field_error().into();
            };
            assert_eq!(segment.ident, "Option", "components fields must be option");
            let syn::PathArguments::AngleBracketed(angle_bracketed) = &segment.arguments else {
                return field_error().into();
            };
            let Some(inner_type) = angle_bracketed.args.first() else {
                return field_error().into();
            };
            inner_type
        };

        // skip index field
        if field_name.to_string().as_str() == "id" {
            continue;
        }

        fields.push((field_name, field_type));
    }

    let write_fields_code = fields
        .iter()
        .map(|(field_name, _field_type)| {
            quote! {
                {
                    // log::trace!("write {}: {}", stringify!(#field_name), stringify!(#field_type));
                    if let Some(value) = self.#field_name.take() {
                        entity.insert(value);
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let load_fields_code = fields
        .iter()
        .map(|(field_name, field_type)| {
            quote! {
                {
                    // println!("load {}: {}", stringify!(#field_name), stringify!(#field_type));
                    self.#field_name = entity.get::<#field_type>().cloned();
                }
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        impl #struct_name {
            pub fn load_from<'a, 'b>(&mut self, entity: &'b EntityRef<'a>) {
                self.id = Some(entity.id());
                #(
                    #load_fields_code
                )*
            }
            pub fn write_into<'a, 'b>(mut self, entity: &'b mut EntityWorldMut<'a>) {
                #(
                    #write_fields_code
                )*
            }
        }
    };

    expanded.into()
}
