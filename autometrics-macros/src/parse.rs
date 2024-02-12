use syn::parse::{Parse, ParseStream};
use syn::{Expr, ItemFn, ItemImpl, LitStr, Result, Token};

mod kw {
    syn::custom_keyword!(track_concurrency);
    syn::custom_keyword!(objective);
    syn::custom_keyword!(success_rate);
    syn::custom_keyword!(latency);
    syn::custom_keyword!(ok_if);
    syn::custom_keyword!(error_if);
    syn::custom_keyword!(struct_name);
}

/// Autometrics can be applied to individual functions or to
/// (all of the methods within) impl blocks.
pub(crate) enum Item {
    Function(ItemFn),
    Impl(ItemImpl),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        input
            .parse()
            .map(Item::Function)
            .or_else(|_| input.parse().map(Item::Impl))
    }
}

#[derive(Default)]
pub(crate) struct AutometricsArgs {
    pub track_concurrency: bool,
    pub ok_if: Option<Expr>,
    pub error_if: Option<Expr>,
    pub objective: Option<Expr>,

    // Fix for https://github.com/autometrics-dev/autometrics-rs/issues/139.
    pub struct_name: Option<String>,
}

impl Parse for AutometricsArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = AutometricsArgs::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::track_concurrency) {
                let _ = input.parse::<kw::track_concurrency>()?;
                args.track_concurrency = true;
            } else if lookahead.peek(kw::ok_if) {
                if args.ok_if.is_some() {
                    return Err(input.error("expected only a single `ok_if` argument"));
                }
                if args.error_if.is_some() {
                    return Err(input.error("cannot use both `ok_if` and `error_if`"));
                }
                let ok_if = input.parse::<ExprArg<kw::ok_if>>()?;
                args.ok_if = Some(ok_if.value);
            } else if lookahead.peek(kw::error_if) {
                if args.error_if.is_some() {
                    return Err(input.error("expected only a single `error_if` argument"));
                }
                if args.ok_if.is_some() {
                    return Err(input.error("cannot use both `ok_if` and `error_if`"));
                }
                let error_if = input.parse::<ExprArg<kw::error_if>>()?;
                args.error_if = Some(error_if.value);
            } else if lookahead.peek(kw::objective) {
                let _ = input.parse::<kw::objective>()?;
                let _ = input.parse::<Token![=]>()?;
                if args.objective.is_some() {
                    return Err(input.error("expected only a single `objective` argument"));
                }
                args.objective = Some(input.parse()?);
            } else if lookahead.peek(kw::struct_name) {
                let _ = input.parse::<kw::struct_name>()?;
                let _ = input.parse::<Token![=]>()?;
                let struct_name = input.parse::<LitStr>()?.value();
                args.struct_name = Some(struct_name);
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                return Err(lookahead.error());
            }
        }
        Ok(args)
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
