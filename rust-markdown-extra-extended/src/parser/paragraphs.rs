use winnow::{
    bytes::take_till1,
    character::newline,
    combinator::opt,
    multi::{many1, many0},
    sequence::preceded,
    stream::Accumulate,
    IResult, Parser,
};

use super::util::MarkdownText;

#[derive(Debug, PartialEq, Eq)]
pub struct Paragraph<'source> {
    pub(crate) content: Vec<MarkdownText<'source>>,
}

/// Parse a line.
///
/// Fundamentally, a line is the basic block of most blocks within markdown, including:
/// * Paragraphs
/// * Headings (more specifically, setext-style headings)
/// * Quote blocks
pub fn parse_line<'source, A>(input: &'source str) -> IResult<&'source str, A>
where
    A: Accumulate<MarkdownText<'source>>,
{
    take_till1("\n")
        .and_then(many1(MarkdownText::parse_markdown_text))
        .context("line of text")
        .parse_next(input)
}

/// Parse a paragraph.
///
/// In markdown, a paragraph is one or more lines of markdown text.  Unlike other block types,
/// there isn't any special characters to delineate this block type from others, so blocks should
/// default to this.
pub fn parse_paragraph(input: &str) -> IResult<&str, Paragraph> {
    ((
        parse_line,
        many0(preceded(newline, parse_line)),
        opt(newline)))
        .context("paragraph")
        .map(
            |x: (
                Vec<MarkdownText<'_>>,
                Vec<Vec<MarkdownText<'_>>>,
                Option<char>,
            )| {
                let mut content = x.0;
                content.extend(x.1.into_iter().flatten());
                Paragraph { content }
            },
        )
        .parse_next(input)
}
