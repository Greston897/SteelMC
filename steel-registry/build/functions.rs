//! Build-time collection of the vanilla datapack's `.mcfunction` files.
//!
//! Vanilla ships the tags, recipes and functions of the built-in datapack as
//! data files. Steel compiles them into `static` declarations so `/function` and
//! `schedule function` resolve without a runtime file lookup, the same way the
//! recipe and advancement generators work.
//!
//! Function bodies keep vanilla's parsing rules: each line is one command, and
//! blank lines and `#` comments are dropped. A trailing `\r` from a CRLF file is
//! trimmed so Windows checkouts behave the same.

use std::fs;
use std::path::Path;

use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use steel_utils::Identifier;

/// One function read from disk.
struct ParsedFunction {
    /// Identifier built from the path below `function/`, without the extension.
    key: Identifier,
    /// Command lines with comments and blank lines removed.
    commands: Vec<String>,
}

pub(crate) fn build() -> TokenStream {
    let function_dir =
        Path::new("../steel-utils/build_assets/builtin_datapacks/minecraft/function");
    println!("cargo:rerun-if-changed={}", function_dir.display());

    let mut functions = Vec::new();
    collect(function_dir, function_dir, &mut functions);
    functions.sort_by_key(|function| function.key.to_string());

    let declarations: Vec<TokenStream> = functions.iter().map(declare).collect();
    let registrations: Vec<TokenStream> = functions
        .iter()
        .map(|function| {
            let ident = ident(&function.key);
            quote! { registry.register(&#ident); }
        })
        .collect();

    quote! {
        use std::sync::LazyLock;

        use crate::function::{Function, FunctionRegistry};
        use steel_utils::Identifier;

        #(#declarations)*

        /// Every function in the built-in vanilla datapack.
        pub static FUNCTIONS: LazyLock<FunctionRegistry> = LazyLock::new(|| {
            let mut registry = FunctionRegistry::new();
            #(#registrations)*
            registry.freeze();
            registry
        });
    }
}

/// Walks `dir`, recursing into subdirectories and collecting every
/// `.mcfunction` under it.
fn collect(root: &Path, dir: &Path, out: &mut Vec<ParsedFunction>) {
    let Ok(entries) = fs::read_dir(dir) else {
        // A vanilla datapack without any function directory is valid: it just
        // means no function exists. Reported once rather than panicking.
        println!(
            "cargo:warning=Function directory {} is missing; no functions will be loaded",
            dir.display()
        );
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("mcfunction")
            && let Some(function) = parse(root, &path)
        {
            out.push(function);
        }
    }
}

/// Reads one function file and turns its path into an identifier.
fn parse(root: &Path, path: &Path) -> Option<ParsedFunction> {
    let key = identifier_for(root, path)?;
    let source = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("Failed to read {}: {error}", path.display());
    });

    let commands = source
        .lines()
        .map(|line| line.trim_end_matches('\r').trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect();

    Some(ParsedFunction { key, commands })
}

/// Builds `minecraft:<relative path without extension>` for a function file.
fn identifier_for(root: &Path, path: &Path) -> Option<Identifier> {
    let relative = path.strip_prefix(root).ok()?;
    let mut segments = Vec::new();
    for component in relative.components() {
        let text = component.as_os_str().to_str()?;
        let text = text.strip_suffix(".mcfunction").unwrap_or(text);
        segments.push(text.to_owned());
    }
    let path_text = segments.join("/");
    if path_text.is_empty() || !Identifier::validate_path(&path_text) {
        println!(
            "cargo:warning=Skipping function with an invalid identifier: {}",
            path.display()
        );
        return None;
    }
    Some(Identifier::new("minecraft", path_text))
}

/// A `static` declaration for one function.
fn declare(function: &ParsedFunction) -> TokenStream {
    let ident = ident(&function.key);
    let key = identifier(&function.key);
    let lines = function.commands.iter().map(|command| {
        let literal = Literal::string(command);
        quote! { #literal }
    });

    quote! {
        static #ident: Function = Function {
            key: #key,
            commands: &[#(#lines),*],
        };
    }
}

/// A Rust identifier derived from the function id, unique per file path.
fn ident(key: &Identifier) -> proc_macro2::Ident {
    let mut name = String::from("FUNCTION_");
    for byte in key.to_string().bytes() {
        if byte.is_ascii_alphanumeric() {
            name.push((byte as char).to_ascii_uppercase());
        } else {
            name.push('_');
        }
    }
    proc_macro2::Ident::new(&name, Span::call_site())
}

/// The identifier expression for a function key.
fn identifier(key: &Identifier) -> TokenStream {
    let namespace = Literal::string(&key.namespace);
    let path = Literal::string(&key.path);
    quote! { Identifier::new_static(#namespace, #path) }
}
