use winnow::{
    branch::alt,
    bytes::take_until0,
    character::{multispace0, newline, space0},
    combinator::opt,
    sequence::delimited,
    IResult, Parser,
};

use crate::{parser::util::{nested_brackets, nested_parenthesis}, AsText};

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

impl<'a> AsText for Image<'a> {
    fn write_as_text<Writer: std::io::Write>(&self, output: &mut Writer) -> std::io::Result<()> {
        match self.image_ref {
            ImageRef::Ref(r) => write!(output, "![{}][{}]", self.alt_text, r)?,
            ImageRef::Inline(i) => write!(output, "![{}]({})", self.alt_text, i)?,
        }
        Ok(())
    }
}

fn ref_style(input: &str) -> IResult<&str, Image> {
    (
        delimited("![", nested_brackets.recognize(), "]"),
        opt(" "),
        opt((newline, space0)),
        delimited("[", take_until0("]"), "]"),
    )
        .map(|x| Image {
            alt_text: x.0,
            image_ref: ImageRef::Ref(x.3),
            title: None,
        })
        .context("ref-style image")
        .parse_next(input)
}

fn inline_style(input: &str) -> IResult<&str, Image> {
    (
        delimited("![", nested_brackets.recognize(), "]"),
        opt(" "),
        "(",
        multispace0,
        nested_parenthesis,
        multispace0,
        opt(alt((
            delimited("\"", take_until0("\""), ("\"", multispace0)),
            delimited("\'", take_until0("\'"), ("\'", multispace0)),
        ))),
        ")",
    )
        .map(|x| Image {
            alt_text: x.0,
            image_ref: ImageRef::Inline(x.4),
            title: x.6,
        })
        .context("inline image")
        .parse_next(input)
}

pub fn parse_image(input: &str) -> IResult<&str, Image> {
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
    fn parse_ref() {
        let (remaining, image) = dbg!(parse_image("![foo][foo_image]\n")).unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(
            image,
            Image {
                alt_text: "foo",
                image_ref: ImageRef::Ref("foo_image"),
                title: None
            }
        )
    }
}
