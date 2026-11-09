use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Visibility
};

#[proc_macro_attribute]
pub fn publics(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(item as DeriveInput);

    // 1. Isolated scope for modifications to prevent borrow conflicts
    let is_enum = match &mut ast.data {
        Data::Struct(data_struct) => {
            for field in &mut data_struct.fields {
                field.vis = Visibility::Public(syn::token::Pub::default());
            }
            false
        }
        Data::Enum(_) => true,
        Data::Union(_) => unimplemented!("Unions are not supported by #[publics]"),
    };

    // 2. Extract immutable references from the root AST components
    let name = &ast.ident;
    let attrs = &ast.attrs;
    let vis = &ast.vis;
    
    // Decompose structural generics to safely append our new T0, T1 parameters
    let generics = &ast.generics;
    let struct_generic_params = &generics.params; // e.g., 'A, B' or empty
    let (_, ty_generics, where_clause) = generics.split_for_impl();

    // Determine leading commas dynamically based on whether structural parameters exist
    let leading_comma = if struct_generic_params.is_empty() {
        quote! {}
    } else {
        quote! { , }
    };

    let expanded = if !is_enum {
        // ==================== STRUCTS HANDLER ====================
        let data_struct = match &ast.data {
            Data::Struct(s) => s,
            _ => unreachable!(),
        };

        let fields = &data_struct.fields;
        let is_named = matches!(fields, Fields::Named(_));

        let generic_t_idents: Vec<syn::Ident> = fields
            .iter()
            .enumerate()
            .map(|(idx, _)| quote::format_ident!("T{}", idx))
            .collect();

        let tuple_types: Vec<&syn::Type> = fields.iter().map(|f| &f.ty).collect();

        let try_assignments: Vec<_> = fields.iter().enumerate().map(|(idx, f)| {
            let index = syn::Index::from(idx);
            let err = format!("Error: Field conversion failed at position {idx}.");
            if is_named {
                let field_ident = &f.ident;
                quote! { #field_ident: value.#index.try_into().map_err(|_| #err)? }
            } else {
                quote! { value.#index.try_into().map_err(|_| #err)? }
            }
        }).collect();

        let instantiation = if is_named {
            quote! { #name { #(#try_assignments,)* } }
        } else {
            quote! { #name ( #(#try_assignments,)* ) }
        };

        let struct_fields_tokens = &data_struct.fields;
        let semi_token = if is_named { quote! {} } else { quote! { ; } };

        quote! {
            // Reconstruct the struct declaration
            #(#attrs)*
            #vis struct #name #ty_generics #struct_fields_tokens #semi_token

            // FIX: Mix structural parameters and macro parameters directly inside the impl declaration
            impl < #struct_generic_params #leading_comma #(#generic_t_idents),* > std::convert::TryFrom<(#(#generic_t_idents),*)> for #name #ty_generics
            #where_clause
            where
                #(#tuple_types: std::convert::TryFrom<#generic_t_idents>,)*
            {
                type Error = &'static str;
                fn try_from(value: (#(#generic_t_idents),*)) -> std::result::Result<#name #ty_generics, Self::Error> {
                    std::result::Result::Ok(#instantiation)
                }
            }
        }

    } else {
        // ==================== ENUMS HANDLER ====================
        let data_enum = match &ast.data {
            Data::Enum(e) => e,
            _ => unreachable!(),
        };

        let mut enum_from_impls = Vec::new();

        for variant in &data_enum.variants {
            let variant_ident = &variant.ident;
            let fields = &variant.fields;
            let is_named = matches!(fields, Fields::Named(_));

            if matches!(fields, Fields::Unit) {
                continue;
            }

            let tuple_types: Vec<&syn::Type> = fields.iter().map(|f| &f.ty).collect();
            
            let temp_idents: Vec<syn::Ident> = fields
                .iter()
                .enumerate()
                .map(|(idx, _)| quote::format_ident!("v{}", idx))
                .collect();

            let from_assignments: Vec<_> = fields.iter().enumerate().map(|(idx, f)| {
                let temp_id = &temp_idents[idx];
                if is_named {
                    let field_ident = &f.ident;
                    quote! { #field_ident: #temp_id.into() }
                } else {
                    quote! { #temp_id.into() }
                }
            }).collect();

            let variant_instantiation = if is_named {
                quote! { #name::#variant_ident { #(#from_assignments,)* } }
            } else {
                quote! { #name::#variant_ident ( #(#from_assignments,)* ) }
            };

            let marker_ident = quote::format_ident!("{}{}Payload", name, variant_ident);

            enum_from_impls.push(quote! {
                #[derive(Debug, Clone)]
                pub struct #marker_ident(pub #(#tuple_types,)*);

                // FIX: Support enums carrying structural parameters
                impl < #struct_generic_params > std::convert::From<#marker_ident> for #name #ty_generics #where_clause {
                    fn from(marker: #marker_ident) -> Self {
                        let #marker_ident(#(#temp_idents,)*) = marker;
                        #variant_instantiation
                    }
                }
            });
        }

        let enum_variants_tokens = &data_enum.variants;

        quote! {
            #(#attrs)*
            #vis enum #name #ty_generics {
                #enum_variants_tokens
            }

            #(#enum_from_impls)*
        }
    };

    TokenStream::from(expanded)
}
