use winnow::{
    branch::alt,
    bytes::{one_of, take_until1},
    character::newline,
    error::{ErrMode::Backtrack, Error, ParseError},
    multi::count,
    trace::trace,
    IResult, Parser,
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

pub fn find_next<'a, F, O>(mut parser: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, winnow::error::Error<&'a str>>
where
    F: Parser<&'a str, O, winnow::error::Error<&'a str>>,
{
    trace("find_next", move |input: &'a str| {
        for i in 0..input.len() {
            let (_, rest) = input.split_at(i);
            if let Ok((_remaining, result)) = parser.parse_next(rest) {
                return Ok((rest, result));
            }
        }
        Err(Backtrack(ParseError::from_error_kind(input, winnow::error::ErrorKind::Fail)))
    })
}

/// Takes until the given parser matches the input stream.  If the parser never matches, the input
/// is consumed.  The result of the given parser is not consumed.
pub fn take_until_match<'a, F, O>(mut parser: F) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str, winnow::error::Error<&'a str>>
where F: Parser<&'a str, O, winnow::error::Error<&'a str>>
{
    trace("take_until_matches", move |input: &'a str| {
        for i in 0..=input.len() {
            let (first, rest) = input.split_at(i);
            if parser.parse_next(rest).is_ok() {
                return Ok((rest, first));
            }
        }
        Err(Backtrack(ParseError::from_error_kind(input, winnow::error::ErrorKind::Fail)))
    })
}

/// Parse a paragraph.
///
/// In markdown, a paragraph is one or more lines of markdown text.  Unlike other block types,
/// there isn't any special characters to delineate this block type from others, so blocks should
/// default to this.
pub fn parse_paragraph(input: &str) -> IResult<&str, Paragraph> {
    let block_termination_chars = "=-#";
    let mut stream_parser = MarkdownText::parse_markdown_text_stream
        .map(|text| Paragraph { text })
        .context("paragraph");

    match find_next(alt((
        count(newline, 2).map(|_: ()| {}).context("2 newlines"),
        (
            newline::<&str, Error<&str>>,
            one_of(block_termination_chars),
        )
            .recognize()
            .context("searching for header characters")
            .void(),
    )).recognize())
    .parse_next(input)
    {
        Ok((_, text)) => take_until1(text)
            .and_then(stream_parser)
            .parse_next(input),
        Err(_) => stream_parser.parse_next(input),
    }
}
