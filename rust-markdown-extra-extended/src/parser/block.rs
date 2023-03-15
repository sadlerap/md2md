use winnow::{branch::alt, IResult, Parser};

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

    use crate::parser::util::MarkdownText;

    use super::*;

    #[test]
    fn parse_paragraph() {
        let input = "just a paragraph";
        let block = parse_block(input).finish().unwrap();
        assert_eq!(
            block,
            Block::Paragraph(Paragraph {
                content: vec![MarkdownText::Text("just a paragraph")]
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
                content: vec![MarkdownText::Text("just a paragraph")]
            })
        )
    }
}
