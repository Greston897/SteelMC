//! Build-time codegen for `minecraft:test_environment` (1/1) and
//! `minecraft:test_instance` (1/1).

use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::datapack::{quote_id, read_entries};

fn generate_test_environment_definition(name: &str, json: &serde_json::Value) -> TokenStream {
    let kind = json
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("test_environment {name} is missing type"));
    match kind {
        "minecraft:all_of" => {
            let definitions = json
                .get("definitions")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| panic!("test_environment {name} all_of is missing definitions"))
                .iter()
                .map(|child| generate_test_environment_definition(name, child));
            quote! { TestEnvironmentDefinition::AllOf { definitions: vec![#(#definitions),*] } }
        }
        "minecraft:clock_time" => {
            let clock = quote_id(
                json.get("clock")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| {
                        panic!("test_environment {name} clock_time is missing clock")
                    }),
            );
            let time = json
                .get("time")
                .and_then(serde_json::Value::as_i64)
                .and_then(|v| i32::try_from(v).ok())
                .unwrap_or_else(|| panic!("test_environment {name} clock_time is missing time"));
            quote! { TestEnvironmentDefinition::ClockTime { clock: #clock, time: #time } }
        }
        "minecraft:difficulty" => {
            let difficulty = json
                .get("difficulty")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| {
                    panic!("test_environment {name} difficulty is missing difficulty")
                });
            let variant = match difficulty {
                "peaceful" => quote! { Difficulty::Peaceful },
                "easy" => quote! { Difficulty::Easy },
                "normal" => quote! { Difficulty::Normal },
                "hard" => quote! { Difficulty::Hard },
                other => panic!("test_environment {name} has unknown difficulty: {other}"),
            };
            quote! { TestEnvironmentDefinition::Difficulty { difficulty: #variant } }
        }
        "minecraft:function" => {
            let setup = if let Some(id) = json.get("setup").and_then(serde_json::Value::as_str) {
                let quoted = quote_id(id);
                quote! { Some(#quoted) }
            } else {
                quote! { None }
            };
            let teardown =
                if let Some(id) = json.get("teardown").and_then(serde_json::Value::as_str) {
                    let quoted = quote_id(id);
                    quote! { Some(#quoted) }
                } else {
                    quote! { None }
                };
            quote! { TestEnvironmentDefinition::Function { setup: #setup, teardown: #teardown } }
        }
        "minecraft:game_rules" => {
            let rules = json
                .get("rules")
                .and_then(serde_json::Value::as_object)
                .unwrap_or_else(|| panic!("test_environment {name} game_rules is missing rules"))
                .iter()
                .map(|(rule, value)| {
                    let rule = quote_id(rule);
                    if let Some(flag) = value.as_bool() {
                        quote! { (#rule, TestGameRuleValue::Bool(#flag)) }
                    } else if let Some(number) = value.as_i64().and_then(|v| i32::try_from(v).ok())
                    {
                        quote! { (#rule, TestGameRuleValue::Int(#number)) }
                    } else {
                        panic!("test_environment {name} has non-bool/int game rule value: {value}")
                    }
                });
            quote! { TestEnvironmentDefinition::GameRules { rules: vec![#(#rules),*] } }
        }
        "minecraft:timeline_attributes" => {
            let timelines = json.get("timelines").unwrap_or_else(|| {
                panic!("test_environment {name} timeline_attributes is missing timelines")
            });
            let ids: Vec<TokenStream> = if let Some(single) = timelines.as_str() {
                vec![quote_id(single)]
            } else if let Some(list) = timelines.as_array() {
                list.iter()
                    .map(|entry| {
                        quote_id(entry.as_str().unwrap_or_else(|| {
                            panic!("test_environment {name} has non-string timeline: {entry}")
                        }))
                    })
                    .collect()
            } else {
                panic!("test_environment {name} timelines must be a string or list")
            };
            quote! { TestEnvironmentDefinition::TimelineAttributes { timelines: vec![#(#ids),*] } }
        }
        "minecraft:weather" => {
            let weather = json
                .get("weather")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("test_environment {name} weather is missing weather"));
            let variant = match weather {
                "clear" => quote! { TestWeather::Clear },
                "rain" => quote! { TestWeather::Rain },
                "thunder" => quote! { TestWeather::Thunder },
                other => panic!("test_environment {name} has unknown weather: {other}"),
            };
            quote! { TestEnvironmentDefinition::Weather { weather: #variant } }
        }
        other => panic!("test_environment {name} has unknown type: {other}"),
    }
}

pub(crate) fn build_environments() -> TokenStream {
    let entries = read_entries("test_environment");
    assert_eq!(
        entries.len(),
        1,
        "expected 1 vanilla test_environment file, got {}",
        entries.len()
    );

    let mut stream = TokenStream::new();
    stream.extend(quote! {
        use std::sync::LazyLock;

        use steel_utils::Identifier;

        // NOTE: only `AllOf` is used by the vanilla test_environment, so
        // `Difficulty`, `TestGameRuleValue` and `TestWeather` stay unimported.
        use crate::gametest::{
            TestEnvironment, TestEnvironmentDefinition, TestEnvironmentRegistry,
        };
    });

    let mut register_stream = TokenStream::new();
    for (name, json) in &entries {
        let ident = Ident::new(&name.to_shouty_snake_case(), Span::call_site());
        let key = quote_id(&format!("minecraft:{name}"));
        let definition = generate_test_environment_definition(name, json);
        stream.extend(quote! {
            pub static #ident: LazyLock<TestEnvironment> = LazyLock::new(|| TestEnvironment {
                key: #key,
                definition: #definition,
            });
        });
        register_stream.extend(quote! {
            registry.register(&#ident);
        });
    }

    stream.extend(quote! {
        pub fn register_test_environments(registry: &mut TestEnvironmentRegistry) {
            #register_stream
        }
    });
    stream
}

fn generate_test_instance_value(name: &str, json: &serde_json::Value) -> TokenStream {
    let kind = json
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("test_instance {name} is missing type"));
    let kind = match kind {
        "minecraft:block_based" => quote! { TestInstanceKind::BlockBased },
        "minecraft:function" => {
            let function = quote_id(
                json.get("function")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| panic!("test_instance {name} function is missing function")),
            );
            quote! { TestInstanceKind::Function { function: #function } }
        }
        other => panic!("test_instance {name} has unknown type: {other}"),
    };
    let environment = quote_id(
        json.get("environment")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("test_instance {name} is missing environment")),
    );
    let dimension = quote_id(
        json.get("dimension")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("minecraft:overworld"),
    );
    let structure = quote_id(
        json.get("structure")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("test_instance {name} is missing structure")),
    );
    let max_ticks = json
        .get("max_ticks")
        .and_then(serde_json::Value::as_i64)
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or_else(|| panic!("test_instance {name} is missing max_ticks"));
    let setup_ticks = json
        .get("setup_ticks")
        .and_then(serde_json::Value::as_i64)
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or(0);
    let required = json
        .get("required")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let rotation = match json
        .get("rotation")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("none")
    {
        "none" => quote! { TestRotation::None },
        "clockwise_90" => quote! { TestRotation::Clockwise90 },
        "180" => quote! { TestRotation::Clockwise180 },
        "counterclockwise_90" => quote! { TestRotation::Counterclockwise90 },
        other => panic!("test_instance {name} has unknown rotation: {other}"),
    };
    let manual_only = json
        .get("manual_only")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let max_attempts = json
        .get("max_attempts")
        .and_then(serde_json::Value::as_i64)
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or(1);
    let required_successes = json
        .get("required_successes")
        .and_then(serde_json::Value::as_i64)
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or(1);
    let sky_access = json
        .get("sky_access")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let padding = json
        .get("padding")
        .and_then(serde_json::Value::as_i64)
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or(0);

    quote! {
        TestInstanceValue {
            kind: #kind,
            environment: #environment,
            dimension: #dimension,
            structure: #structure,
            max_ticks: #max_ticks,
            setup_ticks: #setup_ticks,
            required: #required,
            rotation: #rotation,
            manual_only: #manual_only,
            max_attempts: #max_attempts,
            required_successes: #required_successes,
            sky_access: #sky_access,
            padding: #padding,
        }
    }
}

pub(crate) fn build_instances() -> TokenStream {
    let entries = read_entries("test_instance");
    assert_eq!(
        entries.len(),
        1,
        "expected 1 vanilla test_instance file, got {}",
        entries.len()
    );

    let mut stream = TokenStream::new();
    stream.extend(quote! {
        use std::sync::LazyLock;

        use steel_utils::Identifier;

        use crate::gametest::{
            TestInstance, TestInstanceKind, TestInstanceRegistry, TestInstanceValue, TestRotation,
        };
    });

    let mut register_stream = TokenStream::new();
    for (name, json) in &entries {
        let ident = Ident::new(&name.to_shouty_snake_case(), Span::call_site());
        let key = quote_id(&format!("minecraft:{name}"));
        let value = generate_test_instance_value(name, json);
        stream.extend(quote! {
            pub static #ident: LazyLock<TestInstance> = LazyLock::new(|| TestInstance {
                key: #key,
                value: #value,
            });
        });
        register_stream.extend(quote! {
            registry.register(&#ident);
        });
    }

    stream.extend(quote! {
        pub fn register_test_instances(registry: &mut TestInstanceRegistry) {
            #register_stream
        }
    });
    stream
}
