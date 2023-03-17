use clap::{Parser, ValueEnum};
use color_eyre::eyre::{eyre, Context, Result};
use md2md::{AsHtml, AsText, Markdown};

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
    #[arg(short = 'w', long, default_value_t = 4)]
    tab_width: usize,

    #[arg(value_enum, short = 't', long, default_value_t)]
    output_type: OutputType,
}

#[derive(ValueEnum, Default, Debug, Clone, Copy, PartialEq, Eq)]
enum OutputType {
    /// Write output as markdown
    #[default]
    Markdown,
    Html,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let input = std::fs::read_to_string(&args.input)
        .with_context(|| eyre!("Error reading `{:?}`", &args.input))?;

    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&args.output)
        .with_context(|| eyre!("Failed to open `{:?}` for writing", &args.output))?;

    let cleaned_input = md2md::cleanup(&input, args.tab_width);
    let md = Markdown::parse(&cleaned_input).with_context(|| eyre!("Error parsing markdown"))?;

    match args.output_type {
        OutputType::Markdown => md
            .write_as_text(&mut output)
            .with_context(|| eyre!("Failed to write markdown to `{:?}`", &args.output))?,
        OutputType::Html => md
            .write_html(&mut output)
            .with_context(|| eyre!("Failed to write html to `{:?}`", &args.output))?,
    }

    Ok(())
}
