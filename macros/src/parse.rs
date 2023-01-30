use syn::parse::{Parse, ParseStream};
use syn::{Error, ItemFn, ItemImpl, LitStr, Result, Token};

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
    pub name: Option<String>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = Args::default();
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
                // TODO validate label name
                args.name = Some(name.value());
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                return Err(lookahead.error());
            }
        }
        Ok(args)
    }
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

mod kw {
    syn::custom_keyword!(name);
}
