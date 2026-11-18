#![expect(
    clippy::unwrap_used,
    reason = "build script must fail immediately on invalid extracted predicate data"
)]

//! Build-time codegen for the `minecraft:predicate` registry (3/3).

use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::context_providers::generate_condition;
use crate::datapack::{quote_id, read_entries};

pub(crate) fn build() -> TokenStream {
    let entries = read_entries("predicate");
    assert_eq!(
        entries.len(),
        3,
        "expected 3 vanilla predicate files, got {}",
        entries.len()
    );

    let mut stream = TokenStream::new();
    stream.extend(quote! {
        use std::sync::LazyLock;

        use steel_utils::Identifier;

        use crate::context_provider::{
            ContextCondition, EnchantmentRequirement, LootPredicate, LootPredicateRegistry,
        };
    });

    let mut register_stream = TokenStream::new();
    for (name, json) in &entries {
        let ident = Ident::new(
            &name.replace('/', "_").to_shouty_snake_case(),
            Span::call_site(),
        );
        let key = quote_id(&format!("minecraft:{name}"));
        let condition = generate_condition(json);
        stream.extend(quote! {
            pub static #ident: LazyLock<LootPredicate> =
                LazyLock::new(|| LootPredicate { key: #key, condition: #condition });
        });
        register_stream.extend(quote! {
            registry.register(&#ident);
        });
    }

    stream.extend(quote! {
        pub fn register_predicates(registry: &mut LootPredicateRegistry) {
            #register_stream
        }
    });
    stream
}
