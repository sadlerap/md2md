use winnow::{
    branch::alt,
    bytes::{any, take_till1, take_while1},
    character::{newline, space0},
    combinator::{backtrack_err, eof, fail, opt},
    dispatch,
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated},
    IResult, Parser,
};

use crate::{AsHtml, AsText};

use super::{util::MarkdownText, paragraphs::take_until_match};

#[derive(Debug, PartialEq, Eq)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Header<'source> {
    AtxHeader {
        level: HeadingLevel,
        text: Vec<MarkdownText<'source>>,
    },
    SetextHeader {
        level: HeadingLevel,
        level_len: usize,
        text: Vec<MarkdownText<'source>>,
    },
}

impl<'source> AsHtml for Header<'source> {
    fn write_html<Writer: std::io::Write>(&self, output: &mut Writer) -> std::io::Result<()> {
        match self {
            Header::AtxHeader { level, text } => {
                let level_tag = match level {
                    HeadingLevel::H1 => "h1",
                    HeadingLevel::H2 => "h2",
                    HeadingLevel::H3 => "h3",
                    HeadingLevel::H4 => "h4",
                    HeadingLevel::H5 => "h5",
                    HeadingLevel::H6 => "h6",
                };
                write!(output, "<{level_tag}>")?;
                for t in text.iter() {
                    t.write_html(output)?;
                }
                write!(output, "</{level_tag}>")?;
            }
            Header::SetextHeader { level, text, .. } => {
                let level_tag = match level {
                    HeadingLevel::H1 => "h1",
                    HeadingLevel::H2 => "h2",
                    HeadingLevel::H3 => "h3",
                    HeadingLevel::H4 => "h4",
                    HeadingLevel::H5 => "h5",
                    HeadingLevel::H6 => "h6",
                };
                write!(output, "<{level_tag}>")?;
                for t in text.iter() {
                    t.write_html(output)?;
                }
                write!(output, "</{level_tag}>")?;
            }
        }

        Ok(())
    }
}

impl<'source> AsText for Header<'source> {
    fn write_as_text<Writer: std::io::Write>(&self, output: &mut Writer) -> std::io::Result<()> {
        match self {
            Header::AtxHeader { level, text } => {
                match level {
                    HeadingLevel::H1 => write!(output, "# ")?,
                    HeadingLevel::H2 => write!(output, "## ")?,
                    HeadingLevel::H3 => write!(output, "### ")?,
                    HeadingLevel::H4 => write!(output, "#### ")?,
                    HeadingLevel::H5 => write!(output, "##### ")?,
                    HeadingLevel::H6 => write!(output, "###### ")?,
                }

                for t in text.iter() {
                    t.write_as_text(output)?;
                }

                Ok(())
            }
            Header::SetextHeader {
                level,
                level_len,
                text,
            } => {
                for t in text.iter() {
                    t.write_as_text(output)?;
                }

                let to_write = if *level == HeadingLevel::H1 { "=" } else { "-" };
                for _ in 0..*level_len {
                    write!(output, "{}", to_write)?;
                }

                Ok(())
            }
        }
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

pub fn setext_header(input: &str) -> IResult<&str, Header> {
    let Some(line) = input.lines()
        .find(|&line| setext_level_from_ending.parse_next(line).is_ok()) else {
        return fail(input);
    };

    // short circuit, we're not a header
    if line.contains("\n\n") {
        return backtrack_err(fail).parse_next(input);
    }

    let line_len = line.len();

    let x = (
        take_until_match(setext_ending).and_then(MarkdownText::parse_markdown_text_stream),
        setext_ending,
    )
        .map(|(text, level)| Header::SetextHeader {
            text,
            level_len: line_len,
            level,
        })
        .parse_next(input);

    x
}

pub fn parse_header(input: &'_ str) -> IResult<&str, Header> {
    let find_until_opt_terminator = |ending: &'static str| {
        alt((
            take_till1("\n"),
            terminated(many0(any).map(|_: ()| {}), eof).recognize(),
        ))
        .and_then(terminated(
            MarkdownText::parse_markdown_text_stream,
            (space0, opt(ending), space0),
        ))
        .context("find until opt terminator".to_string())
    };

    dispatch! {delimited(space0, take_while1("#"), space0);
        "######" => find_until_opt_terminator("######").map(|text| Header::AtxHeader {
            text,
            level: HeadingLevel::H6,
        }),
        "#####" => find_until_opt_terminator("#####").map(|text| Header::AtxHeader {
            text,
            level: HeadingLevel::H5,
        }),
        "####" => find_until_opt_terminator("####").map(|text| Header::AtxHeader {
            text,
            level: HeadingLevel::H4,
        }),
        "###" => find_until_opt_terminator("###").map(|text| Header::AtxHeader {
            text,
            level: HeadingLevel::H3,
        }),
        "##" => find_until_opt_terminator("##").map(|text| Header::AtxHeader {
            text,
            level: HeadingLevel::H2,
        }),
        "#" => find_until_opt_terminator("#").map(|text| Header::AtxHeader {
            text,
            level: HeadingLevel::H1,
        }),
        _ => fail
    }
    .context("atx-style header")
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use winnow::FinishIResult;

    use super::*;

    #[test]
    fn test_atx_header() {
        let (remaining, heading) = parse_header("#Hello, World!\n").unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(
            heading,
            Header::AtxHeader {
                level: HeadingLevel::H1,
                text: vec![MarkdownText::Text("Hello, World"), MarkdownText::Text("!")]
            }
        );
    }

    #[test]
    fn test_setext_header_h1() {
        let (remaining, header) = setext_header("Hello, World!\n============\n").unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(
            header,
            Header::SetextHeader {
                level: HeadingLevel::H1,
                level_len: 12,
                text: vec![MarkdownText::Text("Hello, World"), MarkdownText::Text("!")]
            }
        );
    }

    #[test]
    fn test_setext_header_h2() {
        let (remaining, header) = setext_header("Hello, World!\n------------").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(
            header,
            Header::SetextHeader {
                level: HeadingLevel::H2,
                level_len: 12,
                text: vec![MarkdownText::Text("Hello, World"), MarkdownText::Text("!")]
            }
        );
    }

    #[test]
    fn test_atx_embedded() {
        let header = parse_header("# this isn't a link: [foo]").finish().unwrap();
        assert_eq!(
            header,
            Header::AtxHeader {
                level: HeadingLevel::H1,
                text: MarkdownText::parse_markdown_text_stream("this isn't a link: [foo]")
                    .finish()
                    .unwrap()
            }
        )
    }
}
