use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use steel_utils::Identifier;

#[derive(Deserialize)]
#[expect(dead_code)]
pub(crate) struct CriterionJson {
    trigger: Identifier,
    #[serde(default)]
    conditions: Value,
}

impl ToTokens for CriterionJson {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // The trigger id and the raw instance are kept so the server can match a
        // fired trigger against every advancement and evaluate the predicate the
        // instance carries (`steel-core`'s `advancement::predicate`). The
        // instance stays a JSON string because the predicate shape varies by
        // trigger and is easier to interpret at runtime than to type out here.
        let trigger = super::super::features::common::generate_identifier(&self.trigger);
        let instance = self.conditions.to_string();
        tokens.extend(quote! {
            Box::new(SimpleCriterion::new(
                #trigger,
                #instance.to_owned(),
            )) as Box<dyn AnyCriterion>
        });
    }
}

pub(crate) fn parse_criteria(list: &BTreeMap<String, CriterionJson>) -> TokenStream {
    let criteria = list.iter().map(|(k, v)| {
        quote! { (#k.to_owned(), #v) }
    });
    quote! {
        BTreeMap::from([#(#criteria),*])
    }
}
