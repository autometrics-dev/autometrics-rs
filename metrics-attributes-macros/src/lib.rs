use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Error, ItemFn, Path, PathSegment, Result, ReturnType, Type, TypePath,
};

#[proc_macro_attribute]
pub fn instrument(
    _args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    let output = match instrument_inner(item) {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    };

    output.into()
}

fn instrument_inner(item: ItemFn) -> Result<TokenStream> {
    let sig = item.sig;
    let block = item.block;
    let vis = item.vis;

    // If the function is async we need to add a .await after the block
    let maybe_await = if sig.asyncness.is_some() {
        quote! { .await }
    } else {
        TokenStream::new()
    };

    // This is a convoluted way to figure out if the return type resolves to a Result
    // or not. We cannot simply parse the code using syn to figure out if it's a Result
    // because syn doesn't do type resolution and thus would count any renamed version
    // of Result as a different type. Instead, we define two traits with intentionally
    // conflicting method names and use a trick based on the order in which Rust resolves
    // method names to return a different value based on whether the return value is
    // a Result or anything else.
    // This approach is based on dtolnay's answer to this question:
    // https://users.rust-lang.org/t/how-to-check-types-within-macro/33803/5
    // and this answer explains why it works:
    // https://users.rust-lang.org/t/how-to-check-types-within-macro/33803/8
    //
    // TODO should we move this to the main crate export so it isn't redefined every time?
    let trait_to_get_return_type = quote! {
        trait GetLabelsFromResult {
            fn __metrics_attributes_labels(&self) -> &'static [(&'static str, &'static str)];
        }
        impl<T, E> GetLabelsFromResult for ::std::result::Result<T, E> {
            fn __metrics_attributes_labels(&self) -> &'static [(&'static str, &'static str)] {
                match self {
                    Ok(_) => &[("result", "ok")],
                    Err(_) => &[("result", "err")],
                }
            }
        }
        trait GetLabels {
            fn __metrics_attributes_labels(&self) -> &'static [(&'static str, &'static str)] {
                &[]
            }
        }
        impl<T> GetLabels for &T { }
    };

    // TODO make sure we import metrics macros from the right place
    let counter_name = format!("{}_total", sig.ident);
    let histogram_name = format!("{}_duration_seconds", sig.ident);
    let track_metrics = quote! {
        #trait_to_get_return_type
        let labels = ret.__metrics_attributes_labels();
        ::metrics::histogram!(#histogram_name, __start_internal.elapsed().as_secs_f64(), labels);
        ::metrics::increment_counter!(#counter_name, labels);
    };

    // TODO generate doc comments that describe the related metrics

    Ok(quote! {
        #vis #sig {
            let __start_internal = ::std::time::Instant::now();
            let ret = #block #maybe_await;

            #track_metrics

            ret
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_fn() {
        let item = quote! {
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        };
        let item: ItemFn = syn::parse2(item).unwrap();
        let actual = instrument_inner(item).unwrap();
        let expected = quote! {
            pub fn add(a: i32, b: i32) -> i32 {
                let __start_internal = ::std::time::Instant::now();

                let ret = {
                    a + b
                };

                ::metrics::histogram!("add_duration_seconds", __start_internal.elapsed().as_secs_f64());
                ::metrics::increment_counter!("add_total");

                ret
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn async_fn() {
        let item = quote! {
            async fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        };
        let item: ItemFn = syn::parse2(item).unwrap();
        let actual = instrument_inner(item).unwrap();
        let expected = quote! {
            async fn add(a: i32, b: i32) -> i32 {
                let __start_internal = ::std::time::Instant::now();

                let ret = {
                    a + b
                }.await;

                ::metrics::histogram!("add_duration_seconds", __start_internal.elapsed().as_secs_f64());
                ::metrics::increment_counter!("add_total");

                ret
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn return_result() {
        let item = quote! {
            fn check_positive(num: i32) -> Result<(), ()> {
                if num >= 0 {
                    Ok(())
                } else {
                    Err(())
                }
            }
        };
        let item: ItemFn = syn::parse2(item).unwrap();
        let actual = instrument_inner(item).unwrap();
        let expected = quote! {
            fn check_positive(num: i32) -> Result<(), ()> {
                let __start_internal = ::std::time::Instant::now();

                let ret = {
                    if num >= 0 {
                        Ok(())
                    } else {
                        Err(())
                    }
                };

                let status = if ret.is_ok() {
                    "ok"
                } else {
                    "err"
                };
                ::metrics::histogram!("check_positive_duration_seconds", "result" => status, __start_internal.elapsed().as_secs_f64());
                ::metrics::increment_counter!("check_positive_total", "result" => status);

                ret
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
