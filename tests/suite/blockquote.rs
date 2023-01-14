use super::test_cmark;

#[test]
fn test_blockquote1() {
    let input = r#" >  This is a blockquote"#;

    let expected = r#"> This is a blockquote
"#;

    test_cmark(input, expected);
}

#[test]
fn test_blockquote2() {
    let input = r#"
>  This is a blockquote
> Multi-line"#;

    let expected = r#"> This is a blockquote
> Multi-line
"#;

    test_cmark(input, expected);
}

#[test]
fn test_blockquote3() {
    let input = r#"
> - With
    a
    list
> - Another item"#;

    let expected = r#"> - With
>   a
>   list
> - Another item
"#;

    test_cmark(input, expected);
}

#[test]
fn test_blockquote4() {
    let input = r#"
> - List item 1
>     - Nested item"#;

    let expected = r#"> - List item 1
>   - Nested item
"#;

    test_cmark(input, expected);
}

#[test]
fn test_blockquote5() {
    let input = r#"
> Blockquote
>> Nested
>>> Even more nested
> Back to original"#;

    let expected = r#"> Blockquote
>
> > Nested
> >
> > > Even more nested
> > > Back to original
"#;

    test_cmark(input, expected);
}
