use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(EventKind, attributes(event_name, event_info))]
pub fn derive_event_kind(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string().to_lowercase();

    let mut event_name = quote! { #name_str };
    let mut event_info = quote! { modding::events::game_event::GameEventInfo };

    for attr in &input.attrs {
        if attr.path().is_ident("event_name") {
            if let Ok(syn::Lit::Str(s)) = &attr.parse_args() {
                let s = s.value();
                event_name = quote! { #s };
            }
        } else if attr.path().is_ident("event_info") {
            if let Ok(syn::TypePath { path, .. }) = &attr.parse_args() {
                event_info = quote! { #path };
            }
        }
    }

    let expanded = quote! {
        impl EventKind for #name {
            type Info = #event_info;

            fn name() -> &'static str {
                #event_name
            }
        }

        modding::events::inventory::submit! {
            modding::events::RegisteredEventKind {
                name: #event_name,
                register_fn: |app: &mut bevy::prelude::App| {
                    app.add_observer(modding::events::on_game_event::<#name>);
                },
            }
        }
    };

    TokenStream::from(expanded)
}
