//! The definition of the ResultLabels derive macro, that allows to specify
//! inside an enumeration whether variants should be considered as errors or
//! successes as far as the automatic metrics are concerned.
//!
//! For example, this would allow you to put all the client-side errors in a
//! HTTP webserver (4**) as successes, since it means the handler function
//! _successfully_ rejected a bad request, and that should not affect the SLO or
//! the success rate of the function in the metrics.
//!
//! ```rust,ignore
//! #[derive(ResultLabels)]
//! enum ServiceError {
//! // By default, the variant will be labeled as an error,
//! // so you do not need to decorate every variant
//! Database,
//! // It is possible to mention it as well of course.
//! // Only "error" and "ok" are accepted values
//! #[label(result = "error")]
//! Network,
//! #[label(result = "ok")]
//! Authentication,
//! #[label(result = "ok")]
//! Authorization,
//! }
//! ```

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, Data, DataEnum, DeriveInput, Error, Ident,
    Lit, LitStr, Result, Variant,
};

// These labels must match autometrics::ERROR_KEY and autometrics::OK_KEY,
// to avoid a dependency loop just for 2 constants we recreate these here.
const OK_KEY: &str = "ok";
const ERROR_KEY: &str = "error";
const RESULT_KEY: &str = "result";
const ATTR_LABEL: &str = "label";
const ACCEPTED_LABELS: [&str; 2] = [ERROR_KEY, OK_KEY];

/// Entry point of the ResultLabels macro
pub(crate) fn expand(input: DeriveInput) -> Result<TokenStream> {
    let Data::Enum(DataEnum {
        variants,
        ..}) = &input.data else
        {
                return Err(Error::new_spanned(
                    input,
                    "ResultLabels only works with 'Enum's.",
                ))
        };
    let enum_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let conditional_clauses_for_labels = conditional_label_clauses(variants, enum_name)?;

    // NOTE: we cannot reuse the GetResultLabel implementation in the GetLabels implementation,
    // because the GetResultLabel implementation is for a type T, while the GetLabels argument
    // is a reference &T, and the blanket impl of GetResultLabel for &T will be used instead of
    // the implementation we just wrote.
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::autometrics::__private::GetResultLabel for #enum_name #ty_generics #where_clause {
            fn __autometrics_get_result_label(&self) -> Option<&'static str> {
                #(#conditional_clauses_for_labels)*
            }
        }

        #[automatically_derived]
        impl #impl_generics ::autometrics::__private::GetLabels for #enum_name #ty_generics #where_clause {
            fn __autometrics_get_labels(&self) -> Option<::autometrics::__private::ResultAndReturnTypeLabels> {
                use ::autometrics::__private::GetStaticStr;

                let result_label = {
                    #(#conditional_clauses_for_labels)*
                };

                result_label.map(|label| (label, label.__autometrics_static_str()))
            }
        }
    })
}

/// Build the list of match clauses for the generated code.
fn conditional_label_clauses(
    variants: &Punctuated<Variant, Comma>,
    enum_name: &Ident,
) -> Result<Vec<TokenStream>> {
    // Dummy first clause to write all the useful payload with 'else if's
    std::iter::once(Ok(quote![if false {
        None
    }]))
    .chain(variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_matcher: TokenStream = match variant.fields {
            syn::Fields::Named(_) => quote! { #variant_name {..} },
            syn::Fields::Unnamed(_) => quote! { #variant_name (_) },
            syn::Fields::Unit => quote! { #variant_name },
        };
        if let Some(key) = extract_label_attribute(&variant.attrs)? {
            Ok(quote! [
                else if ::std::matches!(self, & #enum_name :: #variant_matcher) {
                   Some(#key)
                }
            ])
        } else {
            // Let the code flow through the last value
            Ok(quote! {})
        }
    }))
    // Fallback case: we return None
    .chain(std::iter::once(Ok(quote! [
        else {
            None
        }
    ])))
    .collect()
}

/// Extract the wanted label from the annotation in the variant, if present.
/// The function looks for `#[label(result = "ok")]` kind of labels.
///
/// ## Error cases
///
/// The function will error out with the smallest possible span when:
///
/// - The attribute on a variant is not a "list" type (so `#[label]` is not allowed),
/// - The key in the key value pair is not "result", as it's the only supported keyword
///   for now (so `#[label(non_existing_label = "ok")]` is not allowed),
/// - The value for the "result" label is not in the autometrics supported set (so
///   `#[label(result = "random label that will break queries")]` is not allowed)
fn extract_label_attribute(attrs: &[Attribute]) -> Result<Option<LitStr>> {
    attrs
            .iter()
            .find_map(|att| match att.parse_meta() {
                Ok(meta) => match &meta {
                    syn::Meta::List(list) => {
                        // Ignore attribute if it's not `label(...)`
                        if list.path.segments.len() != 1 || list.path.segments[0].ident != ATTR_LABEL {
                            return None;
                        }

                        // Only lists are allowed
                        let Some(syn::NestedMeta::Meta(syn::Meta::NameValue(pair))) = list.nested.first() else {
                            return Some(Err(Error::new_spanned(
                            meta,
                            format!("Only `{ATTR_LABEL}({RESULT_KEY} = \"RES\")` (RES can be {OK_KEY:?} or {ERROR_KEY:?}) is supported"),
                            )))
                        };

                        // Inside list, only 'result = ...' are allowed
                        if pair.path.segments.len() != 1 || pair.path.segments[0].ident != RESULT_KEY {
                            return Some(Err(Error::new_spanned(
                                pair.path.clone(),
                            format!("Only `{RESULT_KEY} = \"RES\"` (RES can be {OK_KEY:?} or {ERROR_KEY:?}) is supported"),
                            )));
                        }

                        // Inside 'result = val', 'val' must be a string literal
                        let Lit::Str(ref lit_str) = pair.lit else {
                            return Some(Err(Error::new_spanned(
                                &pair.lit,
                            format!("Only {OK_KEY:?} or {ERROR_KEY:?}, as string literals, are accepted as result values"),
                            )));
                        };

                        // Inside 'result = val', 'val' must be one of the allowed string literals
                        if !ACCEPTED_LABELS.contains(&lit_str.value().as_str()) {
                            return Some(Err(Error::new_spanned(
                                    lit_str,
                            format!("Only {OK_KEY:?} or {ERROR_KEY:?} are accepted as result values"),
                            )));
                        }

                        Some(Ok(lit_str.clone()))
                    },
                    syn::Meta::NameValue(nv) if nv.path.segments.len() == 1 && nv.path.segments[0].ident == ATTR_LABEL => {
                        Some(Err(Error::new_spanned(
                            nv,
                            format!("Only `{ATTR_LABEL}({RESULT_KEY} = \"RES\")` (RES can be {OK_KEY:?} or {ERROR_KEY:?}) is supported"),
                        )))
                    },
                    syn::Meta::Path(p) if p.segments.len() == 1 && p.segments[0].ident == ATTR_LABEL => {
                        Some(Err(Error::new_spanned(
                            p,
                            format!("Only `{ATTR_LABEL}({RESULT_KEY} = \"RES\")` (RES can be {OK_KEY:?} or {ERROR_KEY:?}) is supported"),
                        )))
                    },
                    _ => None,
                },
                Err(e) => Some(Err(Error::new_spanned(
                    att,
                    format!("could not parse the meta attribute: {e}"),
                ))),
            })
            .transpose()
}
