use super::test_cmark;

#[test]
fn test_table1() {
    let input = r#"
|Title|Description|
|---|---|
|Test|This is a test|"#;

    let expected = r#"| Title | Description    |
| ----- | -------------- |
| Test  | This is a test |
"#;

    test_cmark(input, expected);
}

#[test]
fn test_table2() {
    let input = r#"
|Title|Description|
|:--|---|
|Test|This is a test|"#;

    let expected = r#"| Title | Description    |
| :---- | -------------- |
| Test  | This is a test |
"#;

    test_cmark(input, expected);
}

#[test]
fn test_table3() {
    let input = r#"
|Title|Description|
|:-:|---|
|Test|This is a test|"#;

    let expected = r#"| Title | Description    |
| :---: | -------------- |
| Test  | This is a test |
"#;

    test_cmark(input, expected);
}

#[test]
fn test_table4() {
    let input = r#"
|Title|Description|
|--:|---|
|Test|This is a test|"#;

    let expected = r#"| Title | Description    |
| ----: | -------------- |
| Test  | This is a test |
"#;

    test_cmark(input, expected);
}

#[test]
fn test_table5() {
    let input = r#"
|Title| |
|:--|---|
|Test|This is a test|"#;

    let expected = r#"| Title |                |
| :---- | -------------- |
| Test  | This is a test |
"#;

    test_cmark(input, expected);
}

#[test]
fn test_table6() {
    let input = r#"
|Title| |
|:--|---|
|Test| |"#;

    let expected = r#"| Title |     |
| :---- | --- |
| Test  |     |
"#;

    test_cmark(input, expected);
}

#[test]
fn test_table7() {
    let input = r#"
> |Title|Description|
> |---|---|
> |Test|This is a test|"#;

    let expected = r#"> | Title | Description    |
> | ----- | -------------- |
> | Test  | This is a test |
"#;

    test_cmark(input, expected);
}

#[test]
fn test_table8() {
    let input = r#"
> - |Title|Description|
>   |---|---|
>   |Test|This is a test|"#;

    let expected = r#"> - | Title | Description    |
>   | ----- | -------------- |
>   | Test  | This is a test |
"#;

    test_cmark(input, expected);
}

#[test]
fn test_table9() {
    let input = r#"> Spacing required.


|Title|Description|
|---|---|
|Test|This is a test|"#;

    let expected = r#"> Spacing required.

| Title | Description    |
| ----- | -------------- |
| Test  | This is a test |
"#;

    test_cmark(input, expected);
}
