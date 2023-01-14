//! # cmarkfmt
//!
//! A library for formatting CommonMark files.
//!
//! ## Usage
//!
//! ```
//! let input = r#"# This is markdown
//! It *needs* to be formatted."#;
//!
//! let cmfmt = cmarkfmt::Formatter::default();
//! let output = cmfmt.format_cmark(input);
//! println!("{output}");
//! ```

use std::fmt::{self, Debug, Write};

use pulldown_cmark::{
    Alignment, CodeBlockKind, Event, HeadingLevel, LinkType, Options as POptions, Parser, Tag,
};

/// Function for formatting code blocks within markdown.
///
/// The first parameter is the language, and the second parameter is the code
/// itself. If formatted, returns Some(String) with the code block to use.
pub type CodeFormatFn<'a> = &'a dyn Fn(&str, &str) -> Option<String>;

/// A `Formatter` is needed to format markdown. It is created and customized as
/// needed using the `with_*` methods.
///
/// Once created, the `format_cmark` or `format_cmark_writer` methods can be
/// used.
#[derive(Clone)]
pub struct Formatter<'a> {
    code_fmt: Option<CodeFormatFn<'a>>,
    blockquote: &'a str,
    emphasis: &'a str,
    unordered_list: &'a str,
}

impl Default for Formatter<'_> {
    fn default() -> Self {
        Self {
            code_fmt: None,
            blockquote: ">",
            emphasis: "_",
            unordered_list: "-",
        }
    }
}

impl Debug for Formatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FormatBuilder")
            .field("code_fmt", &self.code_fmt.map(|_| ()))
            .field("blockquote", &self.blockquote)
            .field("emphasis", &self.emphasis)
            .field("unordered_list", &self.unordered_list)
            .finish()
    }
}

impl<'a> Formatter<'a> {
    /// Format markdown, returning the formatted result as a String.
    pub fn format_cmark(&self, input: &str) -> String {
        let mut out = String::with_capacity(input.len() + 128);
        self.format_cmark_writer(input, &mut out).unwrap();
        out
    }

    /// Format markdown, writing the result to the provided Writer.
    pub fn format_cmark_writer<W: fmt::Write>(&self, input: &str, w: W) -> fmt::Result {
        let mut opts = POptions::all();
        opts.remove(POptions::ENABLE_SMART_PUNCTUATION);
        let parser = Parser::new_ext(input, opts);

        let mut refdefs = parser
            .reference_definitions()
            .iter()
            .map(|(label, linkdef)| Reference {
                label: label.to_owned(),
                dest: linkdef.dest.to_string(),
                title: linkdef.title.as_ref().map(|v| v.to_string()),
            })
            .collect::<Vec<_>>();
        refdefs.sort_by(|r1, r2| r1.label.cmp(&r2.label));

        let mut ctx = Context::new(w, refdefs, self.into());
        ctx.format(parser)
    }

    /// Sets the `Formatter`s code formatter function. By default, code blocks
    /// are not formatted.
    pub fn with_code_formatter(self, code_fmt: Option<CodeFormatFn<'a>>) -> Self {
        Formatter { code_fmt, ..self }
    }

    /// Sets the blockquote string. Default: ">".
    pub fn with_blockquote(self, blockquote: &'a str) -> Self {
        Formatter { blockquote, ..self }
    }

    /// Sets the emphasis string. Default: "_".
    pub fn with_emphasis(self, emphasis: &'a str) -> Self {
        Formatter { emphasis, ..self }
    }

    /// Sets the unordered list string. Default: "-".
    pub fn with_unordered_list(self, unordered_list: &'a str) -> Self {
        Formatter {
            unordered_list,
            ..self
        }
    }
}

const STRONG: &str = "**";
const STRIKETHROUGH: &str = "~~";

enum StackItem {
    Blockquote,
    CodeIndent,
    List(Option<String>, bool, bool),
}

struct Options<'a> {
    code_fmt: &'a Option<CodeFormatFn<'a>>,
    blockquote_str: &'a str,
    emphasis_str: &'a str,
    unordered_list_str: &'a str,
}

impl<'a> From<&'a Formatter<'a>> for Options<'a> {
    fn from(v: &'a Formatter<'a>) -> Self {
        Options {
            code_fmt: &v.code_fmt,
            blockquote_str: v.blockquote,
            emphasis_str: v.emphasis,
            unordered_list_str: v.unordered_list,
        }
    }
}

struct Context<'a, W: fmt::Write> {
    writer: W,
    refdefs: Vec<Reference>,
    opts: Options<'a>,
    table: Option<Table>,
    stack: Vec<StackItem>,
    text_buf: String,
    scratch: String,
    newline_required: bool,
    code_block: Option<Option<String>>,
    last_line_blank: bool,
}

impl<'a, W: fmt::Write> Context<'a, W> {
    fn new(writer: W, refdefs: Vec<Reference>, opts: Options<'a>) -> Self {
        Context {
            writer,
            refdefs,
            opts,
            table: None,
            stack: Vec::new(),
            scratch: String::with_capacity(512),
            text_buf: String::with_capacity(512),
            newline_required: false,
            code_block: None,
            last_line_blank: true,
        }
    }

    fn format(&mut self, parser: Parser) -> fmt::Result {
        let mut is_last_html = false;
        for event in parser {
            #[cfg(debug_assertions)]
            println!("{event:?}");

            if is_last_html {
                match event {
                    Event::Html(_) | Event::Text(_) | Event::SoftBreak | Event::End(_) => {}
                    _ => self.write_newline()?,
                }
                is_last_html = false;
            }

            match event {
                Event::Start(tag) => self.tag_start(tag)?,
                Event::End(tag) => self.tag_end(tag)?,
                Event::Text(s) => {
                    let out: String;
                    let mut text: &str = &s;
                    if let Some(Some(lang)) = &self.code_block {
                        if let Some(code_fmt) = &self.opts.code_fmt {
                            if let Some(v) = (code_fmt)(lang, &s) {
                                out = v;
                                text = &out;
                            }
                        }
                    }
                    self.write_optional_escape(text)?;
                    self.write_str(text)?;
                }
                Event::Code(s) => {
                    self.write_char('`')?;
                    if let Some('`') = s.chars().next() {
                        self.write_backslash()?;
                    }
                    self.write_str(&s)?;
                    self.write_char('`')?;
                }
                Event::Html(s) => {
                    if self.text_buf.is_empty() {
                        self.write_newline_if_required()?;
                    }
                    self.write_str(&s)?;
                    if s.ends_with('\n') {
                        self.write_newline()?;
                    }
                    is_last_html = true;
                }
                Event::SoftBreak => self.write_newline()?,
                Event::HardBreak => {
                    //self.write_str("  ")?;
                    self.write_char('\\')?;
                    self.write_newline_with_trim(false)?;
                }
                Event::Rule => {
                    if self.newline_required {
                        self.write_newline()?;
                    }
                    self.write_str("---")?;
                    self.write_newline()?;
                    self.newline_required = true;
                }
                Event::TaskListMarker(is_checked) => {
                    self.write_char('[')?;
                    self.write_char(if is_checked { 'x' } else { ' ' })?;
                    self.write_str("] ")?;
                }
                Event::FootnoteReference(label) => {
                    self.write_str("[^")?;
                    self.write_str(&label)?;
                    self.write_char(']')?;
                }
            }
        }

        let refdefs = std::mem::take(&mut self.refdefs);
        if !refdefs.is_empty() {
            self.write_newline()?;
            for refdef in refdefs {
                self.write_char('[')?;
                self.write_str(&refdef.label)?;
                self.write_str("]: ")?;
                self.write_str(&refdef.dest)?;
                if let Some(title) = refdef.title {
                    self.write_str(" \"")?;
                    self.write_str(&title)?;
                    self.write_char('"')?;
                }
                self.write_newline()?;
            }
        }

        Ok(())
    }

    fn tag_start(&mut self, tag: Tag) -> fmt::Result {
        self.write_newline_if_required()?;
        match tag {
            Tag::Heading(lvl, _, _) => self.write_heading_level(lvl)?,
            Tag::BlockQuote => self.stack.push(StackItem::Blockquote),
            Tag::CodeBlock(kind) => {
                if !self.text_buf.is_empty() {
                    self.write_newline()?;
                }
                match kind {
                    CodeBlockKind::Indented => {
                        self.code_block = Some(None);
                        self.stack.push(StackItem::CodeIndent)
                    }
                    CodeBlockKind::Fenced(s) => {
                        self.write_str("```")?;
                        self.write_str(&s)?;
                        self.write_newline()?;
                        self.code_block = Some(Some(s.into_string()));
                    }
                }
            }
            Tag::List(l) => {
                if let Some(StackItem::List(_, _, newline)) = self.stack.last_mut() {
                    *newline = true;
                    self.write_newline()?;
                }
                let l = l.map(|v| v.to_string());
                self.stack.push(StackItem::List(l, false, false));
            }
            Tag::Item => {
                if let Some(StackItem::List(_, written, newline)) = self.stack.last_mut() {
                    *written = false;
                    *newline = false;
                }
            }
            Tag::FootnoteDefinition(value) => {
                self.write_str("[^")?;
                self.write_str(&value)?;
                self.write_str("]: ")?;
            }
            Tag::Table(alignments) => self.table = Some(Table::new(alignments)),
            Tag::TableRow => {
                if let Some(table) = self.table.as_mut() {
                    table.body.push(Vec::with_capacity(table.head.len()));
                }
            }
            Tag::Emphasis => self.write_str(self.opts.emphasis_str)?,
            Tag::Strong => self.write_str(STRONG)?,
            Tag::Strikethrough => self.write_str(STRIKETHROUGH)?,
            Tag::Link(typ, _, _) => match typ {
                LinkType::Autolink | LinkType::Email => self.write_char('<')?,
                _ => self.write_char('[')?,
            },
            Tag::Image(_, _, _) => self.write_str("![")?,
            Tag::Paragraph | Tag::TableHead | Tag::TableCell => {}
        }
        Ok(())
    }

    fn tag_end(&mut self, tag: Tag) -> fmt::Result {
        match tag {
            Tag::Paragraph => {
                if !matches!(self.stack.last(), Some(StackItem::List(..))) {
                    self.newline_required = true;
                }
                if let Some(StackItem::List(_, _, newline)) = self.stack.last_mut() {
                    *newline = true;
                }
                self.write_newline_if_content()
            }
            Tag::Heading(_, id, classes) => {
                if id.is_some() || !classes.is_empty() {
                    self.write_char('{')?;
                    if let Some(id) = id {
                        self.write_str(" #")?;
                        self.write_str(id)?;
                    }
                    for class in classes {
                        self.write_str(" .")?;
                        self.write_str(class)?;
                    }
                    self.write_str(" }")?;
                }

                self.newline_required = true;
                self.write_newline()
            }
            Tag::BlockQuote => {
                self.stack.pop();
                if !matches!(self.stack.last(), Some(StackItem::List(..))) {
                    self.newline_required = true;
                }
                Ok(())
            }
            Tag::CodeBlock(kind) => {
                if let CodeBlockKind::Fenced(_) = kind {
                    self.write_str("```")?;
                }
                self.write_newline()?;
                if let CodeBlockKind::Indented = kind {
                    self.stack.pop();
                }
                self.newline_required = true;
                self.code_block = None;
                Ok(())
            }
            Tag::List(_) => {
                self.stack.pop();
                if !self
                    .stack
                    .iter()
                    .any(|v| matches!(&v, StackItem::List(_, _, _)))
                {
                    self.newline_required = true;
                }
                Ok(())
            }
            Tag::Item => {
                if let Some(StackItem::List(_, _, false)) = self.stack.last() {
                    self.write_newline_if_content()?;
                }
                Ok(())
            }
            Tag::Table(_) => {
                let table = match self.table.take() {
                    Some(table) => table,
                    None => return Ok(()),
                };
                let widths = table.column_widths();
                self.write_table_row(&table.head, &widths)?;

                self.write_char('|')?;
                for (w, a) in widths.iter().zip(table.alignments.iter()) {
                    self.write_char(' ')?;
                    self.write_char(if matches!(a, Alignment::Left | Alignment::Center) {
                        ':'
                    } else {
                        '-'
                    })?;
                    for _ in 0..*w - 2 {
                        self.write_char('-')?;
                    }
                    self.write_char(if matches!(a, Alignment::Right | Alignment::Center) {
                        ':'
                    } else {
                        '-'
                    })?;
                    self.write_str(" |")?;
                }
                self.write_newline()?;

                for b in &table.body {
                    self.write_table_row(b, &widths)?;
                }

                self.table = None;
                self.newline_required = true;

                if let Some(StackItem::List(_, _, newline)) = self.stack.last_mut() {
                    *newline = true;
                }

                Ok(())
            }
            Tag::TableCell => {
                if let Some(table) = self.table.as_mut() {
                    if let Some(b) = table.body.last_mut() {
                        b.push(self.text_buf.to_string());
                    } else {
                        table.head.push(self.text_buf.to_string());
                    }
                    self.text_buf.clear();
                }
                Ok(())
            }
            Tag::Emphasis => self.write_str(self.opts.emphasis_str),
            Tag::Strong => self.write_str(STRONG),
            Tag::Strikethrough => self.write_str(STRIKETHROUGH),
            Tag::Link(LinkType::Reference | LinkType::ReferenceUnknown, dest, title) => {
                let refdefs = std::mem::take(&mut self.refdefs);
                if let Some(refdef) = refdefs.iter().find(|v| dest.eq_ignore_ascii_case(&v.dest)) {
                    self.write_str("][")?;
                    self.write_str(&refdef.label)?;
                    self.write_char(']')?;
                } else {
                    self.write_str("](")?;
                    self.write_str(&dest)?;
                    if !title.is_empty() {
                        self.write_str(" \"")?;
                        self.write_str(&title)?;
                        self.write_char('"')?;
                    }
                    self.write_char(')')?;
                }
                self.refdefs = refdefs;
                Ok(())
            }
            Tag::Link(LinkType::Shortcut | LinkType::ShortcutUnknown, ..) => self.write_char(']'),
            Tag::Link(LinkType::Collapsed | LinkType::CollapsedUnknown, ..) => {
                self.write_str("][]")
            }
            Tag::Link(LinkType::Autolink | LinkType::Email, ..) => self.write_char('>'),
            Tag::Link(_, dest, title) | Tag::Image(_, dest, title) => {
                self.write_str("](")?;
                self.write_str(&dest)?;
                if !title.is_empty() {
                    self.write_str(" \"")?;
                    self.write_str(&title)?;
                    self.write_char('"')?;
                }
                self.write_char(')')
            }
            Tag::FootnoteDefinition(_) | Tag::TableHead | Tag::TableRow => Ok(()),
        }
    }

    fn write_optional_escape(&mut self, s: &str) -> fmt::Result {
        if self.code_block.is_some() {
            if s.starts_with("```") {
                self.write_backslash()?;
            }
            return Ok(());
        }
        if let Some(first) = s.chars().next() {
            if self.table.is_some() && first == '|' {
                return self.write_backslash();
            }
            match first {
                '\\' | '<' | '>' | '*' | '_' | '`' | '[' | ']' | '~' => {
                    return self.write_backslash()
                }
                '#' | '-' | '+' => {
                    if self.text_buf.is_empty() {
                        return self.write_backslash();
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn write_backslash(&mut self) -> fmt::Result {
        self.text_buf.write_char('\\')
    }

    fn write_table_row(&mut self, row: &[String], widths: &[usize]) -> fmt::Result {
        self.write_str("|")?;
        for (s, w) in row.iter().zip(widths.iter()) {
            let width = s.chars().count();
            self.write_char(' ')?;
            self.write_str(s)?;
            for _ in 0..(w - width) {
                self.write_char(' ')?;
            }
            self.write_str(" |")?;
        }
        self.write_newline()
    }

    fn write_newline_if_required(&mut self) -> fmt::Result {
        if self.newline_required {
            self.write_newline()?;
            self.newline_required = false;
        }
        Ok(())
    }

    fn write_newline_if_content(&mut self) -> fmt::Result {
        if !self.text_buf.is_empty() || !self.stack.is_empty() {
            self.write_newline()?;
        }
        Ok(())
    }

    fn write_heading_level(&mut self, lvl: HeadingLevel) -> fmt::Result {
        match lvl {
            HeadingLevel::H1 => self.write_str("# "),
            HeadingLevel::H2 => self.write_str("## "),
            HeadingLevel::H3 => self.write_str("### "),
            HeadingLevel::H4 => self.write_str("#### "),
            HeadingLevel::H5 => self.write_str("##### "),
            HeadingLevel::H6 => self.write_str("###### "),
        }
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.text_buf.write_str(s)
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.text_buf.write_char(c)
    }

    fn write_newline(&mut self) -> fmt::Result {
        self.write_newline_with_trim(true)
    }

    fn write_newline_with_trim(&mut self, trim: bool) -> fmt::Result {
        if !self.text_buf.is_empty() {
            let mut text_buf = std::mem::take(&mut self.text_buf);
            for line in text_buf.lines() {
                self.write_line(line, trim)?;
            }
            text_buf.clear();
            self.text_buf = text_buf;
        } else {
            self.write_line("", trim)?;
        }
        Ok(())
    }

    fn write_line(&mut self, line: &str, trim: bool) -> fmt::Result {
        self.write_padding_to_scratch()?;
        self.scratch.write_str(line)?;
        let buf = if trim {
            self.scratch.trim_end()
        } else {
            &self.scratch
        };
        if !buf.is_empty() || !self.last_line_blank {
            self.writer.write_str(buf)?;
            self.writer.write_char('\n')?;
        }
        self.last_line_blank = buf.is_empty();
        self.scratch.clear();
        Ok(())
    }

    fn write_padding_to_scratch(&mut self) -> fmt::Result {
        for item in self.stack.iter_mut() {
            match item {
                StackItem::Blockquote => {
                    self.scratch.write_str(self.opts.blockquote_str)?;
                    self.scratch.write_char(' ')?
                }
                StackItem::CodeIndent => self.scratch.write_str("    ")?,
                StackItem::List(l, written, _) => {
                    if *written {
                        match l {
                            None => {
                                for _ in 0..self.opts.unordered_list_str.chars().count() + 1 {
                                    self.scratch.write_char(' ')?;
                                }
                            }
                            Some(n) => {
                                for _ in 0..n.chars().count() + 2 {
                                    self.scratch.write_char(' ')?;
                                }
                            }
                        }
                    } else {
                        *written = true;
                        match l {
                            None => {
                                self.scratch.write_str(self.opts.unordered_list_str)?;
                                self.scratch.write_char(' ')?
                            }
                            Some(n) => {
                                self.scratch.write_str(n)?;
                                self.scratch.write_str(". ")?
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

struct Table {
    alignments: Vec<Alignment>,
    head: Vec<String>,
    body: Vec<Vec<String>>,
}

impl Table {
    fn new(alignments: Vec<Alignment>) -> Self {
        Table {
            alignments,
            head: Vec::new(),
            body: Vec::new(),
        }
    }

    fn column_widths(&self) -> Vec<usize> {
        self.head
            .iter()
            .enumerate()
            .map(|(i, h)| {
                self.body
                    .iter()
                    .map(|b| b.get(i).map(|b| b.chars().count()).unwrap_or(0))
                    .max()
                    .unwrap_or_default()
                    .max(h.chars().count())
                    .max(3)
            })
            .collect()
    }
}

struct Reference {
    label: String,
    dest: String,
    title: Option<String>,
}
