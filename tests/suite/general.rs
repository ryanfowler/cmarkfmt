use super::test_cmark;

#[test]
fn test_list1() {
    let input = r#"
# This is a heading 
This is a paragraph. About not *much*.
It ~~spans~~ multiple *lines*. 

```json
{
    "key": "val"
}
```

    { "key1": 100 }

    { "key2: 101 }

---


<table>
<tr><td>Hi</td></tr>
</table>

<span>Some text</span>

Here's a **separate** paragraph.  
And another line.
"#;

    let expected = r#"# This is a heading

This is a paragraph. About not _much_.
It ~~spans~~ multiple _lines_.

```json
{
    "key": "val"
}
```

    { "key1": 100 }

    { "key2: 101 }

---

<table>
<tr><td>Hi</td></tr>
</table>

<span>Some text</span>

Here's a **separate** paragraph.\
And another line.
"#;

    test_cmark(input, expected);
}
