use once_cell::sync::OnceCell;
use pulldown_cmark::HeadingLevel;
use regex::{Regex, RegexBuilder};
use winnow::{
    branch::alt,
    bytes::take_until0,
    character::{newline, space0},
    multi::{many1, many_m_n},
    Parser,
};

static PARSE_ATX_STYLE: OnceCell<Regex> = OnceCell::new();

#[derive(Debug, PartialEq, Eq)]
pub struct Header<'a> {
    level: HeadingLevel,
    text: &'a str,
}

pub fn parse_header(input: &'_ str) -> winnow::IResult<&str, Header> {
    let atx_re = PARSE_ATX_STYLE
        .get_or_try_init(|| {
            RegexBuilder::new(r"(?P<heading>.+)[ ]*\#*\n?")
                .multi_line(true)
                .build()
        })
        .unwrap();

    let setext_style = (
        take_until0("\n"),
        newline.void(),
        alt((
            many1("=").map(|_: ()| HeadingLevel::H1),
            many1("-").map(|_: ()| HeadingLevel::H2),
        )),
        space0.void(),
    )
        .map(|x: (&str, _, HeadingLevel, _)| Header {
            level: x.2,
            text: x.0,
        })
        .context("setext-style header");

    let atx_style = (
        many_m_n(1, 6, "#").map(|depth: usize| {
            match depth {
                1 => HeadingLevel::H1,
                2 => HeadingLevel::H2,
                3 => HeadingLevel::H3,
                4 => HeadingLevel::H4,
                5 => HeadingLevel::H5,
                6 => HeadingLevel::H6,
                _ => unreachable!(), // safe because we only took at most 6 '#'s
            }
        }),
        space0,
        take_until0("\n").map_res(|s: &str| -> Result<&str, &str> {
            let Some(captures) = atx_re.captures(dbg!(s)) else {
                panic!("failed to find matches in ATX regex!");
            };

            let Some(heading) = captures.name("heading") else { panic!("heading not found!"); };
            Ok(heading.as_str())
        }),
    )
        .map(|x: (HeadingLevel, _, &str)| Header {
            level: x.0,
            text: x.2,
        })
        .context("atx-style header");

    let (output, heading) = alt((setext_style, atx_style)).parse_next(input)?;

    Ok((output, heading))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atx_header() {
        let (remaining, heading) = parse_header("#Hello, World!\n").unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(
            heading,
            Header {
                level: HeadingLevel::H1,
                text: "Hello, World!",
            }
        );
    }

    #[test]
    fn test_setext_header() {
        let (remaining, header) = parse_header("Hello, World!\n============\n").unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(
            header,
            Header {
                level: HeadingLevel::H1,
                text: "Hello, World!",
            }
        );
    }
}
