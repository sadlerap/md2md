use winnow::{branch::alt, character::newline, multi::many1, IResult, Parser};

use crate::{AsHtml, AsText};

use super::{
    headers::{parse_header, Header},
    paragraphs::{parse_paragraph, Paragraph},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Block<'source> {
    Paragraph(Paragraph<'source>),
    Heading(Header<'source>),
    Separator(usize),
}

impl<'source> AsHtml for Block<'source> {
    fn write_html<Writer: std::io::Write>(&self, output: &mut Writer) -> std::io::Result<()> {
        match self {
            Block::Paragraph(p) => {
                write!(output, "<p>")?;
                p.write_html(output)?;
                write!(output, "</p>")?;
            }
            Block::Heading(h) => h.write_html(output)?,
            Block::Separator(_) => writeln!(output)?,
        }

        Ok(())
    }
}

impl<'source> AsText for Block<'source> {
    fn write_as_text<Writer: std::io::Write>(&self, output: &mut Writer) -> std::io::Result<()> {
        match self {
            Block::Paragraph(p) => p.write_as_text(output)?,
            Block::Heading(h) => h.write_as_text(output)?,
            Block::Separator(amount) => {
                for _ in 0..*amount {
                    writeln!(output)?
                }
            }
        }
        Ok(())
    }
}

pub fn parse_block(input: &str) -> IResult<&str, Block> {
    alt((
        many1(newline).map(Block::Separator),
        parse_header.map(Block::Heading),
        // try parsing a paragraph last, since we should try to recognize other block types first
        parse_paragraph.map(Block::Paragraph),
    ))
    .context("block")
    .parse_next(input)
}

#[cfg(test)]
mod test {
    use winnow::FinishIResult;

    use crate::parser::{
        headers::HeadingLevel,
        util::MarkdownText::{SoftBreak, Text},
    };

    use super::*;

    #[test]
    fn parse_paragraph() {
        let input = "just a paragraph";
        let block = parse_block(input).finish().unwrap();
        assert_eq!(
            block,
            Block::Paragraph(Paragraph {
                text: vec![Text("just a paragraph")]
            })
        )
    }

    #[test]
    fn parse_paragraph_with_trailing_newline() {
        let input = "just a paragraph\n";
        let block = parse_block(input).finish().unwrap();
        assert_eq!(
            block,
            Block::Paragraph(Paragraph {
                text: vec![Text("just a paragraph"), SoftBreak]
            })
        )
    }

    #[test]
    fn parse_header() {
        let input = "# header";
        let block = parse_block(input).finish().unwrap();
        assert_eq!(
            block,
            Block::Heading(Header::AtxHeader {
                level: HeadingLevel::H1,
                text: vec![Text("header")],
            })
        )
    }

    #[test]
    fn block_stream() {
        let input = "# header\nthis is some text";
        let blocks: Vec<_> = many1(parse_block).parse_next(input).finish().unwrap();
        assert_eq!(
            blocks,
            [
                Block::Heading(Header::AtxHeader {
                    level: HeadingLevel::H1,
                    text: vec![Text("header")]
                }),
                Block::Separator(1),
                Block::Paragraph(Paragraph {
                    text: vec![Text("this is some text")]
                })
            ]
        )
    }

    #[test]
    fn block_neighboring_lines() {
        let input = "foo\nbar";
        let blocks: Vec<_> = many1(parse_block).parse_next(input).finish().unwrap();
        assert_eq!(
            blocks,
            [Block::Paragraph(Paragraph {
                text: vec![Text("foo"), SoftBreak, Text("bar")]
            })]
        )
    }

    #[test]
    fn block_separate_paragraphs() {
        let input = "foo\n\nbar";
        let blocks: Vec<_> = many1(parse_block).parse_next(input).finish().unwrap();
        assert_eq!(
            blocks,
            [
                Block::Paragraph(Paragraph {
                    text: vec![Text("foo")]
                }),
                Block::Separator(2),
                Block::Paragraph(Paragraph {
                    text: vec![Text("bar")]
                })
            ]
        )
    }

    #[test]
    fn trailing_header() {
        let input = "foo\n# bar";
        let blocks: Vec<_> = many1(parse_block).parse_next(input).finish().unwrap();
        assert_eq!(
            blocks,
            [
                Block::Paragraph(Paragraph {
                    text: vec![Text("foo")]
                }),
                Block::Separator(1),
                Block::Heading(Header::AtxHeader {
                    level: HeadingLevel::H1,
                    text: vec![Text("bar")]
                }),
            ]
        )
    }

    // #[test]
    // fn bad_header() {
    //     let input = "test\n\nfoo\n---";
    //     let blocks: Vec<_> = many1(parse_block).parse_next(input).finish().unwrap();
    //     assert_eq!(
    //         blocks,
    //         [
    //             Block::Paragraph(Paragraph {
    //                 text: vec![Text("foo")]
    //             }),
    //             Block::Separator(2),
    //             Block::Heading(Header::SetextHeader {
    //                 level: HeadingLevel::H2,
    //                 level_len: 3,
    //                 text: vec![Text("foo")]
    //             }),
    //         ]
    //     )
    // }
}
