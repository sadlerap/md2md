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

impl<'source> MarkdownText<'source> {
    pub fn parse_markdown_text_until<F>(
        input: &'source str,
        matcher: F,
    ) -> IResult<&'source str, Self>
    where
        F: ContainsToken<<&'source str as Stream>::Token>,
    {
        alt((
            parse_image.map(|image| MarkdownText::Image(image)),
            parse_link.map(|link| MarkdownText::Link(link)),
            parse_auto_link.map(|auto_link| MarkdownText::AutoLink(auto_link)),
            take_till0(matcher)
                .recognize()
                .context("text data")
                .map(|s: &str| MarkdownText::Text(s)),
        ))
        .context("markdown leaf node")
        .parse_next(input)
    }

    fn take1(input: &'source str) -> IResult<&'source str, Self> {
        take(1usize)
            .map(|s| MarkdownText::Text(s))
            .context("take 1 character")
            .parse_next(input)
    }

    pub fn parse_markdown_text(input: &'source str) -> IResult<&'source str, Self> {
        let parser = dispatch! {peek(alt((take_while1("![<`\n"), take(1usize))));
            "![" => alt((
                parse_image.context("image").map(|i| MarkdownText::Image(i)),
                MarkdownText::take1,
            )),
            "[" => alt((
                parse_link.context("link").map(|l| MarkdownText::Link(l)),
                MarkdownText::take1,
            )),
            "`" => alt((
                parse_inline_code.context("code"),
                MarkdownText::take1,
            )),
            "<" => alt((
                parse_auto_link.context("auto link").map(|a| MarkdownText::AutoLink(a)),
                MarkdownText::take1,
            )),
            "\n" => multispace1.map(|_| MarkdownText::SoftBreak).context("soft break"),
            _ => alt((
                take_till1("\n[]<>!").map(|t| MarkdownText::Text(t)),
                terminated(
                    many1(any).map(|_: ()| {}).recognize(),
                    opt(newline)).map(|t| MarkdownText::Text(t))
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
