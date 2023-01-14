use super::test_cmark;

#[test]
fn test_link1() {
    let input = r#"
[basic link](https://example.com)"#;

    let expected = r#"[basic link](https://example.com)
"#;

    test_cmark(input, expected);
}

#[test]
fn test_link2() {
    let input = r#"
Inline [basic link](https://example.com) that's part of a sentence."#;

    let expected = r#"Inline [basic link](https://example.com) that's part of a sentence.
"#;

    test_cmark(input, expected);
}

#[test]
fn test_link3() {
    let input = r#"
Here's a [shortcut]. It should be preserved.

[shortcut]: https://example.com"#;

    let expected = r#"Here's a [shortcut]. It should be preserved.

[shortcut]: https://example.com
"#;

    test_cmark(input, expected);
}

#[test]
fn test_link4() {
    let input = r#"
Here's a [collapsed][] link. It should be preserved.

[collapsed]: https://example.com"#;

    let expected = r#"Here's a [collapsed][] link. It should be preserved.

[collapsed]: https://example.com
"#;

    test_cmark(input, expected);
}

#[test]
fn test_link5() {
    let input = r#"
Here's a [reference][link]. It should be preserved.

[link]: https://example.com"#;

    let expected = r#"Here's a [reference][link]. It should be preserved.

[link]: https://example.com
"#;

    test_cmark(input, expected);
}

#[test]
fn test_link6() {
    let input = r#"
Here's a [reference][link1]. It should be preserved.

[link1]: https://example.com

There can be multiple: [link2].

[link2]: https://example.com/2 "This is a title""#;

    let expected = r#"Here's a [reference][link1]. It should be preserved.

There can be multiple: [link2].

[link1]: https://example.com
[link2]: https://example.com/2 "This is a title"
"#;

    test_cmark(input, expected);
}

#[test]
fn test_link7() {
    let input = r#"
Here's a [reference][link1].
"#;

    let expected = r#"Here's a \[reference\]\[link1\].
"#;

    test_cmark(input, expected);
}

#[test]
fn test_link8() {
    let input = r#"
Here's an <autolink>.
"#;

    let expected = r#"Here's an <autolink>.
"#;

    test_cmark(input, expected);
}
