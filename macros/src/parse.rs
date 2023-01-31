use syn::parse::{Parse, ParseStream};
use syn::{ItemFn, ItemImpl, Result, Token};

/// Autometrics can be applied to individual functions or to
/// (all of the methods within) impl blocks.
pub(crate) enum Item {
    Function(ItemFn),
    Impl(ItemImpl),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![impl]) {
            input.parse().map(Item::Impl)
        } else {
            input.parse().map(Item::Function)
        }
    }
}

#[derive(Default)]
pub(crate) struct Args {
    pub track_concurrency: bool,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = Args::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::track_concurrency) {
                let _ = input.parse::<kw::track_concurrency>()?;
                args.track_concurrency = true;
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                return Err(lookahead.error());
            }
        }
        Ok(args)
    }
}

mod kw {
    syn::custom_keyword!(track_concurrency);
}
