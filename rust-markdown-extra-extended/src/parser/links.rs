use std::borrow::Cow;

use winnow::{
    branch::alt,
    bytes::{none_of, tag_no_case, take_until0, take_until1},
    character::{multispace0, newline, space0},
    combinator::opt,
    multi::many1,
    sequence::delimited,
    IResult, Parser,
};

use crate::parser::util::{nested_brackets, nested_parenthesis};

#[derive(Debug, PartialEq, Eq)]
enum LinkRef<'a> {
    Ref(&'a str),
    Inline(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Link<'a> {
    link_text: &'a str,
    link_ref: LinkRef<'a>,
    title: Option<&'a str>,
}

/// A link where the target is the same as the text.  In markdown, this is constructed with
/// `<https://example.com>`
#[derive(Debug, PartialEq, Eq)]
pub struct AutoLink<'a> {
    target: Cow<'a, str>,
}

fn ref_style(input: &str) -> IResult<&str, Link> {
    (
        delimited("[", nested_brackets.recognize(), "]"),
        opt(" "),
        opt((newline, space0)),
        delimited("[", take_until0("]"), "]"),
    )
        .map(|x| Link {
            link_text: x.0,
            link_ref: LinkRef::Ref(x.3),
            title: None,
        })
        .context("ref-style image")
        .parse_next(input)
}

fn inline_style(input: &str) -> IResult<&str, Link> {
    (
        delimited("[", nested_brackets.recognize(), "]"),
        opt(" "),
        "(",
        multispace0,
        nested_parenthesis,
        opt(multispace0),
        opt(alt((
            delimited("\"", take_until0("\""), "\""),
            delimited("\'", take_until0("\'"), "\'"),
        ))),
        opt(multispace0),
        ")",
    )
        .map(|x| Link {
            link_text: x.0,
            link_ref: LinkRef::Inline(x.4),
            title: x.6,
        })
        .context("inline image")
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
    let (remaining, image) = alt((ref_style, inline_style)).parse_next(input)?;
    Ok((remaining, image))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_inline() {
        let (remaining, link) = parse_link("[foo](https://github.com/)\n").unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(
            link,
            Link {
                link_text: "foo",
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
                link_text: "foo",
                link_ref: LinkRef::Ref("foo_link"),
                title: None
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
