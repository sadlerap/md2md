use winnow::{branch::alt, character::newline, multi::many1, IResult, Parser};

use crate::AsText;

use super::{
    headers::{parse_header, Header},
    paragraphs::{parse_paragraph, Paragraph},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Block<'source> {
    Paragraph(Paragraph<'source>),
    Heading(Header<'source>),
    Separator(usize)
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

impl<'source> AsText for Block<'source> {
    fn write_as_text<Writer: std::io::Write>(&self, output: &mut Writer) -> std::io::Result<()> {
        match self {
            Block::Paragraph(p) => p.write_as_text(output)?,
            Block::Heading(h) => h.write_as_text(output)?,
            Block::Separator(amount) => for _ in 0..*amount { writeln!(output)? },
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use winnow::FinishIResult;

    use crate::parser::{
        headers::HeadingLevel, util::MarkdownText::{Text, SoftBreak},
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
            Block::Heading(Header::AtxHeader{
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
                Block::Heading(Header::AtxHeader{
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
}
