use winnow::{
    branch::alt,
    bytes::{tag, take_until0},
    character::{multispace0, newline, space0},
    combinator::opt,
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

fn ref_style(input: &str) -> IResult<&str, Link> {
    (
        delimited(tag("["), nested_brackets.recognize(), tag("]")),
        opt(tag(" ")),
        opt((newline, space0)),
        delimited(tag("["), take_until0("]"), tag("]")),
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
        delimited(tag("["), nested_brackets.recognize(), tag("]")),
        opt(tag(" ")),
        tag("("),
        multispace0,
        nested_parenthesis,
        opt(multispace0),
        opt(alt((
            delimited(tag("\""), take_until0("\""), tag("\"")),
            delimited(tag("\'"), take_until0("\'"), tag("\'")),
        ))),
        opt(multispace0),
        tag(")"),
    )
        .map(|x| Link {
            link_text: x.0,
            link_ref: LinkRef::Inline(x.4),
            title: x.6,
        })
        .context("inline image")
        .parse_next(input)
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
        let (remaining, link) = dbg!(parse_link("[foo](https://github.com/)\n")).unwrap();
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
        let (remaining, link) = dbg!(parse_link("[foo][foo_link]\n")).unwrap();
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
}
