use pulldown_cmark::HeadingLevel;
use winnow::{
    branch::alt,
    bytes::{take_till1, take_until1, take_while1},
    character::{newline, space0},
    combinator::{fail, opt},
    dispatch,
    multi::many1,
    sequence::{delimited, preceded, terminated},
    IResult, Parser,
};

use crate::AsText;

use super::util::MarkdownText;

#[derive(Debug, PartialEq, Eq)]
pub struct Header<'source> {
    pub(crate) level: HeadingLevel,
    pub(crate) text: Vec<MarkdownText<'source>>,
}

impl<'source> AsText for Header<'source> {
    fn write_as_text<Writer: std::io::Write>(&self, output: &mut Writer) -> std::io::Result<()> {
        match self.level {
            HeadingLevel::H1 => {
                for t in self.text.iter() {
                    t.write_as_text(output)?;
                }
                writeln!(output, "\n=====")?;
                return Ok(());
            },
            HeadingLevel::H2 => {
                for t in self.text.iter() {
                    t.write_as_text(output)?;
                }
                writeln!(output, "\n-----")?;
                return Ok(());
            },
            HeadingLevel::H3 => write!(output, "### ")?,
            HeadingLevel::H4 => write!(output, "#### ")?,
            HeadingLevel::H5 => write!(output, "##### ")?,
            HeadingLevel::H6 => write!(output, "###### ")?,
        };
        for t in self.text.iter() {
            t.write_as_text(output)?;
        }

        Ok(())
    }
}

fn setext_level_from_ending(input: &str) -> IResult<&str, HeadingLevel> {
    delimited(
        space0,
        alt((
            many1("=").map(|_: ()| HeadingLevel::H1),
            many1("-").map(|_: ()| HeadingLevel::H2),
        )),
        space0,
    )
    .context("setext level")
    .parse_next(input)
}

fn setext_ending(input: &str) -> IResult<&str, HeadingLevel> {
    preceded((newline, space0), setext_level_from_ending)
        .context("setext ending")
        .parse_next(input)
}

fn setext_style(input: &str) -> IResult<&str, Header> {
    let Some(line) = input.lines()
        .filter(|&line| setext_level_from_ending.parse_next(line).is_ok())
        .next() else {
        return fail(input);
    };

    let line = format!("\n{line}");

    let x = (
        take_until1(line.as_str()).and_then(MarkdownText::parse_markdown_text_stream),
        setext_ending,
    )
        .map(|(text, level)| Header { text, level })
        .parse_next(input);

    x
}

pub fn parse_header(input: &'_ str) -> IResult<&str, Header> {
    let find_until_opt_terminator = |ending: &'static str| {
        take_till1("\n")
            .and_then(terminated(
                MarkdownText::parse_markdown_text_stream,
                (space0, opt(ending), space0),
            ))
            .context(format!("find until opt terminator"))
    };

    let atx_style = dispatch! {delimited(space0, take_while1("#"), space0);
        "######" => find_until_opt_terminator("######").map(|text| Header {
            text,
            level: HeadingLevel::H6,
        }),
        "#####" => find_until_opt_terminator("#####").map(|text| Header {
            text,
            level: HeadingLevel::H5,
        }),
        "####" => find_until_opt_terminator("####").map(|text| Header {
            text,
            level: HeadingLevel::H4,
        }),
        "###" => find_until_opt_terminator("###").map(|text| Header {
            text,
            level: HeadingLevel::H3,
        }),
        "##" => find_until_opt_terminator("##").map(|text| Header {
            text,
            level: HeadingLevel::H2,
        }),
        "#" => find_until_opt_terminator("#").map(|text| Header {
            text,
            level: HeadingLevel::H1,
        }),
        _ => fail
    }
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
                text: vec![MarkdownText::Text("Hello, World"), MarkdownText::Text("!")],
            }
        );
    }

    #[test]
    fn test_setext_header_h1() {
        let (remaining, header) = parse_header("Hello, World!\n============\n").unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(
            header,
            Header {
                level: HeadingLevel::H1,
                text: vec![MarkdownText::Text("Hello, World"), MarkdownText::Text("!")]
            }
        );
    }

    #[test]
    fn test_setext_header_h2() {
        let (remaining, header) = parse_header("Hello, World!\n------------").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(
            header,
            Header {
                level: HeadingLevel::H2,
                text: vec![MarkdownText::Text("Hello, World"), MarkdownText::Text("!")]
            }
        );
    }
}
