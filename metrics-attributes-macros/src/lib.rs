use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Error, ItemFn, Result};

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
    dbg!(&item);
    let sig = item.sig;
    let block = item.block;
    let vis = item.vis;

    // If the function is async we need to add a .await after the block
    let maybe_await = if sig.asyncness.is_some() {
        quote! { .await }
    } else {
        TokenStream::new()
    };
    let block = quote! {
        #block #maybe_await
    };

    // Define the metrics
    let counter_name = format!("{}_total", sig.ident);
    let histogram_name = format!("{}_duration_seconds", sig.ident);

    // TODO generate doc comments that describe the related metrics

    Ok(quote! {
        #vis #sig {
            let __start_internal = ::std::time::Instant::now();
            ::metrics::increment_counter!(#counter_name);

            let ret = #block;

            ::metrics::histogram!(#histogram_name, __start_internal.elapsed().as_secs_f64());

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
                ::metrics::increment_counter!("add_total");

                let ret = {
                    a + b
                };

                ::metrics::histogram!("add_duration_seconds", __start_internal.elapsed().as_secs_f64());

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
                ::metrics::increment_counter!("add_total");

                let ret = {
                    a + b
                }.await;

                ::metrics::histogram!("add_duration_seconds", __start_internal.elapsed().as_secs_f64());

                ret
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
