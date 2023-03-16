use std::{borrow::Cow, io};

use once_cell::sync::OnceCell;
use regex::{Regex, RegexBuilder};
use winnow::{multi::many1, FinishIResult, Parser};

pub mod parser;

pub trait AsText {
    fn write_as_text<Writer: io::Write>(&self, output: &mut Writer) -> io::Result<()>;
}

pub fn cleanup(data: &'_ str, tab_width: usize) -> Cow<'_, str> {
    static BOM_RE: OnceCell<Regex> = OnceCell::new();
    static LINE_ENDING_RE: OnceCell<Regex> = OnceCell::new();
    static DETAB_RE: OnceCell<Regex> = OnceCell::new();
    static STRIP_WHITESPACE_RE: OnceCell<Regex> = OnceCell::new();

    let data = BOM_RE
        .get_or_init(|| {
            RegexBuilder::new(r"^\xEF\xBB\xBF|\x1A")
                .build()
                .expect("failed to build re")
        })
        .replace_all(data, "");
    let data = LINE_ENDING_RE
        .get_or_init(|| {
            RegexBuilder::new(r"\r\n?")
                .build()
                .expect("failed to build re")
        })
        .replace_all(&data, "\n")
        .into_owned();

    // upstream does this, so we do it too
    // data.push_str("\n\n");
    DETAB_RE
        .get_or_init(|| {
            RegexBuilder::new(r"^.*\t.*$")
                .multi_line(true)
                .build()
                .expect("failed to build re")
        })
        .replace_all(&data, |captures: &regex::Captures| -> String {
            let Some(x) = captures.get(0).map(|m| m.as_str()) else { return "".into() };
            let mut iter = x.split('\t');
            let Some(mut line) = iter.next().map(|s| s.to_string()) else { return "".to_string() };
            for x in iter {
                let amount = (tab_width - x.chars().count()) % tab_width;
                std::iter::repeat(" ")
                    .take(amount)
                    .for_each(|s| line.push_str(s));
            }

            line
        });
    STRIP_WHITESPACE_RE
        .get_or_init(|| {
            RegexBuilder::new(r"^[ ]+$")
                .multi_line(true)
                .build()
                .expect("failed to build re")
        })
        .replace_all(&data, "")
        .into_owned();

    // same here, even though it's not super necessary
    // data.push('\n');
    data.into()
}

/// A parsed representation of a Markdown file
pub struct Markdown<'source> {
    blocks: Vec<parser::block::Block<'source>>,
}

impl<'source> Markdown<'source> {
    pub fn parse(input: &'source str) -> color_eyre::Result<Self> {
        many1(parser::block::parse_block)
            .context("markdown text")
            .map(|blocks| Markdown { blocks })
            .parse_next(input)
            .finish()
            .map_err(|e| color_eyre::eyre::eyre!("parsing error: {:?}", e))
    }
}

impl<'source> AsText for Markdown<'source> {
    fn write_as_text<Writer: io::Write>(&self, output: &mut Writer) -> io::Result<()> {
        for b in self.blocks.iter() {
            b.write_as_text(output)?;
        }

        Ok(())
    }
}
