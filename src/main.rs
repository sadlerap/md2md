use color_eyre::eyre::Result;
use clap::Parser;
use md2md::{Markdown, AsText};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Where to consume input from
    #[arg(short, long)]
    input: std::path::PathBuf,

    /// Where to write output to
    #[arg(short, long)]
    output: std::path::PathBuf,

    /// Default tab width for converting tabs to spaces.
    #[arg(short, long, default_value_t = 4)]
    tab_width: usize,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let input = std::fs::read_to_string(args.input)?;
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(args.output)?;

    let cleaned_input = md2md::cleanup(&input, args.tab_width);
    let md = Markdown::parse(&cleaned_input)?;

    md.write_as_text(&mut output)?;

    Ok(())
}
