use winnow::{branch::alt, character::newline, multi::many1, sequence::preceded, IResult, Parser};

use super::{
    headers::{parse_header, Header},
    paragraphs::{parse_paragraph, Paragraph},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Block<'source> {
    Paragraph(Paragraph<'source>),
    Heading(Header<'source>),
}

pub fn parse_block(input: &str) -> IResult<&str, Block> {
    alt((
        preceded(many1(newline).map(|_: ()| {}), parse_block),
        parse_header.map(|h| Block::Heading(h)),
        // try parsing a paragraph last, since we should try to recognize other block types first
        parse_paragraph.map(|p| Block::Paragraph(p)),
    ))
    .context("block")
    .parse_next(input)
}

#[cfg(test)]
mod test {
    use winnow::FinishIResult;

    use crate::parser::util::MarkdownText::{self, *};

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
            Block::Heading(Header {
                level: pulldown_cmark::HeadingLevel::H1,
                text: vec![MarkdownText::Text("header")],
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
                Block::Heading(Header {
                    level: pulldown_cmark::HeadingLevel::H1,
                    text: vec![MarkdownText::Text("header")]
                }),
                Block::Paragraph(Paragraph {
                    text: vec![MarkdownText::Text("this is some text")]
                })
            ]
        )
    }
}
