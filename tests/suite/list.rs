use super::test_cmark;

#[test]
fn test_list1() {
    let input = r#"
* item 1

* item 2
* item 3"#;

    let expected = r#"- item 1
- item 2
- item 3
"#;

    test_cmark(input, expected);
}

#[test]
fn test_list2() {
    let input = r#"
* item 1
  * item 2
  * item 3
    * item 4"#;

    let expected = r#"- item 1
  - item 2
  - item 3
    - item 4
"#;

    test_cmark(input, expected);
}

#[test]
fn test_list3() {
    let input = r#"
* Multiple
  line
  list
* Next item"#;

    let expected = r#"- Multiple
  line
  list
- Next item
"#;

    test_cmark(input, expected);
}

#[test]
fn test_list4() {
    let input = r#"
* > blockquote
  > inside
  > list
* Next item"#;

    let expected = r#"- > blockquote
  > inside
  > list

- Next item
"#;

    test_cmark(input, expected);
}
