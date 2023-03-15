use std::borrow::Cow;

use winnow::{
    branch::alt,
    bytes::{none_of, tag_no_case, take_until0, take_until1},
    character::{multispace0, newline, space0},
    combinator::opt,
    multi::many1,
    sequence::delimited,
    stream::Accumulate,
    IResult, Parser,
};

use crate::parser::util::{nested_brackets, nested_parenthesis};

use super::util::MarkdownText;

#[derive(Debug, PartialEq, Eq)]
enum LinkRef<'a> {
    Ref(&'a str),
    Inline(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Link<'source> {
    link_text: Vec<MarkdownText<'source>>,
    link_ref: LinkRef<'source>,
    title: Option<&'source str>,
}

/// A link where the target is the same as the text.  In markdown, this is constructed with
/// `<https://example.com>`
#[derive(Debug, PartialEq, Eq)]
pub struct AutoLink<'a> {
    target: Cow<'a, str>,
}

fn parse_brackets<'source, A>(input: &'source str) -> IResult<&'source str, A>
where
    A: Accumulate<MarkdownText<'source>>,
{
    delimited(
        "[",
        nested_brackets
            .recognize()
            .and_then(MarkdownText::parse_markdown_text_stream),
        "]",
    )
    .parse_next(input)
}

fn ref_style(input: &str) -> IResult<&str, Link> {
    (
        parse_brackets,
        opt(" "),
        opt((newline, space0)),
        delimited("[", take_until0("]"), "]"),
    )
        .map(|x| Link {
            link_text: x.0,
            link_ref: LinkRef::Ref(x.3),
            title: None,
        })
        .context("ref-style link")
        .parse_next(input)
}

fn inline_style(input: &str) -> IResult<&str, Link> {
    (
        parse_brackets,
        opt(" "),
        delimited(
            "(",
            (
                multispace0,
                nested_parenthesis,
                opt(multispace0),
                opt(alt((
                    delimited("\"", take_until0("\""), "\""),
                    delimited("\'", take_until0("\'"), "\'"),
                ))),
                opt(multispace0),
            )
                .map(|x| (x.1, x.3)),
            ")",
        ),
    )
        .map(|x| Link {
            link_text: x.0,
            link_ref: LinkRef::Inline(x.2 .0),
            title: x.2 .1,
        })
        .context("inline link")
        .parse_next(input)
}

pub fn parse_auto_link(input: &str) -> IResult<&str, AutoLink> {
    let email = delimited(
        "<",
        (
            opt(tag_no_case("mailto:")),
            take_until1("@"),
            "@",
            take_until1(">"),
        ),
        ">",
    )
    .context("email autolink")
    .map(|x| AutoLink {
        target: Cow::Owned(format!("mailto:{}@{}", x.1, x.3)),
    });
    let normal = delimited(
        "<",
        (
            alt((
                tag_no_case("https"),
                tag_no_case("http"),
                tag_no_case("ftp"),
                tag_no_case("dict"),
            )),
            ":",
            many1(none_of("'\">\r\n\t\u{B}\u{C}")).map(|_: ()| {}),
        )
            .recognize(),
        ">",
    )
    .context("normal autolink")
    .map(|x| AutoLink {
        target: Cow::Borrowed(x),
    });

    alt((email, normal)).parse_next(input)
}

pub fn parse_link(input: &str) -> IResult<&str, Link> {
    alt((ref_style, inline_style)).parse_next(input)
}

#[cfg(test)]
mod test {
    use winnow::FinishIResult;

    use super::*;

    #[test]
    fn parse_inline() {
        let (remaining, link) = parse_link("[foo](https://github.com/)\n").unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(
            link,
            Link {
                link_text: vec![MarkdownText::Text("foo")],
                link_ref: LinkRef::Inline("https://github.com/"),
                title: None
            }
        )
    }

    #[test]
    fn parse_ref() {
        let (remaining, link) = parse_link("[foo][foo_link]\n").unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(
            link,
            Link {
                link_text: vec![MarkdownText::Text("foo")],
                link_ref: LinkRef::Ref("foo_link"),
                title: None
            }
        )
    }

    #[test]
    fn parse_with_brackets() {
        let link = parse_link("[foo [bar]](https://lib.rs)").finish().unwrap();
        assert_eq!(
            link,
            Link {
                link_text: vec![
                    MarkdownText::Text("foo "),
                    MarkdownText::Text("["),
                    MarkdownText::Text("bar"),
                    MarkdownText::Text("]"),
                ],
                link_ref: LinkRef::Inline("https://lib.rs"),
                title: None,
            }
        )
    }

    #[test]
    fn auto_link() {
        let (remaining, link) = parse_auto_link("<https://lib.rs>").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(link.target, "https://lib.rs");
    }

    #[test]
    fn auto_link_email() {
        let (remaining, link) = parse_auto_link("<noreply@example.com>").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(link.target, "mailto:noreply@example.com");
    }

    #[test]
    fn auto_link_email_mailto() {
        let (remaining, link) = parse_auto_link("<mailto:noreply@example.com>").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(link.target, "mailto:noreply@example.com");
    }

    #[test]
    fn not_auto_link() {
        assert!(parse_auto_link("<noreply>").is_err())
    }
}
