# cmarkfmt

A CommonMark formatter library for Rust.

[![](https://img.shields.io/crates/v/cmarkfmt.svg)](https://crates.io/crates/cmarkfmt)

## Usage

```rust 
let input = r#"# This is markdown
It *needs* to be formatted."#;

let cmfmt = cmarkfmt::Formatter::default();
let output = cmfmt.format_cmark(input);
println!("{output}");
```
