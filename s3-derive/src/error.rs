use darling::{FromDeriveInput, FromVariant};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(enum_any))]
pub struct ErrorOpts {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<VariantReceiver, ()>,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(error))]
pub struct VariantReceiver {
    ident: syn::Ident,

    status_code: u16,
    message: String,
}

impl ToTokens for ErrorOpts {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ErrorOpts {
            ref ident,
            ref generics, 
            ref data,
        } = *self;
        
        let variants = data.as_ref().take_enum().expect("Should never be struct");

        let generic_lifetimes: Vec<_> = generics.lifetimes().map(|l| {
            let syn::LifetimeParam {
                attrs,
                lifetime,
                ..
            } = l;

            quote! {
                #(#attrs)*
                #lifetime
            }
        }).collect();
        let generic_types: Vec<_> = generics.type_params().map(|t| {
            let syn::TypeParam {
                attrs,
                ident,
                colon_token,
                bounds,
                ..
            } = t;

            quote! {
                #(#attrs)*
                #ident
                #colon_token
                #bounds
            }
        }).collect();
        let generic_consts: Vec<_> = generics.const_params().map(|c| {
            let syn::ConstParam {
                attrs,
                const_token,
                ident,
                colon_token,
                ty,
                ..
            } = c;

            quote! {
                #(#attrs)*
                #const_token
                #ident
                #colon_token
                #ty
            }
        }).collect();
        let quoted_generics: Vec<_> = generic_lifetimes.into_iter().chain(generic_types).chain(generic_consts).collect();

        let generic_names: Vec<_> = generics.params.iter().map(|p| match p {
            syn::GenericParam::Type(syn::TypeParam { ident, .. }) => quote! { #ident },
            syn::GenericParam::Lifetime(syn::LifetimeParam { lifetime, .. }) => quote! { #lifetime },
            syn::GenericParam::Const(syn::ConstParam { ident, .. }) => quote! { #ident },
        }).collect();

        let where_clauses: Vec<_> = generics.where_clause.iter().flat_map(|c| c.predicates.iter().map(|p| quote! { #p })).collect();
        
        let message_arms: Vec<_> = variants.iter().map(|v| {
            let enum_name = &ident;
            let variant_name = &v.ident;
            let format = &v.message;
            
            quote! {
                #enum_name::#variant_name { .. } => {
                    write!(__formatter, #format)
                }
            }
        })
        .collect();
        let description_arms: Vec<_> = variants.iter().map(|v| {
            let enum_name = &ident;
            let variant_name = &v.ident;
            
            quote! {
                #enum_name::#variant_name { .. } => stringify!(#enum_name::#variant_name),
            }
        }).collect();
        
        let error_response_arms:Vec<_> = variants.iter().map(|v| {
            let enum_name = &ident;
            let variant_name = &v.ident;

            let error_code = format!("{}", variant_name);
            let message = format!("{}", v.message);
            
            quote! {
                #enum_name::#variant_name { ref resource, ref request_id } => ::quick_xml::se::to_string(&s3_api::error::XmlError {
                    code: #error_code.into(),
                    message: #message.into(),
                    resource: String::clone(&resource),
                    request_id: String::clone(&request_id),
                }).expect("error should be serializable into xml"),
            }
        }).collect();
        let status_code_arms:Vec<_> = variants.iter().map(|v| {
            let enum_name = &ident;
            let variant_name = &v.ident;
            let status_code = &v.status_code;
            
            quote! {
                #enum_name::#variant_name { .. } => ::actix_web::http::StatusCode::from_u16(#status_code).unwrap(),
            }
        }).collect();

        tokens.extend(quote! {
            #[allow(single_use_lifetimes)]
            impl<#(#quoted_generics),*> ::core::fmt::Display for #ident<#(#generic_names),*>
            where
                #(#where_clauses),*
            {
                fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    #[allow(unused_variables)]
                    match *self {
                        #(#message_arms),*
                    }
                }
            }

            #[allow(single_use_lifetimes)]
            impl<#(#quoted_generics),*> ::std::error::Error for #ident<#(#generic_names),*>
            where
                Self: ::core::fmt::Debug + ::core::fmt::Display,
                #(#where_clauses),*
            {
                fn description(&self) -> &str {
                    match *self {
                        #(#description_arms)*
                    }
                }

                fn cause(&self) -> ::core::option::Option<&dyn ::std::error::Error> {
                    ::core::option::Option::None
                }

                fn source(&self) -> ::core::option::Option<&(dyn ::std::error::Error + 'static)> {
                    ::core::option::Option::None
                }
            }

            impl<#(#quoted_generics),*> ::actix_web::error::ResponseError for #ident<#(#generic_names),*>
            where
                #(#where_clauses),*
            {
                fn error_response(&self) -> ::actix_web::HttpResponse<::actix_web::body::BoxBody> {
                    ::actix_web::HttpResponse::build(self.status_code())
                        .insert_header(::actix_web::http::header::ContentType::xml())
                        .body(match *self {
                             #(#error_response_arms)*
                        })
                }
                        
                fn status_code(&self) -> ::actix_web::http::StatusCode {
                    match *self {
                        #(#status_code_arms)*
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use darling::FromDeriveInput;
    use syn::parse_str;
    use quote::quote;

    use crate::error::ErrorOpts;

    #[test]
    fn test_empty_enum() {
        let input = r#"#[derive(Error)]
    pub enum TestError {
        #[error(message = "test", status_code = 401)]
        Test,
    }"#;

        let parsed = parse_str(input).unwrap();
        let receiver = ErrorOpts::from_derive_input(&parsed).unwrap();
        let tokens = quote!(#receiver);

    println!(
        r#"
INPUT:

{}

PARSED AS:

{:?}

EMITS:

{}
    "#,
        input, receiver, tokens
    );
    }
}
