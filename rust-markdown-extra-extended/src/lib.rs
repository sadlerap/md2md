use regex::{Regex, RegexBuilder};

pub mod parser;

/// Parses markdown according to the rules that <https://github.com/egil/php-markdown-extra-extended>
/// uses for parsing
pub struct MarkdownParser {
    bom_re: Regex,
    line_ending_re: Regex,
    detab_re: Regex,
    tab_width: usize,
    strip_whitespace_re: Regex,
}

impl MarkdownParser {
    pub fn new(tab_width: usize) -> Self {
        Self {
            bom_re: RegexBuilder::new(r"^\xEF\xBB\xBF|\x1A")
                .build()
                .expect("failed to build re"),
            line_ending_re: RegexBuilder::new(r"\r\n?")
                .build()
                .expect("failed to build re"),
            detab_re: RegexBuilder::new(r"^.*\t.*$")
                .multi_line(true)
                .build()
                .expect("failed to build re"),
            tab_width,
            strip_whitespace_re: RegexBuilder::new(r"^[ ]+$")
                .multi_line(true)
                .build()
                .expect("failed to build re"),
        }
    }

    fn cleanup(&mut self, data: &str) -> String {
        let data = self.bom_re.replace_all(data, "");
        let mut data = self.line_ending_re.replace_all(&data, "\n").into_owned();

        // upstream does this, so we do it too
        data.push_str("\n\n");
        let data = self.detab_re.replace_all(&data, |captures: &regex::Captures| -> String {
            let Some(x) = captures.get(0).map(|m| m.as_str()) else { return "".into() };
            let mut iter = x.split('\t');
            let Some(mut line) = iter.next().map(|s| s.to_string()) else { return "".to_string() };
            for x in iter {
                let amount = (self.tab_width - x.chars().count()) % self.tab_width;
                std::iter::repeat(" ").take(amount).for_each(|s| line.push_str(s));
            }

            line
        });
        let mut data = self.strip_whitespace_re.replace_all(&data, "").into_owned();

        // same here, even though it's not super necessary
        data.push('\n');
        data
    }

    pub fn parse<'a>(mut self, data: &'a str) -> Result<Markdown<'a>, Box<dyn std::error::Error>> {
        let working_text = self.cleanup(data);

        Markdown::parse(working_text)
    }
}

/// A parsed representation of a Markdown file
pub struct Markdown<'buffer> {
    buffer: String,
    events: Vec<pulldown_cmark::Event<'buffer>>,
}

impl<'buffer> Markdown<'buffer> {
    fn parse<I>(_input: I) -> Result<Markdown<'buffer>, Box<dyn std::error::Error>>
    where
        I: Into<String>,
    {
        unimplemented!("parsing markdown is not yet implemented!")
    }
}
