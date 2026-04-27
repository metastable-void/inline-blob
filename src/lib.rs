//! Inline gigabytes of data into a binary or library crate by emitting a
//! `static` placed in `.lrodata.*` so the linker can use large-code-model
//! relocations on 64-bit ELF targets.

use proc_macro::{TokenStream, TokenTree};
use std::str::FromStr;

/// `blob!(<vis> static NAME, <path-expr>);`
///
/// `<path-expr>` is forwarded verbatim to `include_bytes!`, so any expression
/// that the latter accepts works — a string literal, or e.g.
/// `concat!(env!("OUT_DIR"), "/blob.bin")` for a build-script-generated file.
///
/// Expands to:
///
/// ```ignore
/// #[used]
/// #[unsafe(link_section = ".lrodata.<lowercased name>")]
/// <vis> static NAME: [u8; const { include_bytes!("path/to/data").len() }]
///     = *include_bytes!("path/to/data");
/// ```
///
/// (`[u8; _]` is not yet allowed in `static` item signatures on stable, so the
/// length is computed via a `const` block. `include_bytes!` is evaluated at
/// compile time and the resulting array is deduplicated, so the binary only
/// carries one copy of the data.)
#[proc_macro]
pub fn blob(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();

    let static_idx = tokens
        .iter()
        .position(|t| matches!(t, TokenTree::Ident(i) if i.to_string() == "static"))
        .expect("inline-blob: expected `static` keyword (e.g. `pub static NAME, \"path\"`)");

    let vis_stream: TokenStream = tokens[..static_idx].iter().cloned().collect();
    let vis_str = vis_stream.to_string();

    let after = &tokens[static_idx + 1..];

    let name = match after.first() {
        Some(TokenTree::Ident(i)) => i.to_string(),
        _ => panic!("inline-blob: expected identifier after `static`"),
    };

    match after.get(1) {
        Some(TokenTree::Punct(p)) if p.as_char() == ',' => {}
        _ => panic!("inline-blob: expected `,` after identifier"),
    }

    let path_tokens: TokenStream = after[2..].iter().cloned().collect();
    if path_tokens.is_empty() {
        panic!("inline-blob: expected path expression after `,`");
    }
    let path_expr = path_tokens.to_string();

    let anchor_section = format!(".lbss.{}", name.to_lowercase());
    let section = format!(".lrodata.{}", name.to_lowercase());

    let generated = format!(
        "#[used] \
         #[unsafe(link_section = \"{anchor_section}\")] \
         static __ANCHOR_{name}: [u8; 1] = [0];
         #[used] \
         #[unsafe(link_section = \"{section}\")] \
         {vis_str} static {name}: [u8; const {{ ::core::include_bytes!({path_expr}).len() }}] \
         = *::core::include_bytes!({path_expr});"
    );

    TokenStream::from_str(&generated)
        .expect("inline-blob: failed to construct output token stream")
}
