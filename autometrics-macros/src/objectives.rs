use crate::parse::CreateObjectiveArgs;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn create_objective(args: CreateObjectiveArgs) -> TokenStream {
    let name = args.name;
    let success_rate = if let Some(success_rate) = args.success_rate {
        let success_rate = success_rate.normalize().to_string();
        quote! { Some(#success_rate) }
    } else {
        quote! { None }
    };
    let latency = if let Some(latency) = args.latency {
        let target_seconds = latency.target_seconds.normalize().to_string();
        let percentile = latency.percentile.normalize().to_string();
        quote! { Some((#target_seconds, #percentile )) }
    } else {
        quote! { None }
    };

    quote! {
      autometrics::__private::Objective {
        name: #name,
        success_rate: #success_rate,
        latency: #latency,
      }
    }
}
