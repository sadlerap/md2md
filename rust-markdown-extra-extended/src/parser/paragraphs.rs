use winnow::{
    bytes::take_till1,
    character::newline,
    multi::{many1, separated1},
    IResult, Parser, combinator::opt, sequence::terminated,
};

use super::util::MarkdownText;

#[derive(Debug, PartialEq, Eq)]
pub struct Paragraph<'source> {
    pub(crate) lines: Vec<Line<'source>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Line<'source> {
    pub(crate) content: Vec<MarkdownText<'source>>,
}

/// Parse a line.
///
/// Fundamentally, a line is the basic block of most blocks within markdown, including:
/// * Paragraphs
/// * Headings (more specifically, setext-style headings)
/// * Quote blocks
pub fn parse_line(input: &str) -> IResult<&str, Line> {
    take_till1("\n")
        .and_then(many1(MarkdownText::parse_markdown_text))
        .context("line of text")
        .map(|content| Line { content })
        .parse_next(input)
}

/// Parse a paragraph.
///
/// In markdown, a paragraph is one or more lines of markdown text.  Unlike other block types,
/// there isn't any special characters to delineate this block type from others, so blocks should
/// default to this.
pub fn parse_paragraph(input: &str) -> IResult<&str, Paragraph> {
    terminated(separated1(parse_line, newline), opt(newline))
        .context("paragraph")
        .map(|lines: Vec<Line<'_>>| Paragraph { lines })
        .parse_next(input)
}
