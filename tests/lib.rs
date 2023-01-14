use cmarkfmt::FormatBuilder;

mod suite;

pub fn test_cmark(input: &str, expected: &str) {
    let out = FormatBuilder::default().format_cmark(input);
    assert_eq!(expected, &out);
}
