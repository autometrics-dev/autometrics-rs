//! The definition of the ResultLabels derive macro, see
//! autometrics::ResultLabels for more information.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, Data, DataEnum, DeriveInput, Error, Expr,
    ExprLit, Ident, Lit, LitStr, Result, Variant,
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
    let variants = match &input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => {
            return Err(Error::new_spanned(
                input,
                "ResultLabels only works with 'Enum's.",
            ))
        }
    };
    let enum_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let conditional_clauses_for_labels = conditional_label_clauses(variants, enum_name)?;

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::autometrics::__private::GetLabels for #enum_name #ty_generics #where_clause {
            fn __autometrics_get_labels(&self) -> Option<&'static str> {
                #conditional_clauses_for_labels
            }
        }
    })
}

/// Build the list of match clauses for the generated code.
fn conditional_label_clauses(
    variants: &Punctuated<Variant, Comma>,
    enum_name: &Ident,
) -> Result<TokenStream> {
    let clauses: Vec<TokenStream> = variants
        .iter()
        .map(|variant| {
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
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! [
        if false {
            None
        }
        #(#clauses)*
        else {
            None
        }
    ])
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
            .find_map(|att| match &att.meta {
                    syn::Meta::List(list) => {
                        // Ignore attribute if it's not `label(...)`
                        if list.path.segments.len() != 1 || list.path.segments[0].ident != ATTR_LABEL {
                            return None;
                        }

                        // Only lists are allowed
                        let pair = match att.meta.require_list().and_then(|list| list.parse_args::<syn::MetaNameValue>()) {
                            Ok(pair) => pair,
                            Err(..) => return Some(
                                Err(
                                    Error::new_spanned(
                                        &att.meta,
                                        format!("Only `{ATTR_LABEL}({RESULT_KEY} = \"RES\")` (RES can be {OK_KEY:?} or {ERROR_KEY:?}) is supported"),
                                    ),
                                ),
                            ),
                        };

                        // Inside list, only 'result = ...' are allowed
                        if pair.path.segments.len() != 1 || pair.path.segments[0].ident != RESULT_KEY {
                            return Some(Err(Error::new_spanned(
                                pair.path.clone(),
                            format!("Only `{RESULT_KEY} = \"RES\"` (RES can be {OK_KEY:?} or {ERROR_KEY:?}) is supported"),
                            )));
                        }

                        // Inside 'result = val', 'val' must be a string literal
                        let lit_str = match pair.value {
                            Expr::Lit(ExprLit { lit: Lit::Str(ref lit_str), .. }) => lit_str,
                            _ => {
                            return Some(Err(Error::new_spanned(
                                &pair.value,
                            format!("Only {OK_KEY:?} or {ERROR_KEY:?}, as string literals, are accepted as result values"),
                            )));
                        }
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
                })
            .transpose()
}
