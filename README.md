# cmarkfmt

A CommonMark formatter library for Rust.

## Usage

```rust 
let input = r#"# This is markdown
It *needs* to be formatted."#;

let cmfmt = cmarkfmt::Formatter::default();
let output = cmfmt.format_cmark(input);
println!("{output}");
```
