//! Generates `functions.rs`: every `.mcfunction` of the built-in vanilla datapack.
//!
//! Vanilla loads a function as a list of command lines and runs them in order
//! (`FunctionCommand`). The datapack is extracted from the server jar by
//! `steel-utils`' build script into
//! `steel-utils/build_assets/builtin_datapacks/minecraft` with the jar's `data/`
//! prefix stripped, so the tree is `<namespace>/function/**/*.mcfunction`.
//!
//! Lines are split and `#` comments and blank lines dropped exactly like
//! vanilla's `CommandFunction` parser. A function that references another
//! function by id resolves through the same registry, so the generated table
//! only needs the raw text.

use std::fs;
use std::path::{Path, PathBuf};

use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

/// Root of the extracted built-in vanilla datapack.
const DATAPACK_ROOT: &str = "../steel-utils/build_assets/builtin_datapacks/minecraft";
/// Namespace directory every extracted function lives under.
const NAMESPACE: &str = "minecraft";

/// A function collected from the datapack.
struct FunctionEntry {
    /// Datapack-relative path without the `.mcfunction` suffix, e.g. `demo/hello`.
    path: String,
    /// Identifier path of the function, e.g. `demo/hello`.
    ///
    /// `Identifier::vanilla_static` supplies the `minecraft` namespace itself,
    /// so only the path is stored.
    key: String,
    /// Command lines in execution order.
    lines: Vec<String>,
}

/// Walks `data/<namespace>/function`, collecting every `.mcfunction` file.
fn collect(root: &Path) -> Vec<FunctionEntry> {
    let mut found: Vec<PathBuf> = Vec::new();
    // `steel-utils`' build script strips the `data/` prefix while unpacking, so
    // the jar's `data/minecraft/function` lands as `minecraft/function`.
    let function_root = root.join(NAMESPACE).join("function");
    if !function_root.exists() {
        // A checkout without extracted assets (a plain `cargo check` before the
        // jar is fetched) still has to produce a valid, empty module rather than
        // failing the build: `lib.rs` always declares `functions`.
        println!(
            "cargo:warning=No functions found under {}; emitting an empty function table",
            function_root.display()
        );
        return Vec::new();
    }
    println!("cargo:rerun-if-changed={}", function_root.display());
    walk(&function_root, &mut found);

    let mut entries: Vec<FunctionEntry> = found
        .into_iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(&function_root).ok()?;
            let mut segments: Vec<String> = relative.parent().map_or_else(Vec::new, |parent| {
                parent
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy().into_owned())
                    .collect()
            });
            let stem = relative.file_stem()?.to_string_lossy().into_owned();
            segments.push(stem);
            let path_str = segments.join("/");

            let text = fs::read_to_string(&path).ok()?;
            let lines = text
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(str::to_owned)
                .collect();

            Some(FunctionEntry {
                key: path_str.clone(),
                path: path_str,
                lines,
            })
        })
        .collect();

    // Deterministic order: registration and generated identifiers must not
    // depend on the filesystem's iteration order.
    entries.sort_by(|left, right| left.key.cmp(&right.key));
    entries
}

/// Recursively pushes every `.mcfunction` file under `dir`.
fn walk(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(read) = fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "mcfunction") {
            found.push(path);
        }
    }
}

/// Builds the generated function module.
pub(crate) fn build() -> TokenStream {
    let functions = collect(Path::new(DATAPACK_ROOT));

    let mut stream = TokenStream::new();
    stream.extend(quote! {
        //! Generated vanilla datapack functions.

        use crate::function::{Function, FunctionRef};
        use steel_utils::Identifier;
    });

    // One static per function, named after its path, plus the lookup table.
    let mut static_defs = TokenStream::new();
    let mut refs = TokenStream::new();
    let mut seen_idents: Vec<String> = Vec::new();

    for entry in &functions {
        let mut ident_name = entry.path.to_shouty_snake_case();
        // A duplicated identifier would not compile, so disambiguate rather
        // than fail the build on an unexpected datapack layout.
        if seen_idents.contains(&ident_name) {
            ident_name.push_str("_FUNCTION");
        }
        seen_idents.push(ident_name.clone());

        let ident = Ident::new(&ident_name, Span::call_site());
        let key = &entry.key;
        let command_lines = entry.lines.iter().map(|line| quote! { #line });

        static_defs.extend(quote! {
            /// Generated from the built-in vanilla datapack.
            pub static #ident: Function = Function {
                key: Identifier::vanilla_static(#key),
                commands: &[#(#command_lines),*],
            };
        });
        refs.extend(quote! { &#ident, });
    }

    stream.extend(static_defs);
    // A slice rather than a populated registry: `FxHashMap` cannot be built in a
    // `const` context, and the ordering is already deterministic from `collect`.
    stream.extend(quote! {
        /// Every function in the built-in vanilla datapack, sorted by identifier.
        pub static FUNCTIONS: &[FunctionRef] = &[#refs];
    });
    stream
}
