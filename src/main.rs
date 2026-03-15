use clap::{Parser, Subcommand};

mod validate;
mod vocab;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Validates the incoming RDF data stream
    Validate {
        /// Input file to validate (use '-' for stdin)
        #[arg(default_value = "-")]
        input: String,

        /// Format: ntriples or turtle (auto-detected from extension if omitted)
        #[arg(short, long)]
        format: Option<String>,
    },

    /// Fetches the latest @zazuko/rdf-vocabularies from NPM and rebuilds the local cache
    UpdateVocabs,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Validate { input, format } => {
            validate::run_validation(input, format.as_deref())?;
        }
        Commands::UpdateVocabs => {
            vocab::update_vocabs()?;
            eprintln!("Vocabulary cache updated successfully.");
        }
    }

    Ok(())
}
