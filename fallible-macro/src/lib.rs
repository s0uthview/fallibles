use syn::{parse_macro_input, ItemFn, ReturnType, Type, GenericArgument, PathArguments, DeriveInput, Data, Fields, Lit, Meta};
use proc_macro::TokenStream;
use quote::quote;

fn extract_result_error_type(return_type: &ReturnType) -> Option<&Type> {
    if let ReturnType::Type(_, ty) = return_type {
        if let Type::Path(type_path) = &**ty {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Result" {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if args.args.len() == 2 {
                            if let Some(GenericArgument::Type(err_type)) = args.args.iter().nth(1) {
                                return Some(err_type);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[proc_macro_attribute]
pub fn fallible(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let sig = &input.sig;
    let block = &input.block;
    let vis = &input.vis;

    let fn_name = sig.ident.to_string();
    let id_hash = fxhash::hash32(fn_name.as_bytes());

    let error_type = extract_result_error_type(&sig.output);

    let expanded = if let Some(err_ty) = error_type {
        quote! {
            #vis #sig {
                #[cfg(feature = "fallible-sim")]
                if ::fallible::fallible_core::should_simulate_failure(
                    ::fallible::fallible_core::FailurePoint {
                        id: ::fallible::fallible_core::FailurePointId(#id_hash),
                        function: #fn_name,
                        file: file!(),
                        line: line!(),
                        column: column!(),
                    }
                ) {
                    return Err(<#err_ty as ::fallible::fallible_core::FallibleError>::simulated_failure());
                }

                #block
            }
        }
    } else {
        quote! {
            #vis #sig #block
        }
    };

    expanded.into()
}

#[proc_macro_derive(FallibleError, attributes(fallible))]
pub fn derive_fallible_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let custom_message = input.attrs.iter()
        .find_map(|attr| {
            if attr.path().is_ident("fallible") {
                if let Meta::NameValue(nv) = &attr.meta {
                    if nv.path.is_ident("message") {
                        if let syn::Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                return Some(lit_str.value());
                            }
                        }
                    }
                }
            }
            None
        });

    let error_expr = match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(_) => {
                    if let Some(msg) = custom_message {
                        quote! { Self { message: #msg.to_string() } }
                    } else {
                        quote! { Self { message: "simulated failure".to_string() } }
                    }
                },
                Fields::Unnamed(fields) => {
                    if fields.unnamed.len() == 1 {
                        if let Some(msg) = custom_message {
                            quote! { Self(#msg.to_string()) }
                        } else {
                            quote! { Self("simulated failure".to_string()) }
                        }
                    } else {
                        quote! { Self(Default::default()) }
                    }
                },
                Fields::Unit => {
                    quote! { Self }
                }
            }
        },
        Data::Enum(data_enum) => {
            let fallible_variant = data_enum.variants.iter().find(|v| {
                v.attrs.iter().any(|attr| {
                    attr.path().is_ident("fallible") &&
                    matches!(&attr.meta, Meta::Path(_))
                })
            });

            let variant = fallible_variant.or_else(|| data_enum.variants.first());

            if let Some(v) = variant {
                let variant_name = &v.ident;
                match &v.fields {
                    Fields::Named(_) => {
                        if let Some(msg) = custom_message {
                            quote! { Self::#variant_name { message: #msg.to_string() } }
                        } else {
                            quote! { Self::#variant_name { message: "simulated failure".to_string() } }
                        }
                    },
                    Fields::Unnamed(fields) => {
                        if fields.unnamed.len() == 1 {
                            if let Some(msg) = custom_message {
                                quote! { Self::#variant_name(#msg.to_string()) }
                            } else {
                                quote! { Self::#variant_name("simulated failure".to_string()) }
                            }
                        } else {
                            quote! { Self::#variant_name(Default::default()) }
                        }
                    },
                    Fields::Unit => {
                        quote! { Self::#variant_name }
                    }
                }
            } else {
                quote! { panic!("No variants in enum") }
            }
        },
        Data::Union(_) => {
            quote! { panic!("Unions are not supported for FallibleError") }
        }
    };

    let expanded = quote! {
        impl #impl_generics ::fallible::fallible_core::FallibleError for #name #ty_generics #where_clause {
            fn simulated_failure() -> Self {
                #error_expr
            }
        }
    };

    expanded.into()
}