# inline-blob

A proc_macro Rust crate that allows you to include gigabytes
of data inline in binaries (including lib crates).

## How it works

This expands:

```rust
use inline_blob::blob;

blob!(pub static MY_DATA, "path/to/data/bin");
```

into something like:

```rust
#[used]
#[unsafe(link_section = ".lrodata.my_data")]
pub static MY_DATA: [u8; _] = *include_bytes!("path/to/bin");
```

Visibility is supported.

```rust
use inline_blob::blob;

blob!(pub(crate) static MY_DATA, "path/to/bin");
```

is expanded into something like:

```rust
#[used]
#[unsafe(link_section = ".lrodata.my_data")]
pub(crate) static MY_DATA: [u8; _] = *include_bytes!("path/to/bin");
```

We only tested this crate for x86_64 ELF targets (specifically musl and GNU).
