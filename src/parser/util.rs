use std::borrow::Cow;

use winnow::{
    branch::alt,
    bytes::{any, none_of, take, take_till0, take_till1, take_while1},
    character::{multispace1, newline},
    combinator::{opt, peek},
    dispatch,
    multi::{many0, many1},
    sequence::{delimited, terminated},
    stream::{Accumulate, ContainsToken, Stream},
    IResult, Parser,
};

use crate::AsText;

use super::{
    code::parse_inline_code,
    images::{parse_image, Image},
    links::{parse_auto_link, parse_link, AutoLink, Link},
};

/// The various kinds of text that we can parse
#[derive(Debug, PartialEq, Eq)]
pub enum MarkdownText<'source> {
    Text(&'source str),
    Image(Image<'source>),
    Link(Link<'source>),
    AutoLink(AutoLink<'source>),
    SoftBreak,
    Code{code: Cow<'source, str>},
}

impl<'source> AsText for MarkdownText<'source> {
    fn write_as_text<Writer: std::io::Write>(&self, output: &mut Writer) -> std::io::Result<()> {
        match self {
            MarkdownText::Text(t) => write!(output, "{t}")?,
            MarkdownText::Image(image) => image.write_as_text(output)?,
            MarkdownText::Link(link) => link.write_as_text(output)?,
            MarkdownText::AutoLink(link) => link.write_as_text(output)?,
            MarkdownText::SoftBreak => writeln!(output)?,
            MarkdownText::Code { code } => write!(output, "`{code}`")?,
        }
        Ok(())
    }
}

impl<'source> MarkdownText<'source> {
    pub fn parse_markdown_text_until<F>(
        input: &'source str,
        matcher: F,
    ) -> IResult<&'source str, Self>
    where
        F: ContainsToken<<&'source str as Stream>::Token>,
    {
        alt((
            parse_image.map(MarkdownText::Image),
            parse_link.map(MarkdownText::Link),
            parse_auto_link.map(MarkdownText::AutoLink),
            take_till0(matcher)
                .recognize()
                .context("text data")
                .map(MarkdownText::Text),
        ))
        .context("markdown leaf node")
        .parse_next(input)
    }

    fn take1(input: &'source str) -> IResult<&'source str, Self> {
        take(1usize)
            .map(MarkdownText::Text)
            .context("take 1 character")
            .parse_next(input)
    }

    pub fn parse_markdown_text(input: &'source str) -> IResult<&'source str, Self> {
        let parser = dispatch! {peek(alt((take_while1("![<`\n"), take(1usize))));
            "![" => alt((
                parse_image.context("image").map(MarkdownText::Image),
                MarkdownText::take1,
            )),
            "[" => alt((
                parse_link.context("link").map(MarkdownText::Link),
                MarkdownText::take1,
            )),
            "`" => alt((
                parse_inline_code.context("code"),
                MarkdownText::take1,
            )),
            "<" => alt((
                parse_auto_link.context("auto link").map(MarkdownText::AutoLink),
                MarkdownText::take1,
            )),
            "\n" => multispace1.map(|_| MarkdownText::SoftBreak).context("soft break"),
            _ => alt((
                take_till1("\n[]<>!").map(MarkdownText::Text),
                terminated(
                    many1(any).map(|_: ()| {}).recognize(),
                    opt(newline)).map(MarkdownText::Text)
            )).context("text"),
        };

        parser.context("markdown text").parse_next(input)
    }

    pub fn parse_markdown_text_stream<A: Accumulate<Self>>(
        input: &'source str,
    ) -> IResult<&'source str, A> {
        many1(MarkdownText::parse_markdown_text)
            .context("stream of markdown text")
            .parse_next(input)
    }
}

pub fn nested_brackets(input: &str) -> IResult<&str, &str> {
    many0(alt((
        none_of("[]").context("non-bracketed text").recognize(),
        delimited("[", nested_brackets, "]")
            .context("bracketed text")
            .recognize(),
    )))
    .map(|_: ()| {})
    .recognize()
    .parse_next(input)
}

pub fn nested_parenthesis(input: &str) -> IResult<&str, &str> {
    many0(alt((
        none_of("()").context("non-parenthesis text").recognize(),
        delimited("(", nested_parenthesis, ")")
            .context("parenthetical text")
            .recognize(),
    )))
    .map(|_: ()| {})
    .recognize()
    .parse_next(input)
}
