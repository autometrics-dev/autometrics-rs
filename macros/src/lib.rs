use once_cell::sync::Lazy;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io::Write, path::PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Error, Expr, ItemFn, LitStr, Result, Token};

// TODO it would probably be better if this ended up in the directory of the main crate that's
// being built rater than in the out directory of the metrics-attributes-macro crate
static METRICS_FILE: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = PathBuf::new();
    // This is set in build.rs
    path.push(env!("OUT_DIR"));

    let compile_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("error getting system time")
        .as_micros();
    path.push(format!("metrics-{}.yaml", compile_time));

    println!("Writing list of metrics to: {}", path.display());

    path
});

#[derive(Default)]
struct InstrumentArgs {
    name: Option<String>,
}

impl Parse for InstrumentArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = InstrumentArgs::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::name) {
                if args.name.is_some() {
                    return Err(Error::new(
                        input.span(),
                        "expected only a single `name` argument",
                    ))?;
                }
                let name = input.parse::<StrArg<kw::name>>()?.value;
                args.name = Some(name.value());
            } else {
                return Err(lookahead.error());
            }
        }
        Ok(args)
    }
}

#[proc_macro_attribute]
pub fn instrument(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as InstrumentArgs);
    let item = parse_macro_input!(item as ItemFn);

    let output = match instrument_inner(args, item) {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    };

    output.into()
}

fn instrument_inner(args: InstrumentArgs, item: ItemFn) -> Result<TokenStream> {
    let span = item.span();
    let sig = item.sig;
    let block = item.block;
    let vis = item.vis;

    // If the function is async we need to add a .await after the block
    let maybe_await = if sig.asyncness.is_some() {
        quote! { .await }
    } else {
        TokenStream::new()
    };

    // TODO make sure we import metrics macros from the right place
    // TODO maybe it's okay if metrics is a peer dependency
    // TODO include the function name as a label
    let function_name = sig.ident.to_string();
    let base_name = args.name.unwrap_or_else(|| function_name.clone());
    let counter_name = format!("{}_total", base_name);
    let histogram_name = format!("{}_duration_seconds", base_name);
    write_metrics_to_file(&histogram_name, &counter_name, span)?;

    let track_metrics = quote! {
        use metrics_attributes::__private::{GetLabels, GetLabelsFromResult};
        let duration = __metrics_attributes_start.elapsed().as_secs_f64();
        if let Some(label) = ret.__metrics_attributes_get_result_label() {
            metrics::histogram!(#histogram_name, duration, "function" => #function_name, "result" => label);
            metrics::increment_counter!(#counter_name, "function" => #function_name, "result" => label);
        } else {
            metrics::histogram!(#histogram_name, duration, "function" => #function_name);
            metrics::increment_counter!(#counter_name, "function" => #function_name);
        }
    };

    // TODO generate doc comments that describe the related metrics

    Ok(quote! {
        #vis #sig {
            let __metrics_attributes_start = ::std::time::Instant::now();

            let ret = #block #maybe_await;

            #track_metrics

            ret
        }
    })
}

// Copied from tracing-attributes
struct StrArg<T> {
    value: LitStr,
    _p: std::marker::PhantomData<T>,
}

impl<T: Parse> Parse for StrArg<T> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _ = input.parse::<T>()?;
        let _ = input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self {
            value,
            _p: std::marker::PhantomData,
        })
    }
}

struct ExprArg<T> {
    value: Expr,
    _p: std::marker::PhantomData<T>,
}

impl<T: Parse> Parse for ExprArg<T> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _ = input.parse::<T>()?;
        let _ = input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self {
            value,
            _p: std::marker::PhantomData,
        })
    }
}

mod kw {
    syn::custom_keyword!(name);
}

// TODO can we figure out the labels and write those too?
// that would need to happen when the dependent crate is being built
// because we're only getting the labels after the macro runs when
// the crate is being compiled
//
// Alternative approaches:
// - call #[instrument(ret)] if you want the labels included so that we know the labels at macro expansion time
// - inject the printing to a file code but only have it run in #[cfg(test)] or some other mode
// - have a cargo command that goes through the code looking for metrics
fn write_metrics_to_file(histogram_name: &str, counter_name: &str, span: Span) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&*METRICS_FILE)
        .map_err(|err| {
            let message = format!(
                "error opening metrics file {} {:?}",
                METRICS_FILE.display(),
                err
            );
            Error::new(span, message)
        })?;
    writeln!(
        &mut file,
        "- name: {}
  type: histogram
- name: {}
  type: counter
",
        histogram_name, counter_name
    )
    .map_err(|err| {
        Error::new(
            span,
            format!(
                "error writing to metrics file {} {:?}",
                METRICS_FILE.display(),
                err
            ),
        )
    })?;

    Ok(())
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
        let actual = instrument_inner(Default::default(), item).unwrap();
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
        let actual = instrument_inner(Default::default(), item).unwrap();
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
        let actual = instrument_inner(Default::default(), item).unwrap();
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
