use winnow::{
    branch::alt,
    bytes::none_of,
    multi::many0,
    sequence::delimited,
    IResult, Parser,
};

pub fn nested_brackets(input: &str) -> IResult<&str, &str> {
    many0(alt((
        none_of("[]").context("non-bracketed text").recognize(),
        delimited("[", nested_brackets, "]")
            .context("bracketed text")
            .recognize(),
    )))
    .map(|_: ()| {})
    .recognize()
    .parse_next(input)
}

pub fn nested_parenthesis(input: &str) -> IResult<&str, &str> {
    many0(alt((
        none_of("()").context("non-parenthesis text").recognize(),
        delimited("(", nested_parenthesis, ")")
            .context("parenthetical text")
            .recognize(),
    )))
    .map(|_: ()| {})
    .recognize()
    .parse_next(input)
}
