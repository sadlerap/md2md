use winnow::{bytes::take_until1, sequence::delimited, IResult, Parser};

use super::util::MarkdownText;

pub fn parse_inline_code(input: &str) -> IResult<&str, MarkdownText> {
    delimited("`", take_until1("`"), "`")
        .context("parse_inline_code")
        .map(|s: &str| MarkdownText::Code { code: s.into() })
        .parse_next(input)
}

#[cfg(test)]
mod test {
    use winnow::FinishIResult;

    use super::*;

    #[test]
    fn inline_code() {
        let text = "`abxy`";
        let code = parse_inline_code(text).finish().unwrap();
        assert_eq!(
            code,
            MarkdownText::Code {
                code: "abxy".into()
            }
        )
    }

    #[test]
    fn just_a_tick() {
        let text = "`abxy";
        assert!(parse_inline_code(text).finish().is_err());
    }

    #[test]
    fn across_lines() {
        let text = "`inline\ncode\nhere`";
        let code = parse_inline_code(text).finish().unwrap();
        assert_eq!(
            code,
            MarkdownText::Code {
                code: "inline\ncode\nhere".into()
            }
        )
    }
}
