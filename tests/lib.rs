use cmarkfmt::Formatter;

mod suite;

pub fn test_cmark(input: &str, expected: &str) {
    let out = Formatter::default().format_cmark(input);
    assert_eq!(expected, &out);
}
