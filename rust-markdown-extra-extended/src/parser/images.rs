use pulldown_cmark::Event;
use winnow::{
    branch::alt,
    bytes::{none_of, tag, take_until0},
    character::{multispace0, newline, space0},
    combinator::opt,
    multi::many0,
    sequence::delimited,
    IResult, Parser,
};

#[derive(Debug, PartialEq, Eq)]
enum ImageRef<'a> {
    Ref(&'a str),
    Inline(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Image<'a> {
    alt_text: &'a str,
    image_ref: ImageRef<'a>,
    title: Option<&'a str>,
}

fn nested_brackets(input: &str) -> IResult<&str, &str> {
    many0(alt((
        none_of("[]").context("non-bracketed text").recognize(),
        delimited(tag("["), nested_brackets, tag("]"))
            .context("bracketed text")
            .recognize(),
    )))
    .map(|_: ()| {})
    .recognize()
    .parse_next(input)
}

fn nested_parenthesis(input: &str) -> IResult<&str, &str> {
    many0(alt((
        none_of("()").context("non-parenthesis text").recognize(),
        delimited(tag("("), nested_parenthesis, tag(")"))
            .context("parenthetical text")
            .recognize(),
    )))
    .map(|_: ()| {})
    .recognize()
    .parse_next(input)
}

pub fn parse_image(input: &'_ str) -> IResult<&str, Image<'_>> {
    let ref_style = (
        tag("!["),
        nested_brackets.recognize(),
        tag("]"),
        opt(tag(" ")),
        opt((newline, space0)),
        tag("["),
        take_until0("]"),
        tag("]"),
    )
        .map(|x| Image {
            alt_text: x.1,
            image_ref: ImageRef::Ref(x.6),
            title: None,
        })
        .context("ref-style image");

    let inline_style = (
        tag("!["),
        nested_brackets.recognize(),
        tag("]"),
        opt(tag(" ")),
        tag("("),
        multispace0,
        nested_parenthesis,
        opt(multispace0),
        opt(alt((
            (tag("\""), take_until0("\""), tag("\""), opt(multispace0)).map(|x| x.1),
            (tag("\'"), take_until0("\'"), tag("\'"), opt(multispace0)).map(|x| x.1),
        ))),
        tag(")"),
    )
        .map(|x| Image {
            alt_text: x.1,
            image_ref: ImageRef::Inline(x.6),
            title: x.8,
        })
        .context("inline image");

    let (remaining, image) = alt((ref_style, inline_style)).parse_next(input)?;
    Ok((remaining, image))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_inline() {
        let (remaining, image) =
            dbg!(parse_image("![foo](https://github.com/favicon.ico)\n")).unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(
            image,
            Image {
                alt_text: "foo",
                image_ref: ImageRef::Inline("https://github.com/favicon.ico"),
                title: None
            }
        )
    }

    #[test]
    fn test2() {
        let (remaining, read) = nested_brackets("foo[bar]]").unwrap();
        assert_eq!(remaining, "]");
        assert_eq!(read, "foo[bar]");
    }
}
