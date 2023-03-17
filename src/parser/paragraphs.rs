use winnow::{
    branch::alt, character::newline, combinator::eof, multi::many1, sequence::terminated, IResult,
    Parser,
};

use crate::{AsHtml, AsText};

use super::util::MarkdownText;

#[derive(Debug, PartialEq, Eq)]
pub struct Paragraph<'source> {
    pub(crate) text: Vec<MarkdownText<'source>>,
}

impl<'source> AsHtml for Paragraph<'source> {
    fn write_html<Writer: std::io::Write>(&self, output: &mut Writer) -> std::io::Result<()> {
        for t in self.text.iter() {
            t.write_html(output)?;
        }

        Ok(())
    }
}

impl<'source> AsText for Paragraph<'source> {
    fn write_as_text<Writer: std::io::Write>(&self, output: &mut Writer) -> std::io::Result<()> {
        for t in self.text.iter() {
            t.write_as_text(output)?;
        }

        Ok(())
    }
}

/// Parse a paragraph.
///
/// In markdown, a paragraph is one or more lines of markdown text.  Unlike other block types,
/// there isn't any special characters to delineate this block type from others, so blocks should
/// default to this.
pub fn parse_paragraph(input: &str) -> IResult<&str, Paragraph> {
    terminated(
        many1(MarkdownText::parse_markdown_text),
        alt((newline.void(), eof.void())),
    )
    .context("paragraph")
    .map(|text: Vec<MarkdownText<'_>>| Paragraph { text })
    .parse_next(input)
}
