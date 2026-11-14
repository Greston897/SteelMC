//! Generates the fire spread tables from vanilla `FireBlock.bootStrap`.
//!
//! Vanilla fills two `Object2IntMap`s at class-init: `igniteOdds` (how easily a
//! block catches fire) and `burnOdds` (how easily a burning block is consumed).
//! Those numbers are hard-coded in `FireBlock.bootStrap`, not in a data pack, so
//! they cannot come from the extracted registries; `build/flammable.json` holds
//! the same pairs, taken from the 26.3 decompile.

use std::{collections::BTreeMap, fs};

use quote::quote;

use crate::to_block_ident;

pub fn build() -> String {
    println!("cargo:rerun-if-changed=build/flammable.json");

    let flammable_json =
        fs::read_to_string("build/flammable.json").expect("Failed to read flammable.json");
    let flammable: BTreeMap<String, [u32; 2]> =
        serde_json::from_str(&flammable_json).expect("Failed to parse flammable.json");

    let ignite_arms: Vec<proc_macro2::TokenStream> = flammable
        .iter()
        .filter(|(_, odds)| odds[0] > 0)
        .map(|(name, odds)| {
            let ident = to_block_ident(name);
            let ignite = odds[0];
            quote! { b if b == &vanilla_blocks::#ident => #ignite , }
        })
        .collect();

    let burn_arms: Vec<proc_macro2::TokenStream> = flammable
        .iter()
        .filter(|(_, odds)| odds[1] > 0)
        .map(|(name, odds)| {
            let ident = to_block_ident(name);
            let burn = odds[1];
            quote! { b if b == &vanilla_blocks::#ident => #burn , }
        })
        .collect();

    let output = quote! {
        //! Generated fire spread tables, mirroring vanilla `FireBlock.bootStrap`.
        //!
        //! Do not edit by hand: `steel-core/build/flammable.rs` writes this file
        //! from `build/flammable.json`.

        use steel_registry::blocks::BlockRef;
        use steel_registry::vanilla_blocks;

        /// Vanilla `FireBlock.getIgniteOdds(BlockState)`: how readily the block
        /// catches fire. Zero means it does not burn at all.
        #[must_use]
        #[inline]
        pub fn ignite_odds(block: BlockRef) -> u32 {
            match block {
                #(#ignite_arms)*
                _ => 0
            }
        }

        /// Vanilla `FireBlock.getBurnOdds(BlockState)`: how readily a burning
        /// block is consumed. Zero means it never burns away.
        #[must_use]
        #[inline]
        pub fn burn_odds(block: BlockRef) -> u32 {
            match block {
                #(#burn_arms)*
                _ => 0
            }
        }
    };

    output.to_string()
}
