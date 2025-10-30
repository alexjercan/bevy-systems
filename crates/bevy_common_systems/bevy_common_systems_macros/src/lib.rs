use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(EventKind, attributes(event_name))]
pub fn derive_event_kind(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string().to_lowercase();

    let mut event_name = quote! { #name_str };

    for attr in &input.attrs {
        if attr.path().is_ident("event_name") {
            if let Ok(syn::Lit::Str(s)) = &attr.parse_args() {
                let s = s.value();
                event_name = quote! { #s };
            }
        }
    }

    let expanded = quote! {
        impl EventKind for #name {
            type Info = bevy::platform::collections::HashMap<String, String>;

            fn name() -> &'static str {
                #event_name
            }
        }

        modding::events::registry::inventory::submit! {
            modding::events::registry::RegisteredEventKind {
                name: #event_name,
                register_fn: |app: &mut bevy::prelude::App| {
                    app.add_observer(modding::events::game_event::on_game_event::<#name>);
                },
            }
        }
    };

    TokenStream::from(expanded)
}
